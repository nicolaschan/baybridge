use std::{collections::HashMap, time::UNIX_EPOCH};

use crate::{
    configuration::Configuration,
    connectors::http::NamespaceResponse,
    crdt::merge_events,
    crypto::{
        CryptoKey, Signed,
        encode::{decode_verifying_key, encode_verifying_key},
    },
    models::{ContentBlock, Name, NamespaceValues, Value},
};
use anyhow::Result;
use bon::bon;
use ed25519_dalek::VerifyingKey;
use futures::future::join_all;
use itertools::Itertools;
use std::time::{Duration, SystemTime};

use super::{DeletionEvent, Event, SetEvent};

pub struct Actions {
    pub config: Configuration,
}

pub enum Expiry {
    ExpiresAt(u64),
    Ttl(Duration),
}

#[bon]
impl Actions {
    pub fn new(config: Configuration) -> Actions {
        Actions { config }
    }

    #[builder]
    pub async fn set(
        &self,
        name: Name,
        value: Value,
        expiry: Option<Expiry>,
        priority: Option<u64>,
    ) -> Result<()> {
        let mut crypto_key = CryptoKey::from_config(&self.config).await;

        let now = SystemTime::now();
        let since_epoch = now
            .duration_since(UNIX_EPOCH)
            .expect("Error finding current epoch for expiry cleanup");
        let unix_timestamp = since_epoch.as_secs();

        let priority = match priority {
            Some(priority) => priority,
            None => unix_timestamp,
        };

        let expires_at = match expiry {
            Some(expiry) => match expiry {
                Expiry::ExpiresAt(expires_at) => Some(expires_at),
                Expiry::Ttl(ttl) => Some(unix_timestamp + ttl.as_secs()),
            },
            None => None,
        };

        let event = Event::Set(SetEvent {
            name,
            value,
            priority,
            expires_at,
        });
        let signed = crypto_key.sign(event);

        let set_futures = self
            .config
            .get_connections()
            .iter()
            .map(|connection| connection.set(signed.clone()));
        join_all(set_futures).await;

        Ok(())
    }

    pub async fn delete(&self, name: Name) -> Result<()> {
        let mut crypto_key = CryptoKey::from_config(&self.config).await;

        let event = Event::Delete(DeletionEvent { name, priority: 0 });
        let signed = crypto_key.sign(event);

        let set_futures = self
            .config
            .get_connections()
            .iter()
            .map(|conn| conn.set(signed.clone()));
        join_all(set_futures).await;
        Ok(())
    }

    pub async fn get(&self, verifying_key_string: &str, name: &Name) -> Result<Value> {
        let verifying_key = decode_verifying_key(verifying_key_string)?;
        let relevant_events_futures = self
            .config
            .get_connections()
            .iter()
            .map(|conn| conn.get(&verifying_key, name))
            .collect::<Vec<_>>();
        let combined_events = join_all(relevant_events_futures)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .flat_map(|events| events.events.into_iter())
            .collect();
        // TODO: filter by ttl and verify events
        let value = merge_events(combined_events);
        match value {
            Some(value) => Ok(value),
            None => Err(anyhow::anyhow!("Value not found")),
        }
    }

    pub async fn get_by_key(&self, verifying_key: &VerifyingKey, name: &Name) -> Result<Value> {
        self.get(&encode_verifying_key(verifying_key), name).await
    }

    pub async fn get_mine(&self, name: &Name) -> Result<Value> {
        let verifying_key = self.whoami().await;
        self.get_by_key(&verifying_key, name).await
    }

    pub async fn namespace(&self, name: &str) -> Result<NamespaceValues> {
        let namespace_futures = self
            .config
            .get_connections()
            .iter()
            .map(|conn| conn.namespace(name));
        let namespace_responses = join_all(namespace_futures)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect();
        let merged_namespace = (match NamespaceResponse::merge_vec(namespace_responses) {
            Some(response) => Ok(response),
            None => Err(anyhow::anyhow!("Namespace not found")),
        })?;
        let event_mapping: HashMap<VerifyingKey, Vec<Signed<Event>>> = merged_namespace
            .events
            .into_iter()
            .map(|event| (event.verifying_key, event))
            .into_group_map();
        let value_mapping = event_mapping
            .iter()
            .map(|(k, v)| {
                let value = merge_events(v.clone());
                match value {
                    Some(value) => Ok((*k, value)),
                    None => Err(anyhow::anyhow!("Value not found")),
                }
            })
            .filter_map(Result::ok)
            .collect();
        Ok(NamespaceValues {
            namespace: merged_namespace.namespace,
            mapping: value_mapping,
        })
    }

    pub async fn whoami(&self) -> VerifyingKey {
        let crypto_key = CryptoKey::from_config(&self.config).await;
        crypto_key.verifying()
    }

    pub async fn get_immutable(&self, hash: &blake3::Hash) -> Result<ContentBlock> {
        let content_futures = self
            .config
            .get_connections()
            .iter()
            .map(|conn| conn.get_immutable(hash));
        join_all(content_futures)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .next()
            .ok_or_else(|| anyhow::anyhow!("Immutable content not found"))
    }

    pub async fn set_immutable(&self, data: ContentBlock) -> Result<blake3::Hash> {
        let encoded = bincode::serialize(&data)?;
        let hash = blake3::hash(&encoded);
        let set_futures = self
            .config
            .get_connections()
            .iter()
            .map(|conn| conn.set_immutable(data.clone()));
        join_all(set_futures).await;
        Ok(hash)
    }
}
