use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio::sync::Mutex;

use crate::server::sqlite_controller::SqliteController;

pub async fn run(database: &Arc<Mutex<SqliteController>>) -> anyhow::Result<()> {
    let now = SystemTime::now();
    let since_epoch = now
        .duration_since(UNIX_EPOCH)
        .expect("Error finding current epoch for expiry cleanup");
    let unix_timestamp = since_epoch.as_secs();

    let database_guard = database.lock().await;
    let num_events_deleted = database_guard.delete_expired_events(unix_timestamp)?;
    if num_events_deleted > 0 {
        tracing::debug!("Deleted {} expired events", num_events_deleted);
    }
    Ok(())
}
