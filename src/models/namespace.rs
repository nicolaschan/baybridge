use std::collections::HashMap;

use ed25519_dalek::VerifyingKey;

use super::Value;

pub struct NamespaceValues {
    pub namespace: String,
    pub mapping: HashMap<VerifyingKey, Value>,
}
