use serde::{Deserialize, Serialize};

use crate::crypto::Signable;

#[derive(Clone, Deserialize, Serialize)]
pub struct SetKeyPayload {
    pub key: String,
    pub value: String,
    pub priority: u128,
}

impl Signable for SetKeyPayload {}

#[derive(Clone, Deserialize, Serialize)]
pub struct DeletionPayload {
    pub name: String,
    pub priority: u128,
}

impl Signable for DeletionPayload {}
