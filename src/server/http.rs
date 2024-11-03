use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json,
};
use tokio::sync::{Mutex, MutexGuard};
use tokio::time::{sleep, Duration};
use tracing::info;

use crate::{
    client::{Event, RelevantEvents},
    configuration::Configuration,
    connectors::http::NamespaceResponse,
    crypto::{encode::decode_verifying_key, Signed},
    models::{self, Peers},
    server::{sqlite_controller::SqliteController, tasks::gc_expired},
};

#[derive(Clone)]
pub struct AppState {
    database: Arc<Mutex<SqliteController>>,
    peers: Vec<String>,
}

pub async fn start_http_server(config: &Configuration, peers: Vec<String>) -> Result<()> {
    use axum::{routing::get, Router};
    use tokio::net::TcpListener;

    let database_path = config.server_database_path();
    info!("Using database at {}", database_path.display());
    let database = SqliteController::new(&database_path)?;

    let database = Arc::new(Mutex::new(database));
    let database_clone = database.clone();
    let state = AppState { database, peers };

    tokio::spawn(async move {
        loop {
            gc_expired::run(&database_clone).await.unwrap();
            sleep(Duration::from_secs(10)).await;
        }
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/state", get(current_state))
        .route("/peers", get(get_peers))
        .route("/keyspace/:verifying_key", post(set_event))
        .route("/keyspace/:verifying_key/:address_key", get(get_name))
        .route("/namespace/:address_key", get(get_namespace))
        .with_state(state);

    let bind_address = "0.0.0.0:3000";
    info!("Listening on {}", bind_address);
    let listener = TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn root(State(state): State<AppState>) -> impl IntoResponse {
    let version = crate::built_info::GIT_VERSION.unwrap_or("unknown");
    let database_guard = state.database.lock().await;
    let current_state = current_state_hash(&database_guard);
    let key_count: usize = database_guard.event_count().unwrap();
    (
        StatusCode::OK,
        format!(
            "A bay bridge server (git:{}) 🌉 with {} events, state: {}",
            version, key_count, current_state
        ),
    )
}

async fn current_state(State(state): State<AppState>) -> impl IntoResponse {
    let database_guard = state.database.lock().await;
    let hash = current_state_hash(&database_guard);
    Json(models::Hash(hash))
}

fn current_state_hash(database_guard: &MutexGuard<'_, SqliteController>) -> blake3::Hash {
    let all_signed_events = database_guard.signed_events().unwrap();
    let serialized_events = bincode::serialize(&all_signed_events).unwrap();
    blake3::hash(&serialized_events)
}

async fn get_peers(State(state): State<AppState>) -> impl IntoResponse {
    let peers = Peers {
        peers: state.peers.clone(),
    };
    Json(peers)
}

async fn get_name(
    Path((verifying_key_string, name_string)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let database_guard = state.database.lock().await;
    let events = database_guard
        .events_by_key_and_name(verifying_key_string, name_string)
        .unwrap();
    (StatusCode::OK, Json(RelevantEvents { events }))
}

async fn get_namespace(
    Path(name_string): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let database_guard = state.database.lock().await;
    let events = database_guard.events_by_namespace(&name_string).unwrap();
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
    Json(payload): Json<Signed<Event>>,
) -> impl IntoResponse {
    let verifying_key = decode_verifying_key(&verifying_key_string).unwrap();
    let verified = payload.verify(&verifying_key);
    if !verified {
        // Return 403 Forbidden
        return (StatusCode::FORBIDDEN, "Forbidden");
    }

    let name = payload.inner.name().clone();
    let priority = payload.inner.priority();
    let expires_at = payload.inner.expires_at();

    let database_guard = state.database.lock().await;
    database_guard
        .insert_event(verifying_key, name, payload, priority, expires_at)
        .unwrap();

    (StatusCode::OK, "OK")
}
