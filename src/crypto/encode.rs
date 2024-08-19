use anyhow::{anyhow, Result};
use ed25519_dalek::VerifyingKey;

pub fn bytes_to_string(mut bytes: &[u8]) -> Result<String> {
    Ok(ecoji::encode_to_string(&mut bytes)?)
}

pub fn string_to_bytes(encoded: &str) -> Result<Vec<u8>> {
    let mut encoded = encoded.as_bytes();
    Ok(ecoji::decode_to_vec(&mut encoded)?)
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
    bytes_to_string(&verifying_key_bytes).unwrap()
}
