use crate::{
    configuration::Configuration,
    crypto::{
        encode::{decode_verifying_key, encode_verifying_key},
        CryptoKey,
    },
};
use anyhow::Result;
use ed25519_dalek::VerifyingKey;

use super::SetKeyPayload;

pub struct Actions {
    pub config: Configuration,
}

impl Actions {
    pub fn new(config: Configuration) -> Actions {
        Actions { config }
    }

    pub async fn set(&self, key: String, value: String) -> Result<()> {
        let mut crypto_key = CryptoKey::from_config(&self.config).await;
        let connection = self.config.connection();

        let payload = SetKeyPayload {
            key: key.clone(),
            value: value.clone(),
            priority: 0,
        };
        let signed = crypto_key.sign(payload);

        connection.set(signed).await
    }

    pub async fn get(&self, verifying_key_string: &str, key: &str) -> Result<String> {
        let connection = self.config.connection();
        let verifying_key = decode_verifying_key(verifying_key_string)?;
        connection.get(&verifying_key, key).await
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
