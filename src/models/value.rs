use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;

#[serde_as]
#[derive(Clone, Debug, Encode, Decode, Deserialize, Serialize)]
pub struct Value {
    #[serde_as(as = "Base64")]
    bytes: Vec<u8>,
}

impl Value {
    pub fn new(bytes: Vec<u8>) -> Value {
        Value { bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl From<Vec<u8>> for Value {
    fn from(bytes: Vec<u8>) -> Self {
        Value { bytes }
    }
}
