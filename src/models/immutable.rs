use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContentBlock {
    pub data: Vec<u8>,
    pub references: Vec<blake3::Hash>,
}
