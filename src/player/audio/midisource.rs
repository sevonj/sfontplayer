use midi_msg::{Division, Meta, MidiFile, MidiMsg, TimeCodeType, TrackEvent};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::{sync::Arc, time::Duration};

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
}

// Rodio requires Iterator implementation.
// This is where whe generate the next samples.
impl Iterator for MidiSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        //println!("next");
        if self.sequencer.end_of_sequence() {
            return None;
        }

        // The midi synth generates bot L and R samples simultaneously, but Rodio polls samples
        // separately for each channel.

        // Left: generate both channels and store R channel sample.
        if self.next_ch == Channel::L {
            self.next_ch = Channel::R;

            //let mut l = [0.; 1];
            //let mut r = [0.; 1];
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
        //let len = match self.sequencer.get_midi_file() {
        //    Some(midifile) => midifile.get_length(),
        //    None => return None,
        //};
        //let pos = self.sequencer.get_position();
        //let remaining = len - pos;
        //let remaining_samples = remaining * f64::from(SAMPLERATE);
        //Some(remaining_samples as usize)
        None
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        SAMPLERATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None
        //self.sequencer
        //    .get_midi_file()
        //    .map(|midifile| Duration::from_secs_f64(midifile.get_length()))
    }
}

/// MIDI Sequencer
struct Sequencer {
    synthesizer: Synthesizer,
    midifile: Option<MidiFile>,
    bpm: f64,
    /// Index of next event for each track
    track_positions: Vec<usize>,
    /// Song position in samples
    position: usize,
}
impl Sequencer {
    pub const fn new(synthesizer: Synthesizer) -> Self {
        Self {
            synthesizer,
            midifile: None,
            bpm: 120.,
            track_positions: vec![],
            position: 0,
        }
    }

    /// Are there no more messages left?
    pub fn end_of_sequence(&self) -> bool {
        let Some(midifile) = &self.midifile else {
            println!("bailed: no midi");
            return true;
        };
        for (i, track) in midifile.tracks.iter().enumerate() {
            if self.track_positions[i] < track.events().len() {
                return false;
            }
        }
        println!("bailed: end reached");
        for (i, track) in midifile.tracks.iter().enumerate() {
            println!(
                "Track {i:02?} - len: {} pos: {}",
                track.events().len(),
                self.track_positions[i]
            );
        }
        true
    }

    pub fn play(&mut self, midifile: MidiFile) {
        self.position = 0;
        self.track_positions = vec![0; midifile.tracks.len()];
        self.midifile = Some(midifile);

        self.synthesizer.reset();
    }

    pub fn render(&mut self) -> [f32; 2] {
        let Some(events) = self.get_events() else {
            return [0., 0.];
        };

        self.position += 1;

        for event in events {
            match event.event {
                MidiMsg::ChannelVoice { .. }
                | MidiMsg::RunningChannelVoice { .. }
                | MidiMsg::ChannelMode { .. }
                | MidiMsg::RunningChannelMode { .. } => self.handle_channel_event(&event),
                midi_msg::MidiMsg::Meta { msg } => self.handle_meta_event(&msg),
                _ => continue,
            }
        }

        let mut left = [0.];
        let mut right = [0.];
        self.synthesizer.render(&mut left, &mut right);
        [left[0], right[0]]
    }

    fn get_events(&mut self) -> Option<Vec<TrackEvent>> {
        let Some(midifile) = &self.midifile else {
            return None;
        };

        let current_tick = self.get_current_tick();

        let mut events = vec![];
        for (i, track) in midifile.tracks.iter().enumerate() {
            loop {
                let event_idx = self.track_positions[i];
                if event_idx >= track.len() {
                    break;
                }
                let event = &track.events()[event_idx];
                let event_tick = midifile
                    .header
                    .division
                    .beat_or_frame_to_tick(event.beat_or_frame)
                    as usize;
                if current_tick >= event_tick {
                    self.track_positions[i] += 1;
                    events.push(event.clone());
                    if current_tick > event_tick {
                        println!("Somehow an event was missed! Playing it late. {event:?}");
                    }
                } else {
                    break;
                }
            }
        }
        Some(events)
    }

    fn send_raw_event(&mut self, raw: &[u8]) {
        let channel = raw[0] & 0x0f;
        let command = raw[0] & 0xf0;
        let data1;
        let data2;
        match raw.len() {
            2 => {
                data1 = raw[1];
                data2 = 0;
            }
            3 => {
                data1 = raw[1];
                data2 = raw[2];
            }
            _ => panic!("This shouldn't happen. Don't call if len > 3."),
        }
        self.synthesizer.process_midi_message(
            channel.into(),
            command.into(),
            data1.into(),
            data2.into(),
        );
    }

    fn handle_channel_event(&mut self, event: &TrackEvent) {
        let raw = event.event.to_midi();

        if let 2..=3 = raw.len() {
            self.send_raw_event(&raw);
            return;
        }

        // Unexpected message length
        println!("Unhandled event: raw: {raw:02X?}, event: {event:?}");
    }

    fn handle_meta_event(&mut self, msg: &Meta) {
        if let Meta::SetTempo(tempo) = msg {
            self.bpm = 60_000_000. / f64::from(*tempo);
        }
    }

    fn get_current_tick(&self) -> usize {
        let Some(midifile) = &self.midifile else {
            return 0;
        };

        // in seconds
        let tick_duration = match midifile.header.division {
            Division::TicksPerQuarterNote(ticks) => 60. / self.bpm / f64::from(ticks),
            Division::TimeCode {
                frames_per_second,
                ticks_per_frame,
            } => {
                let fps = match frames_per_second {
                    TimeCodeType::FPS24 => 24.,
                    TimeCodeType::FPS25 => 25.,
                    TimeCodeType::DF30 | TimeCodeType::NDF30 => 30.,
                };
                1. / fps / f64::from(ticks_per_frame)
            }
        };
        self.position / (tick_duration * f64::from(self.synthesizer.get_sample_rate())) as usize
    }
}
