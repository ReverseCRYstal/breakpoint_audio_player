#![allow(dead_code)]

use rodio::{OutputStream, Sink};

pub struct AudioPlayer {
    sink: Sink,
    _stream: OutputStream,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            _stream: stream,
            sink: Sink::try_new(&stream_handle).unwrap(),
        }
    }
}

impl AudioPlayer {
    pub fn from_path(path: &str) -> Self {
        let ret = Self {
            ..Default::default()
        };
        ret.play_single_file(path);
        ret
    }
    pub fn switch_to(&self, to_on: bool) {
        if to_on {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    pub fn switch(&self) {
        self.switch_to(self.sink.is_paused())
    }

    pub fn play_single_file(&self, path: &str) {
        let file = std::io::BufReader::new(std::fs::File::open(path).unwrap());

        let source = rodio::Decoder::new(file).unwrap();
        self.sink.pause();
        self.sink.append(source);
    }

    pub fn resume(&self) {
        self.sink.play()
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn set_volumn(&self, value: f32) {
        self.sink.set_volume(value);
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn sleep_until_end(&self) {
        self.sink.sleep_until_end();
    }
}

#[test]
fn play_control_test() {
    let path = ".\\assests\\example_audio.mp3";

    let player = AudioPlayer::default();
    player.play_single_file(path);
    player.sleep_until_end();
}
