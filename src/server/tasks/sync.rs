use bincode::config::standard;
use tracing::debug;

use crate::{
    api::StateHash, connectors::connection::Connection, server::data_controller::DataController,
};

pub async fn run(controller: &DataController, connection: &Connection) -> anyhow::Result<()> {
    let last_sync_hash = controller.get_peer_last_hash(connection.url()).await;
    let other_state = connection.state_hash().await?;
    if last_sync_hash
        .map(|hash| hash == other_state)
        .unwrap_or(false)
    {
        debug!("Already synchronized with {}", connection.url());
        return Ok(());
    }

    let other_events = connection.sync_events().await?;
    debug!(
        "Importing {} events from {}",
        other_events.events.len(),
        connection.url()
    );

    let serialized_events = bincode::encode_to_vec(&other_events.events, standard())?;
    let events_hash = StateHash {
        hash: blake3::hash(&serialized_events),
    };

    controller.insert_events(other_events.events).await?;
    controller
        .set_peer_last_hash(connection.url(), events_hash)
        .await?;

    Ok(())
}
