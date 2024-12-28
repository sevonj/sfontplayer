use midi_msg::{Header, Meta, MidiFile, MidiMsg, Track};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct MidiInspectorTrack {
    pub track: Track,
    pub open: bool,
    pub name: Option<String>,
}
impl MidiInspectorTrack {
    pub fn new(track: Track) -> Self {
        let name = get_track_name(&track);

        Self {
            track,
            open: false,
            name,
        }
    }
}

fn get_track_name(track: &Track) -> Option<String> {
    for trackevent in track.events() {
        let MidiMsg::Meta { msg } = &trackevent.event else {
            continue;
        };
        let Meta::TrackName(name) = msg else {
            continue;
        };
        return Some(name.to_owned());
    }
    None
}

pub struct MidiInspector {
    pub filepath: PathBuf,
    pub header: Header,
    pub tracks: Vec<MidiInspectorTrack>,
}

impl MidiInspector {
    pub fn new(filepath: &Path) -> anyhow::Result<Self> {
        let bytes = fs::read(filepath)?;
        let midifile = MidiFile::from_midi(bytes.as_slice())?;

        let filepath = filepath.to_owned();
        let header = midifile.header;
        let mut tracks = vec![];
        for track in midifile.tracks {
            tracks.push(MidiInspectorTrack::new(track));
        }

        Ok(Self {
            filepath,
            header,
            tracks,
        })
    }
}
