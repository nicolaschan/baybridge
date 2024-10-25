use serde::{Deserialize, Serialize};

use crate::{
    crypto::{Signable, Signed},
    models::{Name, Value},
};

#[derive(Clone, Deserialize, Serialize)]
pub struct SetEvent {
    pub name: Name,
    pub value: Value,
    pub priority: u64,
    pub expires_at: Option<u64>,
}

impl Signable for SetEvent {}

#[derive(Clone, Deserialize, Serialize)]
pub struct DeletionEvent {
    pub name: Name,
    pub priority: u64,
}

impl Signable for DeletionEvent {}

#[derive(Clone, Deserialize, Serialize)]
pub enum Event {
    Set(SetEvent),
    Delete(DeletionEvent),
}

impl Signable for Event {}

impl Event {
    pub fn name(&self) -> &Name {
        match self {
            Event::Set(event) => &event.name,
            Event::Delete(event) => &event.name,
        }
    }

    pub fn priority(&self) -> u64 {
        match self {
            Event::Set(event) => event.priority,
            Event::Delete(event) => event.priority,
        }
    }

    pub fn expires_at(&self) -> Option<u64> {
        match self {
            Event::Set(event) => event.expires_at,
            Event::Delete(_) => None,
        }
    }

    pub fn value(&self) -> Option<Value> {
        match self {
            Event::Set(event) => Some(event.value.clone()),
            Event::Delete(_) => None,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RelevantEvents {
    pub events: Vec<Signed<Event>>,
}
