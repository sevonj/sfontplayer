use std::{fs::File, path::PathBuf, time::Duration, vec};

use rand::seq::SliceRandom;
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

    // --- Soundfonts

    pub fn add_font(&mut self, path: PathBuf) {
        if !self.fonts.contains(&path) {
            self.fonts.push(path);
        }
    }
    pub fn remove_font(&mut self, index: usize) {
        self.font_delete_queue.push(index);
    }
    pub fn clear_fonts(&mut self) {
        self.midis.clear();
        self.midi_idx = None;
    }
    // --- Midi files

    pub fn add_midi(&mut self, path: PathBuf) {
        if !self.contains_midi(&path) {
            self.midis.push(MidiMeta::new(path));
        }
    }
    pub fn remove_midi(&mut self, index: usize) {
        self.midi_delete_queue.push(index);
    }
    pub fn clear_midis(&mut self) {
        self.midis.clear();
        self.midi_idx = None;
    }

    // --- Playback Queue

    /// Create a new song queue from currently available songs.
    /// To be called when song list changes, or shuffle is toggled
    pub fn rebuild_queue(&mut self, shuffle: bool) {
        self.queue.clear();

        // Sequential queue starting from currently selected song
        let first_song_idx = self.midi_idx;
        for i in 0..self.midis.len() {
            self.queue.push(i);
        }

        if shuffle {
            self.queue.shuffle(&mut rand::thread_rng());
            // Put current selected song to the beginnning.
            // If it doesn't exist, the first song is random result of the shuffle.
            if let Some(song_idx) = first_song_idx {
                self.queue.retain(|&x| x != song_idx); // Remove song from queue
                self.queue.insert(0, song_idx); // Insert it to the beginning.
            }
        }
    }

    // --- Misc.

    /// Delete fonts and midis queued for removal.
    /// Call this at the end of the frame.
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
