use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Encode, Decode, Deserialize, Serialize)]
pub struct ContentBlock {
    pub data: Vec<u8>,
    #[bincode(with_serde)]
    pub references: Vec<blake3::Hash>,
}
