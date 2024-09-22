use std::{fs::File, path::PathBuf, sync::Arc};

use midisource::MidiSource;
use rodio::{OutputStream, Sink};
use rustysynth::{MidiFile, SoundFont};

mod midisource;

// Load soundfont file.
fn load_soundfont(path: &PathBuf) -> Result<SoundFont, &str> {
    if let Ok(mut file) = File::open(path) {
        return Ok(SoundFont::new(&mut file).unwrap());
    }
    return Err("Failed to open the file!");
}

// Load midi file.
fn load_mididile(path: &PathBuf) -> Result<MidiFile, &str> {
    if let Ok(mut file) = File::open(path) {
        return Ok(MidiFile::new(&mut file).unwrap());
    }
    return Err("Failed to open the file!");
}

///
pub(crate) struct AudioPlayer {
    path_soundfont: Option<PathBuf>,
    path_midifile: Option<PathBuf>,
    stream: OutputStream,
    sink: Sink,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        Self {
            path_soundfont: None,
            path_midifile: None,
            stream,
            sink,
        }
    }
}

impl AudioPlayer {
    pub(crate) fn set_soundfont(&mut self, path: PathBuf) {
        self.path_soundfont = Some(path);
    }

    pub(crate) fn set_midifile(&mut self, path: PathBuf) {
        self.path_midifile = Some(path);
    }

    // Play loaded midi on loaded sf
    pub(crate) fn play(&mut self) -> Result<(), &str> {
        if self.path_soundfont.is_none() {
            return Err("Can't play, no soundfont!");
        }
        if self.path_midifile.is_none() {
            return Err("Can't play, no midi file!");
        }

        let path_sf = self.path_soundfont.as_ref().unwrap();
        let path_mid = self.path_midifile.as_ref().unwrap();
        let source = MidiSource::new(
            Arc::new(load_soundfont(path_sf)?),
            Arc::new(load_mididile(path_mid)?),
        );
        self.sink.append(source);
        self.sink.play();
        Ok(())
    }

    pub(crate) fn stop(&mut self) {
        self.sink.clear();
    }
}
