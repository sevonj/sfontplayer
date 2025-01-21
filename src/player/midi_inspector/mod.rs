mod preset_mapper;

use midi_msg::{ChannelVoiceMsg, Header, Meta, MidiFile, MidiMsg, Track};
use rustysynth::SoundFont;
use std::{collections::HashSet, path::PathBuf, sync::Arc};

use crate::player::{playlist::MidiMeta, PlayerError};
pub use preset_mapper::PresetMapper;

pub struct MidiInspectorTrack {
    /// Original track contents
    pub track: Track,
    /// Is this track open in the inspector?
    pub is_open: bool,
    /// Track name, if any
    pub name: Option<String>,
    /// Set of unique program change values in the track
    presets: HashSet<u8>,
}

impl MidiInspectorTrack {
    pub fn new(track: Track) -> Self {
        let mut name = None;
        let mut presets = HashSet::new();

        for trackevent in track.events() {
            match &trackevent.event {
                MidiMsg::ChannelVoice {
                    msg: ChannelVoiceMsg::ProgramChange { program },
                    ..
                } => {
                    presets.insert(*program);
                }
                MidiMsg::Meta {
                    msg: Meta::TrackName(trackname),
                } => {
                    name = Some(trackname.to_owned());
                }
                _ => (),
            }
        }

        Self {
            track,
            is_open: false,
            name,
            presets,
        }
    }

    /// Set of unique program change values in the track
    pub const fn get_presets(&self) -> &HashSet<u8> {
        &self.presets
    }
}

pub struct MidiInspector {
    meta: MidiMeta,
    pub header: Header,
    pub tracks: Vec<MidiInspectorTrack>,
    soundfont: Option<Arc<SoundFont>>,
    /// Set of unique program change values in the file
    presets: HashSet<u8>,
    pub preset_mapper: PresetMapper,
}

impl MidiInspector {
    pub fn new(meta: MidiMeta, soundfont: Option<Arc<SoundFont>>) -> Result<Self, PlayerError> {
        let midi_file = meta.get_midifile()?;

        let header = midi_file.header;
        let mut tracks = vec![];
        for track in midi_file.tracks {
            tracks.push(MidiInspectorTrack::new(track));
        }

        let mut presets = HashSet::new();
        for track in &tracks {
            presets.extend(track.get_presets());
        }

        Ok(Self {
            meta,
            header,
            tracks,
            soundfont,
            presets,
            preset_mapper: PresetMapper::new(),
        })
    }

    /// Filepath of inspected MIDI file
    pub fn get_filepath(&self) -> PathBuf {
        self.meta.get_path()
    }

    /// `MidiMeta` of inspected MIDI file
    pub const fn get_meta(&self) -> &MidiMeta {
        &self.meta
    }

    pub fn get_midi(&self) -> Result<MidiFile, PlayerError> {
        let mut midi_file = self.meta.get_midifile()?;

        self.preset_mapper.remap_midi(&mut midi_file);

        Ok(midi_file)
    }

    pub fn get_soundfont(&self) -> Option<Arc<SoundFont>> {
        self.soundfont.clone()
    }

    pub fn set_soundfont(&mut self, value: Option<Arc<SoundFont>>) {
        self.soundfont = value;
    }

    /// Set of unique program change values in the file
    pub const fn get_presets(&self) -> &HashSet<u8> {
        &self.presets
    }
}
