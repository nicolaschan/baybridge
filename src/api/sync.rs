use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{client::Event, crypto::Signed};

#[derive(Encode, Decode, Serialize, Deserialize)]
pub struct SyncEvents {
    pub events: Vec<Signed<Event>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StateHash {
    pub hash: blake3::Hash,
}
