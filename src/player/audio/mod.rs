//! Audio backend module

mod midisequencer;
mod midisource;
mod midisynth;

use midi_msg::MidiFile;
use rodio::Sink;
use rustysynth::SoundFont;
use std::{sync::Arc, time::Duration};

use super::PlayerError;
pub use midisource::MidiSource;

/// Audio backend struct
#[derive(Default)]
pub struct AudioPlayer {
    soundfont: Option<Arc<SoundFont>>,
    midifile: Option<MidiFile>,
    midifile_duration: Option<Duration>,
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
    pub(crate) fn set_soundfont(&mut self, soundfont: Arc<SoundFont>) {
        self.soundfont = Some(soundfont);

        if let Some(sink) = &self.sink {
            if !sink.empty() {
                let pos = sink.get_pos();
                sink.clear();
                let _ = self.start_playback();
                let _ = self.seek_to(pos);
            }
        };
    }

    pub(crate) fn clear_soundfont(&mut self) {
        self.soundfont = None;
    }

    /// Choose new midi file
    pub(crate) fn set_midifile(&mut self, midifile: MidiFile) {
        self.midifile = Some(midifile);
    }

    // --- Playback Control

    /// Unpause
    pub(crate) fn play(&self) -> Result<(), PlayerError> {
        let Some(sink) = &self.sink else {
            return Err(PlayerError::AudioNoSink);
        };
        sink.play();
        Ok(())
    }

    /// Pause
    pub(crate) fn pause(&self) -> Result<(), PlayerError> {
        let Some(sink) = &self.sink else {
            return Err(PlayerError::AudioNoSink);
        };
        sink.pause();
        Ok(())
    }

    /// Standard volume range is 0.0..=1.0
    pub(crate) fn set_volume(&self, volume: f32) -> Result<(), PlayerError> {
        let Some(sink) = &self.sink else {
            return Err(PlayerError::AudioNoSink);
        };
        sink.set_volume(volume);
        Ok(())
    }

    /// Load currently selected midi & font and start playing
    pub(crate) fn start_playback(&mut self) -> Result<(), PlayerError> {
        let Some(soundfont) = &self.soundfont else {
            return Err(PlayerError::AudioNoFont);
        };
        let Some(midifile) = self.midifile.clone() else {
            return Err(PlayerError::AudioNoMidi);
        };
        let Some(sink) = &self.sink else {
            return Err(PlayerError::AudioNoSink);
        };
        let source = MidiSource::new(soundfont, midifile);
        self.midifile_duration = Some(source.song_length());

        sink.append(source);
        sink.play();
        Ok(())
    }

    /// Full stop.
    pub(crate) fn stop_playback(&mut self) -> Result<(), PlayerError> {
        let Some(sink) = &self.sink else {
            return Err(PlayerError::AudioNoSink);
        };
        self.midifile_duration = None;
        sink.clear();
        sink.pause();
        Ok(())
    }

    pub(crate) fn seek_to(&self, pos: Duration) -> Result<(), PlayerError> {
        let Some(sink) = &self.sink else {
            return Err(PlayerError::AudioNoSink);
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
