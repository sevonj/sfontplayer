use midi_msg::{ChannelVoiceMsg, Division, Meta, MidiFile, MidiMsg, TimeCodeType, TrackEvent};
use std::{fmt::Display, time::Duration};

/// Ability to receive messages
pub trait MidiSink {
    /// Returns Err if event couldn't be used.
    fn receive_midi(&mut self, msg: &MidiMsg) -> Result<(), ()>;
    fn reset(&mut self);
}

/// [`TrackEvent`] wrapper with some context for debugging.
struct TrackEventWrap {
    pub track_event: TrackEvent,
    pub track_idx: usize,
    pub event_idx: usize,
}
impl Display for TrackEventWrap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let trk = self.track_idx;
        let ev = self.event_idx;
        let event = &self.track_event.event;
        let raw = event.to_midi();
        write!(f, "T{trk}/E{ev} raw: {raw:02X?}, event: {event:?}")
    }
}

/// MIDI Sequencer
pub struct MidiSequencer {
    midifile: Option<MidiFile>,
    bpm: f64,
    /// Index of next event for each track
    track_positions: Vec<usize>,
    /// Song position
    tick: usize,
    since_last_tick: Duration,
    song_len: Duration,
    song_pos: Duration,
}
impl MidiSequencer {
    pub const fn new() -> Self {
        Self {
            midifile: None,
            bpm: 120.,
            track_positions: vec![],
            tick: 0,
            since_last_tick: Duration::ZERO,
            song_len: Duration::ZERO,
            song_pos: Duration::ZERO,
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

        self.update_song_length();
    }

    pub fn update_events<R>(&mut self, event_sink: &mut R, delta_t: Duration)
    where
        R: MidiSink,
    {
        let Some(events) = self.get_events() else {
            return;
        };

        self.song_pos += delta_t;
        self.since_last_tick += delta_t;
        let tick_duration = self.get_current_tick_duration();
        if self.since_last_tick >= tick_duration {
            self.since_last_tick -= tick_duration;
            self.tick += 1;
        }

        for wrap in events {
            match wrap.track_event.event {
                MidiMsg::ChannelVoice { .. }
                | MidiMsg::RunningChannelVoice { .. }
                | MidiMsg::ChannelMode { .. }
                | MidiMsg::RunningChannelMode { .. } => {
                    if event_sink.receive_midi(&wrap.track_event.event).is_err() {
                        println!("Unhandled: {wrap}");
                    }
                }

                midi_msg::MidiMsg::Meta { msg } => self.handle_meta_event(&msg),
                _ => (),
            }
        }
    }

    /// For seeking. Ignore `NoteOn`.
    fn update_events_quiet<R>(&mut self, event_sink: &mut R)
    where
        R: MidiSink,
    {
        let Some(events) = self.get_events() else {
            return;
        };

        self.song_pos += self.get_current_tick_duration();
        self.tick += 1;

        for wrap in events {
            match wrap.track_event.event {
                MidiMsg::ChannelVoice { msg, .. } | MidiMsg::RunningChannelVoice { msg, .. } => {
                    match msg {
                        ChannelVoiceMsg::NoteOn { .. } | ChannelVoiceMsg::HighResNoteOn { .. } => {}
                        _ => {
                            let _ = event_sink.receive_midi(&wrap.track_event.event);
                        }
                    }
                }
                MidiMsg::ChannelMode { .. } | MidiMsg::RunningChannelMode { .. } => {
                    let _ = event_sink.receive_midi(&wrap.track_event.event);
                }
                midi_msg::MidiMsg::Meta { msg } => self.handle_meta_event(&msg),
                _ => (),
            }
        }
    }

    fn get_events(&mut self) -> Option<Vec<TrackEventWrap>> {
        let Some(midifile) = &self.midifile else {
            return None;
        };

        let mut events = vec![];
        for (track_idx, track) in midifile.tracks.iter().enumerate() {
            loop {
                let event_idx = self.track_positions[track_idx];
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
                    events.push(TrackEventWrap {
                        track_event: event.clone(),
                        track_idx,
                        event_idx: self.track_positions[track_idx],
                    });
                    if self.tick > event_tick {
                        let late = self.tick - event_tick;
                        println!("Somehow an event was missed! Playing it late ({late} ticks). {event:?}");
                    }
                    self.track_positions[track_idx] += 1;
                } else {
                    break;
                }
            }
        }
        Some(events)
    }

    fn handle_meta_event(&mut self, msg: &Meta) {
        if let Meta::SetTempo(tempo) = msg {
            self.bpm = 60_000_000. / f64::from(*tempo);
        }
    }

    fn get_current_tick_duration(&self) -> Duration {
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
            self.song_len = Duration::ZERO;
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
        self.song_len = duration;
    }

    pub const fn get_song_length(&self) -> Duration {
        self.song_len
    }

    pub const fn get_song_position(&self) -> Duration {
        self.song_pos
    }

    pub fn seek_to<R>(&mut self, event_sink: &mut R, pos: Duration)
    where
        R: MidiSink,
    {
        let Some(midifile) = &self.midifile else {
            return;
        };

        if pos < self.song_pos {
            self.bpm = 120.;
            self.track_positions = vec![0; midifile.tracks.len()];
            self.tick = 0;
            self.song_pos = Duration::ZERO;
            event_sink.reset();
        }

        self.since_last_tick = Duration::ZERO;

        while self.song_pos < pos {
            self.update_events_quiet(event_sink);
        }
    }
}
