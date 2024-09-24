//! Audio backend module

use std::{fs::File, path::PathBuf, sync::Arc, time::Duration};

use midisource::MidiSource;
use rodio::{OutputStream, Sink};
use rustysynth::{MidiFile, SoundFont};

mod midisource;

/// Audio backend struct
pub(crate) struct AudioPlayer {
    path_soundfont: Option<PathBuf>,
    path_midifile: Option<PathBuf>,
    midifile_duration: Option<Duration>,

    // We need to keep this alive or the sink goes silent.
    #[allow(dead_code)]
    stream: OutputStream,
    /// Audio sink, controls the output
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
    // --- File Management
    /// Choose new soundfont
    pub(crate) fn set_soundfont(&mut self, path: PathBuf) {
        self.path_soundfont = Some(path);
    }
    /// Choose new midi file
    pub(crate) fn set_midifile(&mut self, path: PathBuf) {
        self.path_midifile = Some(path);
    }

    // --- Playback Control

    /// Unpause
    pub(crate) fn play(&mut self) {
        self.sink.play();
    }
    /// Pause
    pub(crate) fn pause(&mut self) {
        self.sink.pause();
    }
    /// Standard volume range is 0.0..=1.0
    pub(crate) fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
    }
    /// Load currently selected midi & font and start playing
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
    /// Full stop.
    pub(crate) fn stop_playback(&mut self) {
        self.midifile_duration = None;
        self.sink.clear();
        self.sink.pause();
    }

    // --- Playback State

    /// Pause status
    pub(crate) fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
    /// Finished; nothing more to play.
    pub(crate) fn is_empty(&self) -> bool {
        self.sink.empty()
    }
    /// Current midi file duration, if midi file exists
    pub(crate) fn get_midi_length(&self) -> Option<Duration> {
        self.midifile_duration
    }
    /// Playback position. Zero if player is empty.
    pub(crate) fn get_midi_position(&self) -> Duration {
        self.sink.get_pos()
    }
}

// --- Private --- //

/// Private: Load soundfont file.
fn load_soundfont(path: &PathBuf) -> Result<SoundFont, &str> {
    if let Ok(mut file) = File::open(path) {
        return Ok(SoundFont::new(&mut file).unwrap());
    }
    return Err("Failed to open the file!");
}

/// Private: Load midi file.
fn load_midifile(path: &PathBuf) -> Result<MidiFile, &str> {
    if let Ok(mut file) = File::open(path) {
        return Ok(MidiFile::new(&mut file).unwrap());
    }
    return Err("Failed to open the file!");
}
