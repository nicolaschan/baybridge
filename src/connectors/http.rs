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

impl NamespaceResponse {
    pub fn merge(&mut self, other: NamespaceResponse) {
        for (key, events) in other.mapping {
            self.mapping
                .entry(key)
                .and_modify(|existing| existing.extend(events.clone()))
                .or_insert(events);
        }
    }

    pub fn merge_vec(namespace_responses: Vec<NamespaceResponse>) -> Option<NamespaceResponse> {
        let mut namespace_responses = namespace_responses.into_iter();
        let mut merged = namespace_responses.next()?;
        for response in namespace_responses {
            merged.merge(response);
        }
        Some(merged)
    }
}

pub struct HttpConnection {
    url: String,
    client: reqwest::Client,
}

impl HttpConnection {
    pub fn new(url: &str) -> HttpConnection {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        HttpConnection {
            url: url.to_string(),
            client,
        }
    }

    pub async fn set(&self, payload: Signed<Event>) -> Result<()> {
        let verifying_key_string = encode_verifying_key(&payload.verifying_key);
        let path = format!("{}/keyspace/{}", self.url, verifying_key_string);
        debug!("Setting {} on {}", payload.inner.name(), path);
        self.client.post(&path).json(&payload).send().await?;
        Ok(())
    }

    pub async fn get(&self, verifying_key: &VerifyingKey, name: &Name) -> Result<RelevantEvents> {
        let verifying_key_string = encode_verifying_key(verifying_key);
        let path = format!("{}/keyspace/{}/{}", self.url, verifying_key_string, name);
        debug!("Sending request to {}", path);
        self.client
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
        let response: NamespaceResponse = self.client.get(&path).send().await?.json().await?;
        Ok(response)
    }

    pub async fn list(&self) -> Result<Vec<VerifyingKey>> {
        let path = format!("{}/keyspace/", self.url);
        debug!("Sending request to {}", path);
        let response: KeyspaceResponse = self.client.get(&path).send().await?.json().await?;
        let verifying_keys = response
            .verifying_keys
            .iter()
            .filter_map(|vk| decode_verifying_key(vk).ok())
            .collect();
        Ok(verifying_keys)
    }
}
