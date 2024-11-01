use std::time::UNIX_EPOCH;

use crate::{
    configuration::Configuration,
    connectors::http::NamespaceResponse,
    crdt::merge_events,
    crypto::{
        encode::{decode_verifying_key, encode_verifying_key},
        CryptoKey,
    },
    models::{Name, NamespaceValues, Value},
};
use anyhow::Result;
use ed25519_dalek::VerifyingKey;
use futures::future::join_all;

use super::{DeletionEvent, Event, SetEvent};

pub struct Actions {
    pub config: Configuration,
}

impl Actions {
    pub fn new(config: Configuration) -> Actions {
        Actions { config }
    }

    async fn set_internal(&self, name: Name, value: Value, expires_at: Option<u64>) -> Result<()> {
        let mut crypto_key = CryptoKey::from_config(&self.config).await;

        let event = Event::Set(SetEvent {
            name,
            value,
            priority: UNIX_EPOCH.elapsed().unwrap().as_millis() as u64,
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

    pub async fn set_with_expires_at(
        &self,
        name: Name,
        value: Value,
        expires_at: u64,
    ) -> Result<()> {
        self.set_internal(name, value, Some(expires_at)).await
    }

    pub async fn set(&self, name: Name, value: Value) -> Result<()> {
        self.set_internal(name, value, None).await
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
        let value_mapping = merged_namespace
            .mapping
            .iter()
            .map(|(k, v)| {
                let value = merge_events(v.clone());
                match value {
                    Some(value) => Ok((k.clone(), value)),
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

    pub async fn list(&self) -> Result<Vec<VerifyingKey>> {
        let list_futures = self.config.get_connections().iter().map(|conn| conn.list());
        Ok(join_all(list_futures)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .flatten()
            .collect::<Vec<_>>())
    }

    pub async fn whoami(&self) -> String {
        let crypto_key = CryptoKey::from_config(&self.config).await;
        let verifying_key = crypto_key.verifying();
        encode_verifying_key(&verifying_key)
    }
}
