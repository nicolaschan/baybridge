use std::collections::HashMap;

use crate::{
    client::{Event, RelevantEvents},
    crypto::{
        encode::{decode_verifying_key, encode_verifying_key},
        Signed,
    },
    models::Name,
};
use anyhow::Result;
use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Deserialize, Serialize)]
pub struct KeyspaceResponse {
    pub verifying_keys: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct NamespaceResponse {
    pub namespace: String,
    pub mapping: HashMap<String, Vec<Signed<Event>>>,
}

pub struct HttpConnection {
    url: String,
}

impl HttpConnection {
    pub fn new(url: &str) -> HttpConnection {
        HttpConnection {
            url: url.to_string(),
        }
    }

    pub async fn set(&self, payload: Signed<Event>) -> Result<()> {
        let verifying_key_string = encode_verifying_key(&payload.verifying_key);
        let path = format!("{}/keyspace/{}", self.url, verifying_key_string);
        debug!("Setting {} on {}", payload.inner.name(), path);
        reqwest::Client::new()
            .post(&path)
            .json(&payload)
            .send()
            .await?;
        Ok(())
    }

    pub async fn get(&self, verifying_key: &VerifyingKey, name: &Name) -> Result<RelevantEvents> {
        let verifying_key_string = encode_verifying_key(verifying_key);
        let path = format!("{}/keyspace/{}/{}", self.url, verifying_key_string, name);
        debug!("Sending request to {}", path);
        reqwest::Client::new()
            .get(&path)
            .send()
            .await?
            .json::<RelevantEvents>()
            .await
            .map_err(Into::into)
    }

    pub async fn namespace(&self, name: &str) -> Result<NamespaceResponse> {
        let path = format!("{}/namespace/{}", self.url, name);
        debug!("Sending request to {}", path);
        let response: NamespaceResponse = reqwest::Client::new()
            .get(&path)
            .send()
            .await?
            .json()
            .await?;
        Ok(response)
    }

    pub async fn list(&self) -> Result<Vec<VerifyingKey>> {
        let path = format!("{}/keyspace/", self.url);
        debug!("Sending request to {}", path);
        let response: KeyspaceResponse = reqwest::Client::new()
            .get(&path)
            .send()
            .await?
            .json()
            .await?;
        let verifying_keys = response
            .verifying_keys
            .iter()
            .filter_map(|vk| decode_verifying_key(vk).ok())
            .collect();
        Ok(verifying_keys)
    }
}
