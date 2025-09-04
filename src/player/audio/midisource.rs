use midi_msg::MidiFile;
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::{sync::Arc, time::Duration};

use super::midisequencer::MidiSequencer;

#[derive(PartialEq)]
enum Channel {
    L,
    R,
}

/// Audio source for Rodio. This takes in soundfont and midifile, and generates audio samples from
/// them. The disposable struct is consumed by audio sink for each song.
pub struct MidiSource {
    /// The actual audio generator
    synthesizer: Synthesizer,
    /// The midi file sequencer
    sequencer: MidiSequencer,
    /// Sample time
    delta_t: Duration,
    /// We need to cache the R channel sample.
    cached_sample: f32,
    /// Which channel was played last
    next_ch: Channel,
}

impl MidiSource {
    /// New `MidiSource` that immediately starts playing.
    pub fn new(sf: &Arc<SoundFont>, midi_file: MidiFile) -> Self {
        Self::with_sample_rate(sf, midi_file, Self::DEFAULT_SAMPLE_RATE)
    }

    /// New `MidiSource` that immediately starts playing.
    pub fn with_sample_rate(sf: &Arc<SoundFont>, midifile: MidiFile, sample_rate: i32) -> Self {
        let settings = SynthesizerSettings::new(sample_rate);
        let mut synthesizer =
            Synthesizer::new(sf, &settings).expect("Could not create synthesizer");
        synthesizer.set_master_volume(1.0);
        let mut sequencer = MidiSequencer::new();
        sequencer.play(midifile);

        let delta_t = Duration::from_secs_f64(1. / f64::from(synthesizer.get_sample_rate()));
        Self {
            synthesizer,
            delta_t,
            sequencer,
            next_ch: Channel::L,
            cached_sample: 0.,
        }
    }

    pub const fn song_length(&self) -> Duration {
        self.sequencer.song_length()
    }

    pub const DEFAULT_SAMPLE_RATE: i32 = 44100;

    pub fn sample_rate(&self) -> i32 {
        self.synthesizer.get_sample_rate()
    }
}

// Rodio requires Iterator implementation.
// This is where whe generate the next samples.
impl Iterator for MidiSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sequencer.end_of_sequence() {
            return None;
        }

        // The midi synth generates bot L and R samples simultaneously, but Rodio polls samples
        // separately for each channel.

        // Left: generate both channels and store R channel sample.
        if self.next_ch == Channel::L {
            self.next_ch = Channel::R;

            self.sequencer
                .update_events(&mut self.synthesizer, self.delta_t);

            let mut left = [0.];
            let mut right = [0.];
            self.synthesizer.render(&mut left, &mut right);

            self.cached_sample = right[0] / 10.;
            Some(left[0] / 10.)
        }
        // Right: Generate nothing and return cached R ch. sample.
        else {
            self.next_ch = Channel::L;

            Some(self.cached_sample)
        }
    }
}

impl rodio::Source for MidiSource {
    fn current_span_len(&self) -> Option<usize> {
        let time_left = self.sequencer.song_length() - self.sequencer.song_position();
        let samples_left = time_left.as_secs_f64() * f64::from(self.synthesizer.get_sample_rate());
        Some(samples_left as usize)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        Self::DEFAULT_SAMPLE_RATE as u32
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.sequencer.song_length())
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.sequencer.seek_to(&mut self.synthesizer, pos);
        Ok(())
    }
}
