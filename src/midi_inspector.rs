use midi_msg::MidiFile;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct MidiInspector {
    pub midifile: MidiFile,
    pub filepath: PathBuf,
}

impl MidiInspector {
    pub fn new(filepath: &Path) -> anyhow::Result<Self> {
        let bytes = fs::read(filepath)?;
        let midifile = MidiFile::from_midi(bytes.as_slice())?;
        let filepath = filepath.to_owned();
        Ok(Self { midifile, filepath })
    }
}
