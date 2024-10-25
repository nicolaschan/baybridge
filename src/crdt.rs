use crate::{client::Event, crypto::Signed, models::Value};

pub fn merge_events(events: Vec<Signed<Event>>) -> Option<Value> {
    events
        .iter()
        .max_by_key(|event| event.inner.priority())
        .and_then(|event| event.inner.value())
}
