use midi_msg::{ChannelVoiceMsg, Division, Meta, MidiFile, MidiMsg, TimeCodeType, TrackEvent};
use rustysynth::Synthesizer;
use std::time::Duration;

/// MIDI Sequencer
pub struct Sequencer {
    synthesizer: Synthesizer,
    /// Sample duration
    delta_t: Duration,
    midifile: Option<MidiFile>,
    bpm: f64,
    /// Index of next event for each track
    track_positions: Vec<usize>,
    /// Song position
    tick: usize,
    tick_timer: Duration,
    song_length: Duration,
    song_position: Duration,
}
impl Sequencer {
    pub fn new(synthesizer: Synthesizer) -> Self {
        let delta_t = Duration::from_secs_f64(1. / f64::from(synthesizer.get_sample_rate()));
        Self {
            synthesizer,
            delta_t,
            midifile: None,
            bpm: 120.,
            track_positions: vec![],
            tick: 0,
            tick_timer: Duration::ZERO,
            song_length: Duration::ZERO,
            song_position: Duration::ZERO,
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
        self.tick = 0;
        self.track_positions = vec![0; midifile.tracks.len()];
        self.midifile = Some(midifile);

        self.synthesizer.reset();
        self.update_song_length();
    }

    pub fn render(&mut self) -> [f32; 2] {
        self.update_events();

        let mut left = [0.];
        let mut right = [0.];
        self.synthesizer.render(&mut left, &mut right);
        [left[0], right[0]]
    }

    fn update_events(&mut self) {
        let Some(events) = self.get_events() else {
            return;
        };

        self.song_position += self.delta_t;
        self.tick_timer += self.delta_t;
        let tick_duration = self.get_tick_duration();
        if self.tick_timer >= tick_duration {
            self.tick_timer -= tick_duration;
            self.tick += 1;
        }

        for event in events {
            match event.event {
                MidiMsg::ChannelVoice { .. }
                | MidiMsg::RunningChannelVoice { .. }
                | MidiMsg::ChannelMode { .. }
                | MidiMsg::RunningChannelMode { .. } => self.handle_channel_event(&event),
                midi_msg::MidiMsg::Meta { msg } => self.handle_meta_event(&msg),
                _ => (),
            }
        }
    }

    /// For seeking. Ignore `NoteOn`.
    fn update_events_quiet(&mut self) {
        let Some(events) = self.get_events() else {
            return;
        };

        self.song_position += self.delta_t;
        self.tick_timer += self.delta_t;
        let tick_duration = self.get_tick_duration();
        if self.tick_timer >= tick_duration {
            self.tick_timer -= tick_duration;
            self.tick += 1;
        }

        for event in events {
            match event.event {
                MidiMsg::ChannelVoice { msg, .. } | MidiMsg::RunningChannelVoice { msg, .. } => {
                    match msg {
                        ChannelVoiceMsg::NoteOn { .. } | ChannelVoiceMsg::HighResNoteOn { .. } => {}
                        _ => self.handle_channel_event(&event),
                    }
                }
                MidiMsg::ChannelMode { .. } | MidiMsg::RunningChannelMode { .. } => {
                    self.handle_channel_event(&event);
                }
                midi_msg::MidiMsg::Meta { msg } => self.handle_meta_event(&msg),
                _ => (),
            }
        }
    }

    fn get_events(&mut self) -> Option<Vec<TrackEvent>> {
        let Some(midifile) = &self.midifile else {
            return None;
        };

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
                if self.tick >= event_tick {
                    self.track_positions[i] += 1;
                    events.push(event.clone());
                    if self.tick > event_tick {
                        let late = self.tick - event_tick;
                        println!("Somehow an event was missed! Playing it late ({late} ticks). {event:?}");
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
            _ => panic!("This shouldn't happen. Check length before calling."),
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

        if raw.len() == 5 {
            // Break a message that contains MSB and LSB into two separate ones
            if raw[1] == 0x64 {
                let lsb = &raw[0..3];
                let msb = vec![raw[0], raw[3], raw[4]];
                self.send_raw_event(lsb);
                self.send_raw_event(&msb);
                return;
            }
        }

        println!("Unhandled event: raw: {raw:02X?}, event: {event:?}");
    }

    fn handle_meta_event(&mut self, msg: &Meta) {
        if let Meta::SetTempo(tempo) = msg {
            self.bpm = 60_000_000. / f64::from(*tempo);
        }
    }

    fn get_tick_duration(&self) -> Duration {
        let Some(midifile) = &self.midifile else {
            return Duration::ZERO;
        };
        let in_secs = match midifile.header.division {
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
        Duration::from_secs_f64(in_secs)
    }

    fn update_song_length(&mut self) {
        let Some(midifile) = &self.midifile else {
            self.song_length = Duration::ZERO;
            return;
        };

        let mut track_positions = vec![0; midifile.tracks.len()];
        let mut tick = 0;
        let mut duration = Duration::ZERO;
        let mut bpm = 120.;
        loop {
            let mut done = true;
            for (i, track) in midifile.tracks.iter().enumerate() {
                loop {
                    let event_idx = track_positions[i];
                    if event_idx >= track.len() {
                        break;
                    }
                    done = false;

                    let event = &track.events()[event_idx];
                    let event_tick = midifile
                        .header
                        .division
                        .beat_or_frame_to_tick(event.beat_or_frame)
                        as usize;
                    if tick >= event_tick {
                        track_positions[i] += 1;
                        #[allow(clippy::collapsible_match)]
                        if let MidiMsg::Meta { msg } = &event.event {
                            if let Meta::SetTempo(tempo) = msg {
                                bpm = 60_000_000. / f64::from(*tempo);
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
            tick += 1;
            let tick_duration = match midifile.header.division {
                Division::TicksPerQuarterNote(ticks) => 60. / bpm / f64::from(ticks),
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
            duration += Duration::from_secs_f64(tick_duration);
            if done {
                break;
            }
        }
        self.song_length = duration;
    }

    pub const fn get_song_length(&self) -> Duration {
        self.song_length
    }

    pub const fn get_song_position(&self) -> Duration {
        self.song_position
    }

    pub fn get_sample_rate(&self) -> i32 {
        self.synthesizer.get_sample_rate()
    }

    pub fn seek_to(&mut self, t: Duration) {
        let Some(midifile) = &self.midifile else {
            return;
        };

        if t < self.song_position {
            self.bpm = 120.;
            self.track_positions = vec![0; midifile.tracks.len()];
            self.tick = 0;
            self.tick_timer = Duration::ZERO;
            self.song_position = Duration::ZERO;
            self.synthesizer.reset();
        }

        while self.song_position < t {
            self.update_events_quiet();
        }
    }
}
