use crate::{
    api::{StateHash, SyncEvents},
    client::{Event, RelevantEvents},
    crypto::Signed,
    models::Name,
};
use anyhow::Result;
use ed25519_dalek::VerifyingKey;

use super::http::{HttpConnection, NamespaceResponse};

pub enum Connection {
    Http(HttpConnection),
}

impl Connection {
    pub fn url(&self) -> &str {
        match self {
            Connection::Http(http) => http.url().as_str(),
        }
    }

    pub async fn set(&self, payload: Signed<Event>) -> Result<()> {
        match self {
            Connection::Http(http) => http.set(payload).await,
        }
    }

    pub async fn get(&self, verifying_key: &VerifyingKey, name: &Name) -> Result<RelevantEvents> {
        match self {
            Connection::Http(http) => http.get(verifying_key, name).await,
        }
    }

    pub async fn namespace(&self, name: &str) -> Result<NamespaceResponse> {
        match self {
            Connection::Http(http) => http.namespace(name).await,
        }
    }

    pub async fn state_hash(&self) -> Result<StateHash> {
        match self {
            Connection::Http(http) => http.state_hash().await,
        }
    }

    pub async fn sync_events(&self) -> Result<SyncEvents> {
        match self {
            Connection::Http(http) => http.sync_events().await,
        }
    }
}
