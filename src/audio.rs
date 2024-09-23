use std::{fs::File, path::PathBuf, sync::Arc, time::Duration};

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
fn load_midifile(path: &PathBuf) -> Result<MidiFile, &str> {
    if let Ok(mut file) = File::open(path) {
        return Ok(MidiFile::new(&mut file).unwrap());
    }
    return Err("Failed to open the file!");
}

///
pub(crate) struct AudioPlayer {
    path_soundfont: Option<PathBuf>,
    path_midifile: Option<PathBuf>,
    midifile_duration: Option<Duration>,
    stream: OutputStream,
    sink: Sink,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.pause();
        Self {
            path_soundfont: None,
            path_midifile: None,
            midifile_duration: None,
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
    pub(crate) fn play(&mut self) {
        self.sink.play();
    }
    pub(crate) fn pause(&mut self) {
        self.sink.pause();
    }
    pub(crate) fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.sink.empty()
    }
    pub(crate) fn end_reached(&self) -> bool {
        if let Some(len) = self.get_midi_length() {
            return self.get_midi_position() >= len;
        }
        false
    }
    pub(crate) fn get_midi_length(&self) -> Option<Duration> {
        self.midifile_duration
    }
    pub(crate) fn get_midi_position(&self) -> Duration {
        self.sink.get_pos()
    }

    // Play loaded midi on loaded sf
    pub(crate) fn start_playback(&mut self) -> Result<(), &str> {
        if self.path_soundfont.is_none() {
            return Err("Can't play, no soundfont!");
        }
        if self.path_midifile.is_none() {
            return Err("Can't play, no midi file!");
        }

        let path_sf = self.path_soundfont.as_ref().unwrap();
        let path_mid = self.path_midifile.as_ref().unwrap();
        let midifile = load_midifile(path_mid)?;
        self.midifile_duration = Some(Duration::from_secs_f64(midifile.get_length()));

        let source = MidiSource::new(Arc::new(load_soundfont(path_sf)?), Arc::new(midifile));
        self.sink.append(source);
        self.sink.play();
        Ok(())
    }

    pub(crate) fn stop_playback(&mut self) {
        self.midifile_duration = None;
        self.sink.clear();
    }
}
