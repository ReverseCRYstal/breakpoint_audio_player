// MIT License
//
// Copyright (c) 2023 CrYStaL
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::time::{Duration, Instant};

/// A simple timer designed for playback record
#[derive(Default)]
pub struct Timer {
    total_time: Duration,
    updater: Option<Instant>,
}

impl Timer {
    pub fn pause(&mut self) {
        if self.updater.is_none() {
            return;
        }

        self.total_time += self.updater.unwrap().elapsed();
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
        self.total_time = Duration::default();
    }

    pub fn read(&self) -> Duration {
        if self.updater.is_none() {
            self.total_time
        } else {
            self.total_time + self.updater.unwrap().elapsed()
        }
    }

    pub fn overwrite(&mut self, dur: Duration) {
        self.updater = Some(Instant::now());
        self.total_time = dur;
    }
}
