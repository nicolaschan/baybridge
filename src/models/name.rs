use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Name(String);

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Name {
    pub fn new(name: String) -> Name {
        Name(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Name {
    fn from(name: String) -> Self {
        Name(name)
    }
}
