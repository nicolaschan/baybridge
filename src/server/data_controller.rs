use crate::{api::StateHash, client::Event, crypto::Signed};

use super::sqlite_store::SqliteStore;

#[derive(Clone)]
pub struct DataController {
    store: SqliteStore,
}

impl DataController {
    pub fn new(store: SqliteStore) -> Self {
        Self { store }
    }

    pub async fn get_peer_last_hash(&self, url: &str) -> Option<StateHash> {
        self.store.get_peer_last_hash(url).await
    }

    pub async fn set_peer_last_hash(
        &self,
        url: &str,
        events_hash: StateHash,
    ) -> anyhow::Result<()> {
        self.store.set_peer_last_hash(url, events_hash).await
    }

    pub async fn current_state_hash(&self) -> anyhow::Result<StateHash> {
        self.store.current_state_hash().await
    }

    pub async fn delete_expired_events(&self, unix_timestamp: u64) -> anyhow::Result<usize> {
        self.store.delete_expired_events(unix_timestamp).await
    }

    pub async fn insert_event(&self, event: Signed<Event>) -> anyhow::Result<usize> {
        self.store.insert_event(event).await
    }

    pub async fn insert_events(&self, events: Vec<Signed<Event>>) -> anyhow::Result<()> {
        for event in events {
            self.insert_event(event).await?;
        }
        Ok(())
    }

    pub async fn event_count(&self) -> anyhow::Result<usize> {
        self.store.event_count().await
    }

    pub async fn signed_events(&self) -> anyhow::Result<Vec<Signed<Event>>> {
        self.store.signed_events().await
    }

    pub async fn events_by_key_and_name(
        &self,
        verifying_key: String,
        name: String,
    ) -> anyhow::Result<Vec<Signed<Event>>> {
        self.store.events_by_key_and_name(verifying_key, name).await
    }

    pub async fn events_by_namespace(&self, namespace: &str) -> anyhow::Result<Vec<Signed<Event>>> {
        self.store.events_by_namespace(namespace).await
    }
}
