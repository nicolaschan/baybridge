use crate::{
    client::{DeletionPayload, SetKeyPayload as SetPayload},
    crypto::Signed,
};
use anyhow::Result;
use ed25519_dalek::VerifyingKey;

use super::http::{HttpConnection, NamespaceResponse};

pub enum Connection {
    Http(HttpConnection),
}

impl Connection {
    pub async fn set(&self, payload: Signed<SetPayload>) -> Result<()> {
        match self {
            Connection::Http(http) => http.set(payload).await,
        }
    }

    pub async fn delete(&self, payload: Signed<DeletionPayload>) -> Result<()> {
        match self {
            Connection::Http(http) => http.delete(payload).await,
        }
    }

    pub async fn get(&self, verifying_key: &VerifyingKey, key: &str) -> Result<String> {
        match self {
            Connection::Http(http) => http.get(verifying_key, key).await,
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
