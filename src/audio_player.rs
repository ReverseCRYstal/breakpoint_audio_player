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

//! abstraction of playback function

use rodio::{OutputStream, Sink};
use std::path::Path;
use std::time::Duration;

use crate::timer::Timer;

/// Play an audio with a `Sink`
/// Only one audio can be played and controlled in the playback queue
pub struct SingletonPlayer {
    sink: Sink,
    _stream: OutputStream,
    timer: Timer,
}

impl Default for SingletonPlayer {
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            _stream: stream,
            sink: Sink::try_new(&stream_handle).unwrap(),
            timer: Timer::default(),
        }
    }
}

impl SingletonPlayer {
    pub fn try_new(path: &Path) -> Result<Self, String> {
        let mut ret = Self {
            ..Default::default()
        };

        if let Err(result) = ret.play_once(path) {
            Err(result)
        } else {
            Ok(ret)
        }
    }

    #[inline]
    pub fn switch_to_status(&mut self, to_on: bool) {
        if to_on {
            self.resume()
        } else {
            self.pause()
        }
    }

    #[inline]
    pub fn switch_playback_status(&mut self) {
        self.switch_to_status(self.is_paused())
    }

    #[inline]
    pub fn get_progress(&mut self) -> u64 {
        //self.progress = self.timer.read().as_secs();
        //self.progress
        self.timer.read().as_secs()
    }

    pub fn set_progress(&mut self, value: u64) {
        // self.progress = value;
        self.timer.overwrite(Duration::from_secs(value));
        unimplemented!("Actually controls playback.");
    }

    #[inline]
    pub fn set_speed(&self, value: f32) {
        self.sink.set_speed(value);
    }

    pub fn play_once(&mut self, path: &Path) -> Result<(), String> {
        if self.sink.empty() {
            self.pause();
        } else {
            self.sink.clear();
        }

        if !path.to_str().unwrap().is_empty() {
            let file = std::io::BufReader::new(std::fs::File::open(path).unwrap());

            match rodio::Decoder::new(file) {
                Ok(source) => {
                    self.sink.append(source);
                    self.timer.clear();

                    Ok(())
                }
                Err(error) => Err(error.to_string()),
            }
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn resume(&mut self) {
        self.timer.start();
        self.sink.play();
    }

    #[inline]
    pub fn pause(&mut self) {
        self.timer.pause();
        self.sink.pause();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    /// Sets volume by percentage. \
    /// The value `100.0` is the 'normal' volume.\
    /// See `Sink::set_volume for details.
    #[inline]
    pub fn set_volume(&self, value: u8) {
        self.sink.set_volume(value as f32 / 100.0);
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
}
