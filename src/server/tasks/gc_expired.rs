use std::time::{SystemTime, UNIX_EPOCH};

use crate::server::data_controller::DataController;

pub async fn run(controller: &DataController) -> anyhow::Result<()> {
    let now = SystemTime::now();
    let since_epoch = now
        .duration_since(UNIX_EPOCH)
        .expect("Error finding current epoch for expiry cleanup");
    let unix_timestamp = since_epoch.as_secs();

    let num_events_deleted = controller.delete_expired_events(unix_timestamp).await?;
    if num_events_deleted > 0 {
        tracing::debug!("Deleted {} expired events", num_events_deleted);
    }
    Ok(())
}
