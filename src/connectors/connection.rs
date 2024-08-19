use crate::{client::SetKeyPayload as SetPayload, crypto::Signed};
use anyhow::Result;
use ed25519_dalek::VerifyingKey;

use super::http::HttpConnection;

pub enum Connection {
    Http(HttpConnection),
}

impl Connection {
    pub async fn set(&self, payload: Signed<SetPayload>) -> Result<()> {
        match self {
            Connection::Http(http) => http.set(payload).await,
        }
    }

    pub async fn get(&self, verifying_key: &VerifyingKey, key: &str) -> Result<String> {
        match self {
            Connection::Http(http) => http.get(verifying_key, key).await,
        }
    }

    pub async fn list(&self) -> Result<Vec<VerifyingKey>> {
        match self {
            Connection::Http(http) => http.list().await,
        }
    }
}
