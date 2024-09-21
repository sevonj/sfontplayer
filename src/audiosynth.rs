use std::{fs::File, path::PathBuf, sync::Arc};

use rodio::Sink;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

const SAMPLERATE: u32 = 44100;

#[derive(Default)]
pub(crate) struct AudioSynth {
    soundfont: Option<Arc<SoundFont>>,
    midifile: Option<Arc<MidiFile>>,
    sink: Sink,
}
impl AudioSynth {
    // Load soundfont from file.
    pub(crate) fn load_soundfont(&mut self, path: &PathBuf) -> Result<(), &str> {
        if let Ok(mut file) = File::open(path) {
            self.soundfont = Some(Arc::new(SoundFont::new(&mut file).unwrap()));
            return Ok(());
        }
        return Err("Failed to open the file!");
    }

    pub(crate) fn play_midi(&mut self, path: &PathBuf) -> Result<(), &str> {
        if self.soundfont.is_none() {
            return Err("Can't play file no soundfont!");
        }

        // Load the MIDI file.
        let mut mid = File::open(path).unwrap();
        self.midifile = Some(Arc::new(MidiFile::new(&mut mid).unwrap()));
        

        Ok(())
    }
}

pub struct MidiSource {
    sequencer: MidiFileSequencer,
    last_ch_was_r: bool,
    cached_sample: f32,
}

impl MidiSource {
    pub fn new(sf: Arc<SoundFont>, midifile: Arc<MidiFile>) -> Self {
        let settings = SynthesizerSettings::new(SAMPLERATE as i32);
        let synthesizer = Synthesizer::new(&sf, &settings).unwrap();
        let mut sequencer = MidiFileSequencer::new(synthesizer);

        // Play the MIDI file.
        sequencer.play(&midifile, false);

        Self {
            sequencer,
            last_ch_was_r: true,
            cached_sample: 0.,
        }
    }
}

impl Iterator for MidiSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sequencer.end_of_sequence() {
            return None;
        }
        if self.last_ch_was_r {
            let mut l = [0.0; 1];
            let mut r = [0.0; 1];
            self.sequencer.render(&mut l, &mut r);
            self.cached_sample = r[0];
            self.last_ch_was_r = false;
            Some(l[0])
        } else {
            self.last_ch_was_r = true;
            Some(self.cached_sample)
        }
    }
}

impl rodio::Source for MidiSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        SAMPLERATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
