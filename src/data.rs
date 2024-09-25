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
        self.duration
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct Workspace {
    pub name: String,
    pub fonts: Vec<PathBuf>,
    pub midis: Vec<MidiMeta>,
    pub font_idx: Option<usize>,
    pub midi_idx: Option<usize>,
    pub queue: Vec<usize>,
    #[serde(skip)]
    pub queue_idx: Option<usize>,

    #[serde(skip)]
    midi_delete_queue: Vec<usize>,
    #[serde(skip)]
    font_delete_queue: Vec<usize>,
}
impl Workspace {
    pub fn contains_midi(&self, filepath: &PathBuf) -> bool {
        for i in 0..self.midis.len() {
            if self.midis[i].get_path() == *filepath {
                return true;
            }
        }
        false
    }
    pub fn remove_midi(&mut self, index: usize) {
        self.midi_delete_queue.push(index);
    }
    pub fn remove_font(&mut self, index: usize) {
        self.font_delete_queue.push(index);
    }
    /// Delete fonts and midis queued for removal.
    pub fn delete_queued(&mut self) {
        for deleted_idx in self.midi_delete_queue.clone() {
            self.midis.remove(deleted_idx);

            // Check if deletion affected index
            if let Some(current) = self.midi_idx {
                match deleted_idx {
                    deleted if deleted == current => self.midi_idx = None,
                    deleted if deleted < current => self.midi_idx = Some(current - 1),
                    _ => (),
                }
            }
        }
        self.midi_delete_queue.clear();
        for deleted_idx in self.font_delete_queue.clone() {
            self.fonts.remove(deleted_idx);

            // Check if deletion affected index
            if let Some(current) = self.font_idx {
                match deleted_idx {
                    deleted if deleted == current => self.font_idx = None,
                    deleted if deleted < current => self.font_idx = Some(current - 1),
                    _ => (),
                }
            }
        }
        self.font_delete_queue.clear();
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            name: "Workspace".to_owned(),
            fonts: vec![],
            midis: vec![],
            font_idx: None,
            midi_idx: None,
            queue: vec![],
            queue_idx: None,
            midi_delete_queue: vec![],
            font_delete_queue: vec![],
        }
    }
}
