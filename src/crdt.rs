use crate::{client::Event, crypto::Signed, models::Value};

pub fn merge_events(events: Vec<Signed<Event>>) -> Option<Value> {
    events.last().and_then(|event| match event.inner {
        Event::Set(ref event) => Some(event.value.clone()),
        Event::Delete(_) => None,
    })
}
