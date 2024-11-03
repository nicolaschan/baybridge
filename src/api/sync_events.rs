use serde::{Deserialize, Serialize};

use crate::{client::Event, crypto::Signed};

#[derive(Serialize, Deserialize)]
pub struct SyncEvents {
    pub events: Vec<Signed<Event>>,
}
