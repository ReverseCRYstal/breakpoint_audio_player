//! abstraction of playback function

use rodio::{OutputStream, Sink};
use std::path::Path;
use std::time::Duration;

/// Play an audio with a `Sink`
/// Only one audio can be played and controlled in the playback queue
pub struct SingletonPlayer {
    sink: Sink,
    _stream: OutputStream,
    total_duration: Duration,
    progress: u64,
}

impl Default for SingletonPlayer {
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            _stream: stream,
            sink: Sink::try_new(&stream_handle).unwrap(),
            progress: 0,
            total_duration: Duration::default(),
        }
    }
}

impl SingletonPlayer {
    pub fn try_new(path: &Path) -> Result<Self, String> {
        let mut ret = Self {
            ..Default::default()
        };
        if let Err(result) = ret.play_once(path) {
            Err(result.to_string())
        } else {
            Ok(ret)
        }
    }

    #[inline]
    pub fn switch_to(&self, to_on: bool) {
        if to_on {
            self.resume()
        } else {
            self.pause()
        }
    }

    #[inline]
    pub fn switch(&self) {
        self.switch_to(self.is_paused())
    }
    pub fn get_total_duration(&self) -> Duration {
        self.total_duration
    }
    pub fn get_progress(&self) -> u64 {
        u64::default()
        // unimplemented!()
    }

    pub fn set_progress(&self, value: u64) {
        unimplemented!()
    }

    pub fn set_speed(&self, value: f32) {
        self.sink.set_speed(value);
    }

    pub fn play_once(&mut self, path: &Path) -> Result<(), &str> {
        use rodio::decoder::DecoderError::*;

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
                    self.total_duration = mp3_duration::from_path(&path).unwrap();

                    Ok(())
                }
                Err(error) => match error {
                    UnrecognizedFormat => Err("加载了尚未识别数据的格式。"),
                    IoError(_) => Err("读取、写入或查找流时发生IO错误。"),
                    DecodeError(_) => Err("流包含格式错误的数据，无法解码或解复用。"),
                    LimitError(_) => Err("对流进行解码或解复用时达到了默认或用户定义的限制。限制用于防止来自恶意流的拒绝服务攻击。"),
                    ResetRequired => Err("在继续之前，需要重置解复用器或解码器。"),
                    NoStreams => Err("解码器未找到任何流"),
                }
            }
        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn resume(&self) {
        self.sink.play()
    }

    #[inline]
    pub fn pause(&self) {
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
    pub fn set_volume(&self, value: f32) {
        self.sink.set_volume(value / 100.0);
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    #[inline]
    pub fn _sleep_until_end(&self) {
        self.sink.sleep_until_end();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rodio::Source;
}
