use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::{sync::Arc, time::Duration};

const SAMPLERATE: u32 = 44100;

/// Audio source for Rodio. This takes in soundfont and midifile, and generates audio samples from
/// them. The disposable struct is consumed by audio sink for each song.
pub struct BufferedMidiSource {
    /// Buffer contains samples alternating between L and R channels.
    buffer_r: Vec<f32>,
    buffer_l: Vec<f32>,
    /// position in buffer (each channel sample counts)
    position: usize,
}

impl BufferedMidiSource {
    /// New MidiSource that immediately starts playing.
    pub fn new(sf: Arc<SoundFont>, midifile: Arc<MidiFile>) -> Self {
        // Create midi synth
        let settings = SynthesizerSettings::new(SAMPLERATE as i32);
        let synthesizer = Synthesizer::new(&sf, &settings).unwrap();
        let mut sequencer = MidiFileSequencer::new(synthesizer);
        sequencer.play(&midifile, false);

        // Render the entire contents to a buffer
        let len = (midifile.get_length() * SAMPLERATE as f64) as usize;
        let mut buffer_l = vec![0.; len];
        let mut buffer_r = vec![0.; len];
        sequencer.render(&mut buffer_l, &mut buffer_r);

        // Return self
        Self {
            position: 0,
            buffer_r,
            buffer_l,
        }
    }
}

// Rodio requires Iterator implementation.
// This is where whe generate the next samples.
impl Iterator for BufferedMidiSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.buffer_r.len() * 2 {
            return None;
        }

        let pos = self.position / 2;
        self.position += 1;

        // Rodio polls different channels separately.
        // Alternate L and R channels. L first.
        if pos % 2 == 1 {
            return Some(self.buffer_r[pos]);
        } else {
            return Some(self.buffer_l[pos]);
        }
    }
}

impl rodio::Source for BufferedMidiSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.position)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        SAMPLERATE
    }

    fn total_duration(&self) -> Option<Duration> {
        let num_samples = self.buffer_r.len() * 2;
        Some(Duration::from_secs_f64(num_samples as f64 / SAMPLERATE as f64))
    }
}
