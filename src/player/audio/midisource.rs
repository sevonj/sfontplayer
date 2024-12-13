use super::error::PlayerError;
use midi_msg::{Division, Track};
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::{fs, path::Path, sync::Arc, time::Duration};

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
    sequencer: MidiFileSequencer,
    /// We need to cache the R channel sample.
    cached_sample: f32,
    /// Which channel was played last
    next_ch: Channel,
}

impl MidiSource {
    /// New `MidiSource` that immediately starts playing.
    #[allow(clippy::cast_possible_wrap)] // It's ok to cast here
    pub fn new(sf: &Arc<SoundFont>, midifile: &Arc<MidiFile>) -> Self {
        let settings = SynthesizerSettings::new(SAMPLERATE as i32);
        let mut synthesizer =
            Synthesizer::new(sf, &settings).expect("Could not create synthesizer");
        synthesizer.set_master_volume(1.0);
        let mut sequencer = MidiFileSequencer::new(synthesizer);
        sequencer.play(midifile, false);

        Self {
            sequencer,
            next_ch: Channel::L,
            cached_sample: 0.,
        }
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

            let mut l = [0.; 1];
            let mut r = [0.; 1];
            self.sequencer.render(&mut l, &mut r);
            self.cached_sample = r[0];
            Some(l[0])
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
        let len = match self.sequencer.get_midi_file() {
            Some(midifile) => midifile.get_length(),
            None => return None,
        };
        let pos = self.sequencer.get_position();
        let remaining = len - pos;
        let remaining_samples = remaining * f64::from(SAMPLERATE);
        Some(remaining_samples as usize)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        SAMPLERATE
    }

    fn total_duration(&self) -> Option<Duration> {
        self.sequencer
            .get_midi_file()
            .map(|midifile| Duration::from_secs_f64(midifile.get_length()))
    }
}

/// MIDI Sequencer
pub(crate) struct Sequencer {
    synthesizer: Synthesizer,
    midifile: Option<midi_msg::MidiFile>,
    //division: Division,
    bpm: f64,
    //tracks: Vec<Track>,
    /// Index of next event for each track
    track_positions: Vec<usize>,
    /// Song position in samples
    position: usize,
}
impl Sequencer {
    pub fn new(synthesizer: Synthesizer) -> Result<Self, PlayerError> {
        //
        //let bytes = fs::read(filepath)?;
        //let midifile = midi_msg::MidiFile::from_midi(bytes.as_slice())?;
        //
        //let mut tracks = vec![];
        //let mut track_positions = vec![];
        //for track in midifile.tracks {
        //    if let Track::Midi(_) = track {
        //        tracks.push(track.clone());
        //        track_positions.push(0);
        //    }
        //}

        Ok(Self {
            synthesizer,
            midifile: None,
            //division: midifile.header.division.clone(),
            bpm: 120.,
            //tracks,
            track_positions: vec![],
            position: 0,
        })
    }

    pub fn render(&mut self) -> [f32; 2] {
        let Some(midifile) = &self.midifile else {
            return [0., 0.];
        };

        let tick = self.get_current_tick();

        let mut events = vec![];
        for (i, track) in midifile.tracks.iter().enumerate() {
            loop {
                let event_idx = self.track_positions[i];
                if event_idx >= track.len() {
                    break;
                }
                let event = &track.events()[event_idx];
                if tick
                    >= midifile
                        .header
                        .division
                        .beat_or_frame_to_tick(event.beat_or_frame) as usize
                {
                    self.track_positions[i] += 1;
                    events.push(event.clone());
                }
            }
        }

        self.position += 1;

        todo!()
    }

    fn get_current_tick(&self) -> usize {
        let Some(midifile) = &self.midifile else {
            return 0;
        };

        let samples_per_tick = match midifile.header.division {
            Division::TicksPerQuarterNote(n) => 60. / self.bpm / n as f64,
            Division::TimeCode {
                frames_per_second,
                ticks_per_frame,
            } => {
                let fps = match frames_per_second {
                    midi_msg::TimeCodeType::FPS24 => 24.,
                    midi_msg::TimeCodeType::FPS25 => 25.,
                    midi_msg::TimeCodeType::DF30 => 30.,
                    midi_msg::TimeCodeType::NDF30 => 30.,
                };
                1. / fps / ticks_per_frame as f64
            }
        };

        (samples_per_tick * self.synthesizer.get_sample_rate() as f64) as usize
    }
}
