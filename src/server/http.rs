use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json,
};
use tokio::time::{sleep, Duration};
use tower_http::services::ServeDir;
use tracing::info;

use crate::{
    api::SyncEvents,
    client::{Event, RelevantEvents},
    configuration::Configuration,
    connectors::{
        connection::Connection,
        http::{HttpConnection, NamespaceResponse},
    },
    crypto::{encode::decode_verifying_key, Signed},
    models::Peers,
    server::{sqlite_controller::SqliteController, task_controller::TaskController},
};

use super::templates;

#[derive(Clone)]
pub struct AppState {
    controller: SqliteController,
    peers: Vec<String>,
}

pub async fn start_http_server(config: &Configuration, peers: Vec<String>) -> Result<()> {
    use axum::{routing::get, Router};
    use tokio::net::TcpListener;

    let database_path = config.server_database_path();
    info!("Using database at {}", database_path.display());
    let controller = SqliteController::new(&database_path)?;
    let peer_connections = peers
        .iter()
        .map(|peer| Connection::Http(HttpConnection::new(peer)))
        .collect();

    let task_controller = TaskController::builder()
        .controller(controller.clone())
        .peer_connections(peer_connections)
        .build();
    let state = AppState { controller, peers };

    tokio::spawn(async move {
        loop {
            task_controller.run_tasks().await.unwrap();
            sleep(Duration::from_secs(10)).await;
        }
    });

    let app = Router::new()
        .route("/", get(dashboard))
        .route("/info", get(info))
        .route("/keyspace/:verifying_key", post(set_event))
        .route("/keyspace/:verifying_key/:address_key", get(get_name))
        .route("/namespace/:address_key", get(get_namespace))
        .route("/sync/peers", get(sync_peers))
        .route("/sync/state", get(sync_state))
        .route("/sync/events", get(sync_events))
        .nest_service("/dist", ServeDir::new("dist"))
        .with_state(state);

    let bind_address = "0.0.0.0:3000";
    info!("Listening on {}", bind_address);
    let listener = TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn dashboard(State(state): State<AppState>) -> impl IntoResponse {
    let version = crate::built_info::GIT_VERSION
        .unwrap_or("unknown")
        .to_string();
    let state_hash = state
        .controller
        .current_state_hash()
        .await
        .unwrap()
        .hash
        .to_string()
        .chars()
        .take(12)
        .collect();
    let event_count = state.controller.event_count().await.unwrap();
    let peer_count = state.peers.len();
    templates::Dashboard {
        state_hash,
        version,
        event_count,
        peer_count,
    }
}

async fn info(State(state): State<AppState>) -> impl IntoResponse {
    let version = crate::built_info::GIT_VERSION.unwrap_or("unknown");
    let current_state = state.controller.current_state_hash().await.unwrap();
    let key_count: usize = state.controller.event_count().await.unwrap();
    (
        StatusCode::OK,
        format!(
            "A bay bridge server (git:{}) 🌉 with {} events, state: {:?}",
            version, key_count, current_state
        ),
    )
}

async fn sync_state(State(state): State<AppState>) -> impl IntoResponse {
    let hash = state.controller.current_state_hash().await.unwrap();
    Json(hash)
}

async fn sync_peers(State(state): State<AppState>) -> impl IntoResponse {
    let peers = Peers {
        peers: state.peers.clone(),
    };
    Json(peers)
}

async fn sync_events(State(state): State<AppState>) -> impl IntoResponse {
    let events = state.controller.signed_events().await.unwrap();
    Json(SyncEvents { events })
}

async fn get_name(
    Path((verifying_key_string, name_string)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let events = state
        .controller
        .events_by_key_and_name(verifying_key_string, name_string)
        .await
        .unwrap();
    (StatusCode::OK, Json(RelevantEvents { events }))
}

async fn get_namespace(
    Path(name_string): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let events = state
        .controller
        .events_by_namespace(&name_string)
        .await
        .unwrap();
    (
        StatusCode::OK,
        Json(NamespaceResponse {
            namespace: name_string,
            events,
        }),
    )
}

async fn set_event(
    Path(verifying_key_string): Path<String>,
    State(state): State<AppState>,
    Json(event): Json<Signed<Event>>,
) -> impl IntoResponse {
    let verifying_key = decode_verifying_key(&verifying_key_string).unwrap();
    let verified = event.verify(&verifying_key);
    if !verified {
        // Return 403 Forbidden
        return (StatusCode::FORBIDDEN, "Forbidden");
    }

    state.controller.insert_event(event).await.unwrap();

    (StatusCode::OK, "OK")
}
