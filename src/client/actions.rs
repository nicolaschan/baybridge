use crate::{
    configuration::Configuration,
    connectors::http::NamespaceResponse,
    crdt::merge_events,
    crypto::{
        encode::{decode_verifying_key, encode_verifying_key},
        CryptoKey,
    },
    models::{Name, Value},
};
use anyhow::Result;
use ed25519_dalek::VerifyingKey;

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
        let connection = self.config.connection();

        let event = Event::Set(SetEvent {
            name,
            value,
            priority: 0,
            expires_at,
        });
        let signed = crypto_key.sign(event);

        connection.set(signed).await
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
        let connection = self.config.connection();

        let event = Event::Delete(DeletionEvent { name, priority: 0 });
        let signed = crypto_key.sign(event);

        connection.set(signed).await
    }

    pub async fn get(&self, verifying_key_string: &str, name: &Name) -> Result<Value> {
        let connection = self.config.connection();
        let verifying_key = decode_verifying_key(verifying_key_string)?;
        let relevant_events = connection.get(&verifying_key, name).await?;
        let value = merge_events(relevant_events.events);
        match value {
            Some(value) => Ok(value),
            None => Err(anyhow::anyhow!("Value not found")),
        }
    }

    pub async fn namespace(&self, name: &str) -> Result<NamespaceResponse> {
        let connection = self.config.connection();
        connection.namespace(name).await
    }

    pub async fn list(&self) -> Result<Vec<VerifyingKey>> {
        let connection = self.config.connection();
        connection.list().await
    }

    pub async fn whoami(&self) -> String {
        let crypto_key = CryptoKey::from_config(&self.config).await;
        let verifying_key = crypto_key.verifying();
        encode_verifying_key(&verifying_key)
    }
}
