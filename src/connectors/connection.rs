use crate::{
    client::{DeletionEvent, Event, RelevantEvents},
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
    pub async fn set(&self, payload: Signed<Event>) -> Result<()> {
        match self {
            Connection::Http(http) => http.set(payload).await,
        }
    }

    pub async fn delete(&self, payload: Signed<DeletionEvent>) -> Result<()> {
        match self {
            Connection::Http(http) => http.delete(payload).await,
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

    pub async fn list(&self) -> Result<Vec<VerifyingKey>> {
        match self {
            Connection::Http(http) => http.list().await,
        }
    }
}
