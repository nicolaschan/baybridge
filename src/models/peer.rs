use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Peers {
    pub peers: Vec<String>,
}
