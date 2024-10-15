use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json,
};
use baybridge::{
    client::{DeletionPayload, SetKeyPayload},
    configuration::Configuration,
    connectors::http::{KeyspaceResponse, NamespaceResponse},
    crypto::{
        encode::{decode_verifying_key, encode_verifying_key},
        Signed,
    },
    models::Value,
};
use rusqlite::Connection;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::info;

use std::time::{SystemTime, UNIX_EPOCH};

pub async fn start_http_server(config: &Configuration) -> Result<()> {
    use axum::{routing::get, Router};
    use tokio::net::TcpListener;

    let database_path = config.server_database_path();
    info!("Using database at {}", database_path.display());
    let database = Connection::open(database_path)?;

    database.execute(
        "CREATE TABLE IF NOT EXISTS contents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            verifying_key BLOB NOT NULL,
            key BLOB NOT NULL,
            value BLOB NOT NULL,
            expires_at INTEGER
        )",
        (),
    )?;

    let database = Arc::new(Mutex::new(database));

    let database_clone = database.clone();
    tokio::spawn(async move {
        loop {
            cleanup_expired(&database_clone).await.unwrap();
            sleep(Duration::from_secs(10)).await;
        }
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/keyspace/", get(list_keyspace))
        .route("/keyspace/:verifying_key", post(set_key))
        .route(
            "/keyspace/:verifying_key/:address_key",
            get(get_key).delete(delete_name),
        )
        .route("/namespace/:address_key", get(get_namespace))
        .with_state(database);

    let bind_address = "0.0.0.0:3000";
    info!("Listening on {}", bind_address);
    let listener = TcpListener::bind(bind_address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn root(State(database): State<Arc<Mutex<Connection>>>) -> impl IntoResponse {
    let database_guard = database.lock().await;
    let version = baybridge::built_info::GIT_VERSION.unwrap_or("unknown");
    let key_count: usize = database_guard
        .query_row("SELECT COUNT(*) FROM contents", [], |row| row.get(0))
        .unwrap();
    (
        StatusCode::OK,
        format!(
            "A bay bridge server (git:{}) 🌉 with {} keys",
            version, key_count
        ),
    )
}

async fn list_keyspace(State(database): State<Arc<Mutex<Connection>>>) -> impl IntoResponse {
    let database_guard = database.lock().await;
    let verifying_keys: Vec<String> = database_guard
        .prepare("SELECT DISTINCT verifying_key FROM contents")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|result| result.unwrap())
        .collect();
    (StatusCode::OK, Json(KeyspaceResponse { verifying_keys }))
}

async fn get_key(
    Path((verifying_key_string, key_string)): Path<(String, String)>,
    State(database): State<Arc<Mutex<Connection>>>,
) -> impl IntoResponse {
    let database_guard = database.lock().await;
    let result: (Vec<u8>, Option<u64>) = database_guard
        .query_row(
            "SELECT value, expires_at FROM contents WHERE verifying_key = ? AND key = ?",
            (&verifying_key_string, &key_string.as_bytes()),
            |row| {
                row.get(0)
                    .and_then(|v: Vec<u8>| row.get(1).map(|e: Option<u64>| (v, e)))
            },
        )
        .unwrap();
    (StatusCode::OK, Json(Value::new(result.0, result.1)))
}

async fn get_namespace(
    Path(key_string): Path<String>,
    State(database): State<Arc<Mutex<Connection>>>,
) -> impl IntoResponse {
    let database_guard = database.lock().await;
    let mut statement = database_guard
        .prepare("SELECT verifying_key, value, expires_at FROM contents WHERE key = ?")
        .unwrap();
    let result = statement.query([&key_string.as_bytes()]).unwrap();
    let namespace: Vec<(String, Vec<u8>, Option<u64>)> = result
        .mapped(|row| {
            Ok((
                row.get(0).unwrap(),
                row.get(1).unwrap(),
                row.get(2).unwrap(),
            ))
        })
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    let mapping = namespace
        .into_iter()
        .map(|(verifying_key, value_bytes, expires_at)| {
            (verifying_key, Value::new(value_bytes, expires_at))
        })
        .collect();
    (
        StatusCode::OK,
        Json(NamespaceResponse {
            namespace: key_string,
            mapping,
        }),
    )
}

async fn set_key(
    Path(verifying_key_string): Path<String>,
    State(database): State<Arc<Mutex<Connection>>>,
    Json(payload): Json<Signed<SetKeyPayload>>,
) -> impl IntoResponse {
    let verifying_key = decode_verifying_key(&verifying_key_string).unwrap();
    let verified = payload.verify(&verifying_key);
    if !verified {
        // Return 403 Forbidden
        return (StatusCode::FORBIDDEN, "Forbidden");
    }

    let database_guard = database.lock().await;
    database_guard
        .execute(
            "INSERT INTO contents (verifying_key, key, value) VALUES (?, ?, ?)",
            (
                &encode_verifying_key(&verifying_key), // Use the normalized key
                payload.inner.key.as_bytes(),
                payload.inner.value.as_bytes(),
            ),
        )
        .unwrap();

    (StatusCode::OK, "OK")
}

async fn delete_name(
    Path((verifying_key_string, key_string)): Path<(String, String)>,
    State(database): State<Arc<Mutex<Connection>>>,
    Json(payload): Json<Signed<DeletionPayload>>,
) -> impl IntoResponse {
    let verifying_key = decode_verifying_key(&verifying_key_string).unwrap();
    let verified = payload.verify(&verifying_key);
    if !verified {
        // Return 403 Forbidden
        return (StatusCode::FORBIDDEN, "Forbidden");
    }

    let database_guard = database.lock().await;
    database_guard
        .execute(
            "DELETE FROM contents WHERE verifying_key = ? AND key = ?",
            (&verifying_key_string, &key_string.as_bytes()),
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
            "DELETE FROM contents WHERE expires_at <= ?",
            (unix_timestamp,),
        )
        .unwrap();
    Ok(())
}
