use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;
use tokio::sync::Mutex;

pub async fn run(database: &Arc<Mutex<Connection>>) -> anyhow::Result<()> {
    let now = SystemTime::now();
    let since_epoch = now
        .duration_since(UNIX_EPOCH)
        .expect("Error finding current epoch for expiry cleanup");
    let unix_timestamp = since_epoch.as_secs().to_string();

    let database_guard = database.lock().await;
    let num_events_deleted = database_guard
        .execute(
            "DELETE FROM events WHERE expires_at <= ?",
            (unix_timestamp,),
        )
        .unwrap();
    if num_events_deleted > 0 {
        tracing::debug!("Deleted {} expired events", num_events_deleted);
    }
    Ok(())
}
