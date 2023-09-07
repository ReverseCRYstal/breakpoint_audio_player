use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize)]
pub struct Breakpoint {
    hint: String,
    anchored_at: Duration,
}

impl From<Duration> for Breakpoint {
    fn from(value: Duration) -> Self {
        Self {
            hint: String::new(),
            anchored_at: value,
        }
    }
}

impl Breakpoint {
    fn new(hint: String, anchored_at: Duration) -> Self {
        Self { hint, anchored_at }
    }
}
