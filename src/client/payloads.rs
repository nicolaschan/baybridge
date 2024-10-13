use serde::{Deserialize, Serialize};

use crate::{crypto::Signable, models::Value};

#[derive(Clone, Deserialize, Serialize)]
pub struct SetKeyPayload {
    pub key: String,
    pub value: Value,
    pub priority: u128,
    pub expires_at: Option<u64>
}

impl Signable for SetKeyPayload {}

#[derive(Clone, Deserialize, Serialize)]
pub struct DeletionPayload {
    pub name: String,
    pub priority: u128,
}

impl Signable for DeletionPayload {}
