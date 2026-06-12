//! BGM and sound effects through rodio. Fails soft: if no output device
//! is available the game runs silent.

use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use std::path::Path;
use std::sync::Arc;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

pub struct Audio {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    bgm: Option<Sink>,
    bgm_name: Option<String>,
    sfx: HashMap<String, Arc<[u8]>>,
}

impl Audio {
    pub fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        Some(Self { _stream: stream, handle, bgm: None, bgm_name: None, sfx: HashMap::new() })
    }

    pub fn register_sfx(&mut self, name: &str, wav: Vec<u8>) {
        self.sfx.insert(name.to_string(), wav.into());
    }

    pub fn play_sfx(&self, name: &str) {
        if let Some(data) = self.sfx.get(name) {
            let cursor = Cursor::new(data.clone());
            if let Ok(source) = Decoder::new(cursor) {
                let _ = self.handle.play_raw(source.convert_samples());
            }
        }
    }

    /// Stream a BGM wav from disk on infinite loop. No-op when the track
    /// is already playing.
    pub fn play_bgm(&mut self, path: &Path) {
        let name = path.to_string_lossy().into_owned();
        if self.bgm_name.as_deref() == Some(&name) {
            return;
        }
        if let Some(old) = self.bgm.take() {
            old.stop();
        }
        let Ok(file) = std::fs::File::open(path) else { return };
        let Ok(source) = Decoder::new(BufReader::new(file)) else { return };
        let Ok(sink) = Sink::try_new(&self.handle) else { return };
        sink.set_volume(0.6);
        sink.append(source.repeat_infinite());
        self.bgm = Some(sink);
        self.bgm_name = Some(name);
    }
}
