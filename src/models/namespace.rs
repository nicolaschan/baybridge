use std::collections::HashMap;

use super::Value;

pub struct NamespaceValues {
    pub namespace: String,
    pub mapping: HashMap<String, Value>,
}
