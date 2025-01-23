mod midi_renderer;
mod preset_mapper;

use midi_msg::{ChannelVoiceMsg, Header, Meta, MidiFile, MidiMsg, Track};
use midi_renderer::MidiRenderer;
use rustysynth::SoundFont;
use std::{collections::HashSet, path::PathBuf, sync::Arc};

use crate::player::{playlist::MidiMeta, PlayerError};
pub use preset_mapper::PresetMapper;

#[derive(Debug)]
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
    pub const fn presets(&self) -> &HashSet<u8> {
        &self.presets
    }
}

#[derive(Debug)]
pub struct MidiInspector {
    midimeta: MidiMeta,
    pub header: Header,
    pub tracks: Vec<MidiInspectorTrack>,
    soundfont: Option<Arc<SoundFont>>,
    /// Set of unique program change values in the file
    presets: HashSet<u8>,
    pub preset_mapper: PresetMapper,
    pub midi_renderer: MidiRenderer,
}

impl MidiInspector {
    pub fn new(midimeta: MidiMeta, soundfont: Option<Arc<SoundFont>>) -> Result<Self, PlayerError> {
        let midi_file = midimeta.fetch_midifile()?;

        let header = midi_file.header;
        let mut tracks = vec![];
        for track in midi_file.tracks {
            tracks.push(MidiInspectorTrack::new(track));
        }

        let mut presets = HashSet::new();
        for track in &tracks {
            presets.extend(track.presets());
        }

        Ok(Self {
            midimeta,
            header,
            tracks,
            soundfont,
            presets,
            preset_mapper: PresetMapper::default(),
            midi_renderer: MidiRenderer::default(),
        })
    }

    /// Filepath of inspected MIDI file
    pub const fn midi_filepath(&self) -> &PathBuf {
        self.midimeta.filepath()
    }

    /// `MidiMeta` of inspected MIDI file
    pub const fn midimeta(&self) -> &MidiMeta {
        &self.midimeta
    }

    /// Get the inspected midi file, with potential changes made by the user.
    pub fn midifile(&self) -> Result<MidiFile, PlayerError> {
        let mut midifile = self.midimeta.fetch_midifile()?;
        self.preset_mapper.remap_midi(&mut midifile);
        Ok(midifile)
    }

    pub fn soundfont(&self) -> Option<Arc<SoundFont>> {
        self.soundfont.clone()
    }

    pub fn set_soundfont(&mut self, value: Option<Arc<SoundFont>>) {
        self.soundfont = value;
    }

    /// Set of unique program change values in the file
    pub const fn presets(&self) -> &HashSet<u8> {
        &self.presets
    }

    pub fn render(&self) -> Result<(), PlayerError> {
        let Some(soundfont) = &self.soundfont else {
            return Err(PlayerError::AudioNoFont);
        };
        self.midi_renderer.render(self.midifile()?, soundfont)?;
        Ok(())
    }
}
