use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::URL_SAFE};
use ed25519_dalek::VerifyingKey;

pub fn bytes_to_string(bytes: &[u8]) -> String {
    URL_SAFE.encode(bytes)
}

pub fn string_to_bytes(encoded: &str) -> Result<Vec<u8>> {
    URL_SAFE.decode(encoded.as_bytes()).map_err(|e| anyhow!(e))
}

pub fn decode_verifying_key(verifying_key_string: &str) -> Result<VerifyingKey> {
    let bytes = string_to_bytes(verifying_key_string)?;
    let bytes = bytes
        .try_into()
        .map_err(|_e| anyhow!("Expected decoded bytes to be 32 length"))?;
    Ok(VerifyingKey::from_bytes(&bytes)?)
}

pub fn encode_verifying_key(verifying_key: &VerifyingKey) -> String {
    let verifying_key_bytes = verifying_key.to_bytes();
    bytes_to_string(&verifying_key_bytes)
}
