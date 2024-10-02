//! Audio backend module

use std::{fs::File, path::PathBuf, sync::Arc, time::Duration};

use error::PlayerError;
use midisource::MidiSource;
use rodio::{OutputStream, Sink};
use rustysynth::{MidiFile, SoundFont};

mod error;
mod midisource;

/// Audio backend struct
pub struct AudioPlayer {
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
        let (stream, stream_handle) = OutputStream::try_default().expect("Could not create stream");
        let sink = Sink::try_new(&stream_handle).expect("Could not create sink");
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
    pub(crate) fn play(&self) {
        self.sink.play();
    }
    /// Pause
    pub(crate) fn pause(&self) {
        self.sink.pause();
    }
    /// Standard volume range is 0.0..=1.0
    pub(crate) fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }
    /// Load currently selected midi & font and start playing
    pub(crate) fn start_playback(&mut self) -> anyhow::Result<()> {
        let Some(path_sf) = &self.path_soundfont else {
            anyhow::bail!(PlayerError::NoFont);
        };
        let Some(path_mid) = &self.path_midifile else {
            anyhow::bail!(PlayerError::NoMidi);
        };

        let soundfont = Arc::new(load_soundfont(path_sf)?);
        let midifile = Arc::new(load_midifile(path_mid)?);
        self.midifile_duration = Some(Duration::from_secs_f64(midifile.get_length()));

        let source = MidiSource::new(&soundfont, &midifile);
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

    /// Pause status. Fully stopped should also always be paused.
    pub(crate) fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
    /// Finished; nothing more to play.
    pub(crate) fn is_empty(&self) -> bool {
        self.sink.empty()
    }
    /// Current midi file duration, if midi file exists
    pub const fn get_midi_length(&self) -> Option<Duration> {
        self.midifile_duration
    }
    /// Playback position. Zero if player is empty.
    pub(crate) fn get_midi_position(&self) -> Duration {
        self.sink.get_pos()
    }
}

// --- Private --- //

/// Private: Load soundfont file.
fn load_soundfont(path: &PathBuf) -> anyhow::Result<SoundFont> {
    match File::open(path) {
        Ok(mut file) => match SoundFont::new(&mut file) {
            Ok(soundfont) => Ok(soundfont),
            Err(e) => anyhow::bail!(PlayerError::InvalidFont { source: e }),
        },
        Err(e) => anyhow::bail!(PlayerError::CantAccessFile {
            path: path.clone(),
            source: e,
        }),
    }
}

/// Private: Load midi file.
fn load_midifile(path: &PathBuf) -> anyhow::Result<MidiFile> {
    match File::open(path) {
        Ok(mut file) => match MidiFile::new(&mut file) {
            Ok(midifile) => Ok(midifile),
            Err(e) => anyhow::bail!(PlayerError::InvalidMidi { source: e }),
        },
        Err(e) => anyhow::bail!(PlayerError::CantAccessFile {
            path: path.clone(),
            source: e,
        }),
    }
}
