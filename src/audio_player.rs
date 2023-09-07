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

//use rodio::decoder::DecoderError;
use rodio::{source::Buffered, Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
//use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::timer::Timer;

/// Play an audio with a `Sink`
/// Only one audio can be played and controlled in the playback queue
pub struct SingletonPlayer {
    sink: Sink,
    src: Option<Buffered<Decoder<BufReader<File>>>>,
    _stream: OutputStream,
    timer: Timer,
    total_duration: Option<Duration>,
}

impl Default for SingletonPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl SingletonPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            src: None,
            total_duration: None,
            _stream: stream,
            sink: Sink::try_new(&stream_handle).unwrap(),
            timer: Timer::default(),
        }
    }
}

impl SingletonPlayer {
    pub fn replace_file(&mut self, reader: BufReader<File>) -> Result<(), anyhow::Error> {
        if self.sink.empty() {
            self.pause();
        } else {
            self.sink.clear();
        };

        rodio::Decoder::new(reader).map(|decoder| {
            let buffered = decoder.buffered();

            self.src = Some(buffered.clone());
            self.sink.append(buffered);
            self.timer.clear();
        })?;
        // mp3_duration::from_read(reader);
        Ok(())
    }
}

impl SingletonPlayer {
    #[inline(always)]
    pub fn get_progress(&mut self) -> u64 {
        self.timer.read().as_secs()
    }

    pub fn set_progress(&mut self, value: u64) {
        if self.is_empty() {
            // self.progress = value;
            self.timer.overwrite(Duration::from_secs(value));

            let paused = self.is_paused();

            self.sink.skip_one();
            self.sink.append(unsafe {
                self.src
                    .clone()
                    .unwrap_unchecked()
                    .skip_duration(Duration::from_secs(value))
            });

            if paused {
                self.pause();
            }
        }
    }

    #[inline(always)]
    pub fn set_speed(&self, value: f32) {
        self.sink.set_speed(value);
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

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    /// 注意 `value` 是百分比音量
    #[inline]
    pub fn set_volume(&self, value: u8) {
        self.sink.set_volume(value as f32 / 100.0);
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    #[inline]
    pub fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }
}
