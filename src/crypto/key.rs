use crate::configuration::Configuration;
use anyhow::{anyhow, Context, Result};
use ed25519_dalek::{ed25519::signature::SignerMut, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

use tokio::io::AsyncWriteExt;
use tracing::debug;

use super::{
    encode::{bytes_to_string, string_to_bytes},
    signed::Signable,
    Signed,
};

fn generate_signing_key() -> SigningKey {
    let mut crypto_secure_rng = OsRng;
    SigningKey::generate(&mut crypto_secure_rng)
}

fn decode_signing_key(encoded_string: &str) -> Result<SigningKey> {
    let bytes: [u8; 32] = string_to_bytes(encoded_string)?
        .try_into()
        .map_err(|_e| anyhow!("Expected decoded bytes to be 32 length"))?;
    Ok(SigningKey::from_bytes(&bytes))
}

fn encode_signing_key(signing_key: &SigningKey) -> String {
    let signing_key_bytes = signing_key.to_bytes();
    bytes_to_string(signing_key_bytes.as_slice()).unwrap()
}

pub struct CryptoKey {
    signing_key: SigningKey,
}

impl CryptoKey {
    pub fn verifying(&self) -> VerifyingKey {
        VerifyingKey::from(&self.signing_key)
    }

    pub fn sign<T: Signable>(&mut self, payload: T) -> Signed<T> {
        let serialized = bincode::serialize(&payload).unwrap();
        let signature = self.signing_key.sign(&serialized);
        Signed {
            inner: payload,
            verifying_key: self.verifying(),
            signature,
        }
    }

    async fn generate_new(config: &Configuration) -> Self {
        let signing_key = generate_signing_key();

        debug!(
            "Saving new signing key to {}",
            config.signing_key_path().display()
        );

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&config.signing_key_path())
            .await
            .unwrap();
        let encoded_key = encode_signing_key(&signing_key);
        file.write_all(encoded_key.as_bytes()).await.unwrap();

        Self { signing_key }
    }

    pub async fn from_config(config: &Configuration) -> Self {
        let signing_key_path = config.signing_key_path();
        let existing_key = tokio::fs::read_to_string(&signing_key_path)
            .await
            .context("No existing key file found")
            .and_then(|encoded_key| decode_signing_key(&encoded_key));
        match existing_key {
            Ok(signing_key) => {
                debug!(
                    "Using existing signing key from {}",
                    signing_key_path.display()
                );
                Self { signing_key }
            }
            Err(_) => Self::generate_new(config).await,
        }
    }
}
