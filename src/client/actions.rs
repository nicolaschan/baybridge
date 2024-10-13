use crate::{
    configuration::Configuration,
    connectors::http::NamespaceResponse,
    crypto::{
        encode::{decode_verifying_key, encode_verifying_key},
        CryptoKey,
    },
    models::Value,
};
use anyhow::Result;
use ed25519_dalek::VerifyingKey;

use super::{DeletionPayload, SetKeyPayload};

pub struct Actions {
    pub config: Configuration,
}

impl Actions {
    pub fn new(config: Configuration) -> Actions {
        Actions { config }
    }

    async fn set_internal(&self, key: String, value: Value, expires_at: Option<u64>) -> Result<()> {
        let mut crypto_key = CryptoKey::from_config(&self.config).await;
        let connection = self.config.connection();

        let payload = SetKeyPayload {
            key: key,
            value: value,
            priority: 0,
            expires_at: expires_at
        };
        let signed = crypto_key.sign(payload);

        connection.set(signed).await
    }
        
    pub async fn set_with_expires_at(&self, key: String, value: Value, expires_at: u64) -> Result<()> {
        self.set_internal(key, value, Some(expires_at)).await
    }

    pub async fn set(&self, key: String, value: Value) -> Result<()> {
        self.set_internal(key, value, None).await
    }

    pub async fn delete(&self, name: &str) -> Result<()> {
        let mut crypto_key = CryptoKey::from_config(&self.config).await;
        let connection = self.config.connection();

        let payload = DeletionPayload {
            name: name.to_string(),
            priority: 0,
        };
        let signed = crypto_key.sign(payload);

        connection.delete(signed).await
    }

    pub async fn get(&self, verifying_key_string: &str, key: &str) -> Result<Value> {
        let connection = self.config.connection();
        let verifying_key = decode_verifying_key(verifying_key_string)?;
        connection.get(&verifying_key, key).await
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
