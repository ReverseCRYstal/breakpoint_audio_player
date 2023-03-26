use rodio::{OutputStream, Sink};

pub struct AudioPlayer {
    sink: Sink,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self {
            sink: Sink::try_new(&OutputStream::try_default().unwrap().1).unwrap(),
        }
    }
}

impl AudioPlayer {
    pub fn switch_to(&self, to_on: bool) {
        if to_on {
            self.sink.play()
        } else {
            self.sink.pause()
        }
    }

    pub fn switch(&self) {
        self.switch_to(!self.sink.is_paused())
    }

    pub fn play(&self, path: String) {
        let file = std::io::BufReader::new(std::fs::File::open(path.clone()).unwrap());
        
        let source = rodio::Decoder::new(file).unwrap();
        
        self.sink.clear();
        self.sink.append(source);
    }

    pub fn pause(&self) {
        self.sink.pause();
    }
}
