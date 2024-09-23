use std::{fs::File, path::PathBuf, time::Duration, vec};

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
    pub fn get_duration(&self) -> Option<Duration> {
        self.duration.clone()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct Workspace {
    pub name: String,
    pub soundfonts: Vec<PathBuf>,
    pub midis: Vec<MidiMeta>,
    pub selected_sf: Option<usize>,
    pub selected_midi: Option<usize>,
    pub queue: Vec<usize>,
    #[serde(skip)]
    pub queue_idx: Option<usize>,
}
impl Workspace {
    pub fn contains_midi(&self, filepath: &PathBuf) -> bool {
        for i in 0..self.midis.len() {
            if self.midis[i].get_path() == filepath.to_owned() {
                return true;
            }
        }
        false
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            name: "Workspace".to_owned(),
            soundfonts: vec![],
            midis: vec![],
            selected_sf: None,
            selected_midi: None,
            queue: vec![],
            queue_idx: None,
        }
    }
}
