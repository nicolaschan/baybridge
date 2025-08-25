use crate::{
    api::{StateHash, SyncEvents},
    client::{Event, RelevantEvents},
    crypto::{Signed, encode::encode_verifying_key},
    models::Name,
};
use anyhow::Result;
use ed25519_dalek::VerifyingKey;
use failsafe::futures::CircuitBreaker;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Deserialize, Serialize)]
pub struct NamespaceResponse {
    pub namespace: String,
    pub events: Vec<Signed<Event>>,
}

impl NamespaceResponse {
    pub fn merge(&mut self, mut other: NamespaceResponse) {
        self.events.append(&mut other.events)
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
    url: url::Url,
    client: reqwest::Client,
    circuit_breaker: failsafe::StateMachine<
        failsafe::failure_policy::OrElse<
            failsafe::failure_policy::SuccessRateOverTimeWindow<failsafe::backoff::EqualJittered>,
            failsafe::failure_policy::ConsecutiveFailures<failsafe::backoff::EqualJittered>,
        >,
        (),
    >,
}

impl HttpConnection {
    pub fn new(url: url::Url) -> HttpConnection {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        let circuit_breaker = failsafe::Config::new().build();
        HttpConnection {
            url,
            client,
            circuit_breaker,
        }
    }

    pub fn url(&self) -> &url::Url {
        &self.url
    }

    pub async fn set(&self, payload: Signed<Event>) -> Result<()> {
        let verifying_key_string = encode_verifying_key(&payload.verifying_key);
        let url = self.url.join(&format!("keyspace/{verifying_key_string}"))?;
        debug!("Setting {} on {}", payload.inner.name(), url.as_str());
        let request_future = self.client.post(url.as_str()).json(&payload).send();
        self.circuit_breaker.call(request_future).await?;
        Ok(())
    }

    pub async fn get(&self, verifying_key: &VerifyingKey, name: &Name) -> Result<RelevantEvents> {
        let verifying_key_string = encode_verifying_key(verifying_key);
        let url = self
            .url
            .join(&format!("keyspace/{verifying_key_string}/{name}"))?;
        debug!("Sending request to {}", url.as_str());
        let request_future = self.client.get(url.as_str()).send();
        let response = self.circuit_breaker.call(request_future).await?;
        response.json::<RelevantEvents>().await.map_err(Into::into)
    }

    pub async fn namespace(&self, name: &str) -> Result<NamespaceResponse> {
        let url = self.url.join(&format!("namespace/{name}"))?;
        debug!("Sending request to {}", url.as_str());
        let request_future = self.client.get(url.as_str()).send();
        let response = self.circuit_breaker.call(request_future).await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn state_hash(&self) -> Result<StateHash> {
        let url = self.url.join("sync/state")?;
        debug!("Sending request to {}", url.as_str());
        let request_future = self.client.get(url.as_str()).send();
        let response = self.circuit_breaker.call(request_future).await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn sync_events(&self) -> Result<SyncEvents> {
        let url = self.url.join("sync/events")?;
        debug!("Sending request to {}", url.as_str());
        let request_future = self.client.get(url.as_str()).send();
        let response = self.circuit_breaker.call(request_future).await?;
        response.json().await.map_err(Into::into)
    }

    pub async fn get_immutable(&self, hash: &blake3::Hash) -> Result<Vec<u8>> {
        let url = self.url.join(&format!("immutable/{hash}"))?;
        debug!("Sending request to {}", url.as_str());
        let request_future = self.client.get(url.as_str()).send();
        let response = self.circuit_breaker.call(request_future).await?;
        response.json::<Vec<u8>>().await.map_err(Into::into)
    }

    pub async fn set_immutable(&self, data: Vec<u8>) -> Result<blake3::Hash> {
        let url = self.url.join("immutable")?;
        debug!("Setting immutable data on {}", url.as_str());
        let request_future = self.client.post(url.as_str()).json(&data).send();
        let response = self.circuit_breaker.call(request_future).await?;
        response.json::<blake3::Hash>().await.map_err(Into::into)
    }
}
