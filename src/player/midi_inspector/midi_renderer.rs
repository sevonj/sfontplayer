use std::{path::PathBuf, sync::Arc};

use midi_msg::MidiFile;
use rodio::Source;
use rustysynth::SoundFont;
use wavers::{Samples, WaversError};

use crate::player::{audio::MidiSource, PlayerError};

/// Renders a MIDI file into an audio file.
#[derive(Debug, Clone)]
pub struct MidiRenderer {
    pub filepath: PathBuf,
    pub sample_rate: i32,
}

impl Default for MidiRenderer {
    fn default() -> Self {
        Self {
            filepath: PathBuf::default(),
            sample_rate: MidiSource::DEFAULT_SAMPLE_RATE,
        }
    }
}

impl MidiRenderer {
    pub(super) fn render(
        &self,
        midi_file: MidiFile,
        soundfont: &Arc<SoundFont>,
    ) -> Result<(), PlayerError> {
        let mut midi_source = MidiSource::with_sample_rate(soundfont, midi_file, self.sample_rate);

        let mut buffer = vec![];
        for sample in midi_source.by_ref() {
            buffer.push(sample);
        }

        let samples: Samples<f32> = Samples::from(buffer);
        wavers::write(
            &self.filepath,
            &samples,
            midi_source.sample_rate(),
            midi_source.channels(),
        )?;
        Ok(())
    }
}

impl From<WaversError> for PlayerError {
    fn from(value: WaversError) -> Self {
        match value {
            WaversError::IoError(error) => error.into(),
            _ => Self::AudioRenderError,
        }
    }
}
