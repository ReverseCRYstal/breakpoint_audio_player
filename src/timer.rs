use std::time::{Duration, Instant};

/// A simple timer designed for playback record
#[derive(Default)]
pub struct Timer {
    elapsed: Duration,
    updater: Option<Instant>,
}

impl Timer {
    pub fn pause(&mut self) {
        if self.updater.is_none() {
            return;
        }

        self.elapsed += self.updater.unwrap().elapsed();
        self.updater = None;
    }

    pub fn start(&mut self) {
        if self.updater.is_none() {
            self.updater = Some(Instant::now());
        }
    }

    pub fn clear(&mut self) {
        self.pause();
        self.updater = None;
        self.elapsed = Duration::default();
    }

    pub fn read(&self) -> Duration {
        if self.updater.is_none() {
            self.elapsed
        } else {
            self.elapsed + self.updater.unwrap().elapsed()
        }
    }

    pub fn overwrite(&mut self, dur: Duration) {
        self.updater = Some(Instant::now());
        self.elapsed = dur;
    }
}
