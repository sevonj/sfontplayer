use std::{fs::File, path::PathBuf, time::Duration};

use rustysynth::MidiFile;

/// Reference to a midi file with metadata
#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
#[serde(default)]
pub(crate) struct MidiMeta {
    filepath: PathBuf,
    duration: Option<Duration>,
    error: bool,
}
impl MidiMeta {
    pub fn new(filepath: PathBuf) -> Self {
        let mut this = Self {
            filepath,
            duration: None,
            error: false,
        };
        this.refresh();
        this
    }
    pub fn refresh(&mut self) {
        if let Ok(mut file) = File::open(&self.filepath) {
            if let Ok(midifile) = MidiFile::new(&mut file) {
                self.duration = Some(Duration::from_secs_f64(midifile.get_length()));
                self.error = false;
            } else {
                self.duration = None;
                self.error = true;
            }
        }
    }
    pub fn get_path(&self) -> PathBuf {
        self.filepath.clone()
    }
    pub fn get_name(&self) -> String {
        self.filepath
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }
    pub fn get_duration(&self) -> Option<Duration> {
        self.duration
    }
    pub fn is_error(&self) -> bool {
        self.error
    }
}
