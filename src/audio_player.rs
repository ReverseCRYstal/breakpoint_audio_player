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
    pub fn replace_file(&mut self, mut reader: BufReader<File>) -> Result<(), anyhow::Error> {
        if self.sink.empty() {
            self.pause();
        } else {
            self.sink.clear();
        };

        self.total_duration = Some(mp3_duration::from_read(reader.get_mut())?);

        rodio::Decoder::new(reader).map(|decoder| {
            let buffered = decoder.buffered();

            self.src = Some(buffered.clone());
            self.sink.append(buffered);
            self.timer.clear();
        })?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.total_duration = None;
        self.sink.clear();
        self.src = None;
    }
}

impl SingletonPlayer {
    #[inline(always)]
    pub fn get_progress(&self) -> Duration {
        self.timer.read()
    }

    pub fn set_progress(&mut self, value: Duration) {
        if !self.is_empty() {
            self.timer.overwrite(value);

            let paused = self.is_paused();

            dbg!(self.is_empty());
            self.sink.skip_one();

            dbg!(self.is_empty());
            self.sink.append(self.src.clone().unwrap());

            dbg!(self.is_empty());
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
    pub fn reset(&self) {
        self.sink.clear();
        self.sink.append(self.src.clone().unwrap());
    }

    #[inline]
    pub fn total_duration(&self) -> Option<Duration> {
        self.total_duration
    }
}
