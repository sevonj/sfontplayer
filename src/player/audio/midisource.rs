use midi_msg::MidiFile;
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::{sync::Arc, time::Duration};

use super::sequencer::Sequencer;

const SAMPLERATE: u32 = 44100;

#[derive(PartialEq)]
enum Channel {
    L,
    R,
}

/// Audio source for Rodio. This takes in soundfont and midifile, and generates audio samples from
/// them. The disposable struct is consumed by audio sink for each song.
pub struct MidiSource {
    /// The actual midi player
    sequencer: Sequencer,
    /// We need to cache the R channel sample.
    cached_sample: f32,
    /// Which channel was played last
    next_ch: Channel,
}

impl MidiSource {
    /// New `MidiSource` that immediately starts playing.
    #[allow(clippy::cast_possible_wrap)] // It's ok to cast here
    pub fn new(sf: &Arc<SoundFont>, midifile: MidiFile) -> Self {
        let settings = SynthesizerSettings::new(SAMPLERATE as i32);
        let mut synthesizer =
            Synthesizer::new(sf, &settings).expect("Could not create synthesizer");
        synthesizer.set_master_volume(1.0);
        let mut sequencer = Sequencer::new(synthesizer);
        sequencer.play(midifile);

        Self {
            sequencer,
            next_ch: Channel::L,
            cached_sample: 0.,
        }
    }

    pub const fn get_song_length(&self) -> Duration {
        self.sequencer.get_song_length()
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

            let samples = self.sequencer.render();
            self.cached_sample = samples[1]; // R
            Some(samples[0]) // L
        }
        // Right: Generate nothing and return cached R ch. sample.
        else {
            self.next_ch = Channel::L;

            Some(self.cached_sample)
        }
    }
}

impl rodio::Source for MidiSource {
    fn current_frame_len(&self) -> Option<usize> {
        let time_left = self.sequencer.get_song_length() - self.sequencer.get_song_position();
        let samples_left = time_left.as_secs_f64() * f64::from(self.sequencer.get_sample_rate());
        Some(samples_left as usize)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        SAMPLERATE
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.sequencer.get_song_length())
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), rodio::source::SeekError> {
        self.sequencer.seek_to(pos);
        Ok(())
    }
}
