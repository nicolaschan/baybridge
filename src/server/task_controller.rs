use tracing::warn;

use crate::connectors::connection::Connection;

use super::{sqlite_controller::SqliteController, tasks};

#[derive(bon::Builder)]
pub struct TaskController {
    controller: SqliteController,
    #[builder(default)]
    peer_connections: Vec<Connection>,
}

impl TaskController {
    pub async fn run_tasks(&self) -> anyhow::Result<()> {
        tasks::gc_expired::run(&self.controller).await?;

        for connection in &self.peer_connections {
            if let Err(e) = tasks::sync::run(&self.controller, connection).await {
                warn!(
                    "Failed to synchronize with connection {}: {:?}",
                    connection.url(),
                    e
                );
            }
        }

        Ok(())
    }
}
