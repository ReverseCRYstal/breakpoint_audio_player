use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize, Serialize, Default, Clone)]
pub struct Breakpoint {
    hint: String,
    timepoint: Duration,
}

impl From<Duration> for Breakpoint {
    fn from(value: Duration) -> Self {
        Self {
            hint: String::new(),
            timepoint: value,
        }
    }
}

impl From<String> for Breakpoint {
    fn from(value: String) -> Self {
        Self {
            hint: value,
            timepoint: Duration::ZERO,
        }
    }
}

impl Breakpoint {
    pub fn new(timepoint: Duration, hint: String) -> Self {
        Self { hint, timepoint }
    }
    pub fn hint(&self) -> String {
        self.hint.clone()
    }

    pub fn timepoint(&self) -> Duration {
        self.timepoint
    }
}
