//! Audio backend module

use std::{
    fs::{self, File},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use error::PlayerError;
use midi_msg::MidiFile;
use midisource::MidiSource;
use rodio::Sink;
use rustysynth::SoundFont;

mod error;
mod midisource;
mod sequencer;

/// Audio backend struct
#[derive(Default)]
pub struct AudioPlayer {
    path_soundfont: Option<PathBuf>,
    path_midifile: Option<PathBuf>,
    midifile_duration: Option<Duration>,

    // We need to keep this alive or the sink goes silent.
    //#[allow(dead_code)]
    //stream: OutputStream,
    /// Audio sink, controls the output
    sink: Option<Sink>,
}

impl AudioPlayer {
    pub(crate) fn set_sink(&mut self, value: Option<Sink>) {
        if let Some(ref sink) = value {
            sink.pause();
        }
        self.sink = value;
    }

    // --- File Management

    /// Choose new soundfont
    pub(crate) fn set_soundfont(&mut self, path: PathBuf) {
        self.path_soundfont = Some(path);

        if let Some(sink) = &self.sink {
            if !sink.empty() {
                let pos = sink.get_pos();
                sink.clear();
                let _ = self.start_playback();
                let _ = self.seek_to(pos);
            }
        };
    }
    /// Choose new midi file
    pub(crate) fn set_midifile(&mut self, path: PathBuf) {
        self.path_midifile = Some(path);
    }

    // --- Playback Control

    /// Unpause
    pub(crate) fn play(&self) -> anyhow::Result<()> {
        let Some(sink) = &self.sink else {
            anyhow::bail!(PlayerError::NoSink);
        };
        sink.play();
        Ok(())
    }
    /// Pause
    pub(crate) fn pause(&self) -> anyhow::Result<()> {
        let Some(sink) = &self.sink else {
            anyhow::bail!(PlayerError::NoSink);
        };
        sink.pause();
        Ok(())
    }
    /// Standard volume range is 0.0..=1.0
    pub(crate) fn set_volume(&self, volume: f32) -> anyhow::Result<()> {
        let Some(sink) = &self.sink else {
            anyhow::bail!(PlayerError::NoSink);
        };
        sink.set_volume(volume);
        Ok(())
    }
    /// Load currently selected midi & font and start playing
    pub(crate) fn start_playback(&mut self) -> anyhow::Result<()> {
        let Some(path_sf) = &self.path_soundfont else {
            anyhow::bail!(PlayerError::NoFont);
        };
        let Some(path_mid) = &self.path_midifile else {
            anyhow::bail!(PlayerError::NoMidi);
        };
        let Some(sink) = &self.sink else {
            anyhow::bail!(PlayerError::NoSink);
        };

        let soundfont = Arc::new(load_soundfont(path_sf)?);
        let midifile = load_midifile(path_mid)?;

        let source = MidiSource::new(&soundfont, midifile);
        self.midifile_duration = Some(source.get_song_length());

        sink.append(source);
        sink.play();
        Ok(())
    }
    /// Full stop.
    pub(crate) fn stop_playback(&mut self) -> anyhow::Result<()> {
        let Some(sink) = &self.sink else {
            anyhow::bail!(PlayerError::NoSink);
        };
        self.midifile_duration = None;
        sink.clear();
        sink.pause();
        Ok(())
    }
    pub(crate) fn seek_to(&self, pos: Duration) -> anyhow::Result<()> {
        let Some(sink) = &self.sink else {
            anyhow::bail!(PlayerError::NoSink);
        };
        let _ = sink.try_seek(pos);
        Ok(())
    }

    // --- Playback State

    /// Pause status. Fully stopped should also always be paused.
    pub(crate) fn is_paused(&self) -> bool {
        let Some(sink) = &self.sink else {
            return true;
        };
        sink.is_paused()
    }
    /// Finished; nothing more to play.
    pub(crate) fn is_empty(&self) -> bool {
        let Some(sink) = &self.sink else {
            return true;
        };
        sink.empty()
    }
    /// Current midi file duration, if midi file exists
    pub const fn get_midi_length(&self) -> Option<Duration> {
        self.midifile_duration
    }
    /// Playback position. Zero if player is empty.
    pub(crate) fn get_midi_position(&self) -> Duration {
        let Some(sink) = &self.sink else {
            return Duration::ZERO;
        };
        sink.get_pos()
    }
}

// --- Private --- //

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

fn load_midifile(filepath: &PathBuf) -> anyhow::Result<MidiFile> {
    let bytes = fs::read(filepath)?;
    Ok(midi_msg::MidiFile::from_midi(bytes.as_slice())?)
}
