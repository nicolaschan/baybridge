use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Value {
    #[serde_as(as = "Base64")]
    bytes: Vec<u8>,
    pub expires_at: Option<u64>,
}

impl Value {
    pub fn new(bytes: Vec<u8>, expires_at: Option<u64>) -> Value {
        Value { bytes, expires_at }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
