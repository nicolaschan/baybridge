use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json,
};
use baybridge::{
    client::{Event, RelevantEvents},
    configuration::Configuration,
    connectors::http::{KeyspaceResponse, NamespaceResponse},
    crypto::{
        encode::{decode_verifying_key, encode_verifying_key},
        Signed,
    },
    models::Peers,
};
use itertools::Itertools;
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::info;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct AppState {
    database: Arc<Mutex<Connection>>,
    peers: Vec<String>,
}

pub async fn start_http_server(config: &Configuration, peers: Vec<String>) -> Result<()> {
    use axum::{routing::get, Router};
    use tokio::net::TcpListener;

    let database_path = config.server_database_path();
    info!("Using database at {}", database_path.display());
    let database = Connection::open(database_path)?;

    database.execute(
        "CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            verifying_key BLOB NOT NULL,
            name BLOB NOT NULL,
            signed_event BLOB NOT NULL UNIQUE,
            priority BIGINT NOT NULL,
            expires_at INTEGER
        )",
        (),
    )?;

    let database = Arc::new(Mutex::new(database));
    let database_clone = database.clone();
    let state = AppState { database, peers };

    tokio::spawn(async move {
        loop {
            cleanup_expired(&database_clone).await.unwrap();
            sleep(Duration::from_secs(10)).await;
        }
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/peers", get(get_peers))
        .route("/keyspace/", get(list_keyspace))
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
    let version = baybridge::built_info::GIT_VERSION.unwrap_or("unknown");
    let database_guard = state.database.lock().await;
    let key_count: usize = database_guard
        .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
        .unwrap();
    (
        StatusCode::OK,
        format!(
            "A bay bridge server (git:{}) 🌉 with {} keys",
            version, key_count
        ),
    )
}

async fn get_peers(State(state): State<AppState>) -> impl IntoResponse {
    let peers = Peers {
        peers: state.peers.clone(),
    };
    Json(peers)
}

async fn list_keyspace(State(state): State<AppState>) -> impl IntoResponse {
    let database_guard = state.database.lock().await;
    let verifying_keys: Vec<String> = database_guard
        .prepare("SELECT DISTINCT verifying_key FROM events")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|result| result.unwrap())
        .collect();
    (StatusCode::OK, Json(KeyspaceResponse { verifying_keys }))
}

async fn get_name(
    Path((verifying_key_string, name_string)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let database_guard = state.database.lock().await;
    let events = database_guard
        .prepare("SELECT signed_event FROM events WHERE verifying_key = ? AND name = ?")
        .unwrap()
        .query_map(
            params![&verifying_key_string, name_string.as_bytes()],
            |row| {
                let signed_event_serialized: Vec<u8> = row.get(0)?;
                let signed_event: Signed<Event> =
                    bincode::deserialize(&signed_event_serialized).unwrap();
                Ok(signed_event)
            },
        )
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    (StatusCode::OK, Json(RelevantEvents { events }))
}

async fn get_namespace(
    Path(name_string): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let database_guard = state.database.lock().await;
    let event_mapping: HashMap<String, Vec<Signed<Event>>> = database_guard
        .prepare("SELECT verifying_key, signed_event FROM events WHERE name = ?")
        .unwrap()
        .query_map([&name_string.as_bytes()], |row| {
            let verifying_key: String = row.get(0)?;
            let signed_event_serialized: Vec<u8> = row.get(1)?;
            let signed_event: Signed<Event> =
                bincode::deserialize(&signed_event_serialized).unwrap();
            Ok((verifying_key, signed_event))
        })
        .unwrap()
        .filter_map(Result::ok)
        .into_group_map();
    (
        StatusCode::OK,
        Json(NamespaceResponse {
            namespace: name_string,
            mapping: event_mapping,
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

    let normalized_verifying_key = encode_verifying_key(&verifying_key);
    let name = payload.inner.name().as_str().as_bytes();
    let signed_event_serialized = bincode::serialize(&payload).unwrap();
    let priority = payload.inner.priority();
    let expires_at = payload.inner.expires_at();

    let database_guard = state.database.lock().await;
    database_guard
        .execute(
            "INSERT INTO events (verifying_key, name, signed_event, priority, expires_at) VALUES (?, ?, ?, ?, ?)",
            params![normalized_verifying_key, name, signed_event_serialized, priority, expires_at],
        )
        .unwrap();

    (StatusCode::OK, "OK")
}

async fn cleanup_expired(database: &Arc<Mutex<Connection>>) -> Result<()> {
    let now = SystemTime::now();
    let since_epoch = now
        .duration_since(UNIX_EPOCH)
        .expect("Error finding current epoch for expiry cleanup");
    let unix_timestamp = since_epoch.as_secs().to_string();

    let database_guard = database.lock().await;
    database_guard
        .execute(
            "DELETE FROM events WHERE expires_at <= ?",
            (unix_timestamp,),
        )
        .unwrap();
    Ok(())
}
