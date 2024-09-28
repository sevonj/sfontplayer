mod font_meta;
mod midi_meta;

use font_meta::FontMeta;
use midi_meta::MidiMeta;
use rand::seq::SliceRandom;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::{
    fs::{self},
    path::PathBuf,
    vec,
};
use walkdir::WalkDir;

/// Option for how soundfonts or midis are managed
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Default, Clone, Copy, Debug)]
#[repr(u8)]
pub(crate) enum FileListMode {
    /// The contents are added and removed manually.
    #[default]
    Manual,
    /// The contents are fetched automatically from a directory.
    Directory,
    /// The contents are fetched automatically from a directory and subdirectories.
    Subdirectories,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct Workspace {
    pub name: String,

    pub fonts: Vec<FontMeta>,
    pub font_idx: Option<usize>,
    font_list_mode: FileListMode,
    font_dir: Option<PathBuf>,
    #[serde(skip)]
    midi_delete_queue: Vec<usize>,

    pub midis: Vec<MidiMeta>,
    pub midi_idx: Option<usize>,
    midi_list_mode: FileListMode,
    pub midi_dir: Option<PathBuf>,
    #[serde(skip)]
    font_delete_queue: Vec<usize>,

    #[serde(skip)]
    pub queue: Vec<usize>,
    #[serde(skip)]
    pub queue_idx: Option<usize>,
}
impl Workspace {
    // --- Soundfonts

    pub fn add_font(&mut self, path: PathBuf) {
        if !self.contains_font(&path) {
            self.fonts.push(FontMeta::new(path));
        }
    }
    pub fn remove_font(&mut self, index: usize) {
        self.font_delete_queue.push(index);
    }
    pub fn clear_fonts(&mut self) {
        self.fonts.clear();
        self.font_idx = None;
    }
    pub fn contains_font(&self, filepath: &PathBuf) -> bool {
        for i in 0..self.fonts.len() {
            if self.fonts[i].get_path() == *filepath {
                return true;
            }
        }
        false
    }
    pub fn get_font_list_mode(&self) -> FileListMode {
        self.font_list_mode
    }
    pub fn get_font_dir(&self) -> &Option<PathBuf> {
        &self.font_dir
    }
    pub fn set_font_dir(&mut self, path: PathBuf) {
        if self.font_list_mode == FileListMode::Manual {
            return;
        }
        self.font_dir = Some(path);
        self.refresh_font_list();
    }
    pub fn set_font_list_type(&mut self, mode: FileListMode) {
        self.font_list_mode = mode;
        self.refresh_font_list()
    }
    /// Refresh font file list for directory list.
    pub fn refresh_font_list(&mut self) {
        if self.font_list_mode == FileListMode::Manual {
            return;
        }

        // Remove files
        for i in 0..self.fonts.len() {
            let filepath = self.fonts[i].get_path();
            // File doesn't exist anymore
            if !filepath.exists() {
                self.remove_font(i);
            }
            match self.font_list_mode {
                FileListMode::Directory => {
                    // Delete if dir is not immediate parent
                    if filepath.parent() != self.font_dir.as_deref() {
                        self.remove_font(i);
                    }
                }
                FileListMode::Subdirectories => {
                    // Delete if dir is not a parent
                    if let Some(dir) = &self.font_dir {
                        if !filepath.starts_with(dir) {
                            self.remove_font(i);
                        }
                    }
                }
                FileListMode::Manual => unreachable!(),
            }
        }
        self.delete_queued();

        // Look for new files
        let dir = match &self.font_dir {
            Some(path) => path,
            None => {
                self.clear_fonts();
                return;
            }
        };
        match self.font_list_mode {
            FileListMode::Directory => {
                let paths = fs::read_dir(dir).unwrap();
                for entry in paths.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if self.contains_font(&path) {
                        continue;
                    }
                    if path.is_file() && path.extension().map(|s| s == "sf2").unwrap_or(false) {
                        self.add_font(path);
                    }
                }
            }
            FileListMode::Subdirectories => {
                for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && path.extension().map(|s| s == "sf2").unwrap_or(false) {
                        self.add_font(path.into());
                    }
                }
            }
            FileListMode::Manual => unreachable!(),
        }
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
    pub fn contains_midi(&self, filepath: &PathBuf) -> bool {
        for i in 0..self.midis.len() {
            if self.midis[i].get_path() == *filepath {
                return true;
            }
        }
        false
    }
    pub fn get_midi_list_mode(&self) -> FileListMode {
        self.midi_list_mode
    }
    pub fn get_midi_dir(&self) -> &Option<PathBuf> {
        &self.midi_dir
    }
    pub fn set_midi_dir(&mut self, path: PathBuf) {
        if self.midi_list_mode == FileListMode::Manual {
            return;
        }
        self.midi_dir = Some(path);
        self.refresh_midi_list();
    }
    pub fn set_midi_list_mode(&mut self, mode: FileListMode) {
        self.midi_list_mode = mode;
        self.refresh_midi_list()
    }
    /// Refresh midi file list for directory list.
    pub fn refresh_midi_list(&mut self) {
        if self.midi_list_mode == FileListMode::Manual {
            return;
        }

        println!("Midi refresh!");
        // Remove files
        for i in 0..self.midis.len() {
            let filepath = self.midis[i].get_path();
            // File doesn't exist anymore
            if !filepath.exists() {
                self.remove_midi(i);
            }
            match self.midi_list_mode {
                FileListMode::Directory => {
                    // Delete if dir is not immediate parent
                    if filepath.parent() != self.midi_dir.as_deref() {
                        self.remove_midi(i);
                    }
                }
                FileListMode::Subdirectories => {
                    // Delete if dir is not a parent
                    if let Some(dir) = &self.midi_dir {
                        if !filepath.starts_with(dir) {
                            self.remove_midi(i);
                        }
                    }
                }
                FileListMode::Manual => unreachable!(),
            }
        }
        self.delete_queued();

        // Look for new files
        let dir = match &self.midi_dir {
            Some(path) => path,
            None => {
                self.clear_midis();
                return;
            }
        };
        match self.midi_list_mode {
            FileListMode::Directory => {
                let paths = fs::read_dir(dir).unwrap();
                for entry in paths.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if self.contains_midi(&path) {
                        continue;
                    }
                    if path.is_file() && path.extension().map(|s| s == "mid").unwrap_or(false) {
                        self.add_midi(path);
                    }
                }
            }
            FileListMode::Subdirectories => {
                for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && path.extension().map(|s| s == "mid").unwrap_or(false) {
                        self.add_midi(path.into());
                    }
                }
            }
            FileListMode::Manual => unreachable!(),
        }
    }

    // --- Playback Queue

    /// Create a new song queue from currently available songs.
    /// To be called when song list changes, or shuffle is toggled
    pub fn rebuild_queue(&mut self, shuffle: bool) {
        self.queue.clear();

        // Create a sequential queue from all songs
        for i in 0..self.midis.len() {
            self.queue.push(i);
        }

        // Start from currently selected song, if any
        self.queue_idx = Some(self.midi_idx.unwrap_or(0));

        if shuffle {
            self.queue.shuffle(&mut rand::thread_rng());

            // Make selected song the first in queue
            if let Some(song_idx) = self.midi_idx {
                self.queue.retain(|&x| x != song_idx); // Remove song from queue
                self.queue.insert(0, song_idx); // Insert it to the beginning.
            }
            // Because selected song always becomes first on shuffle
            self.queue_idx = Some(0);
        }
    }

    // --- Misc.

    /// Midis and fonts aren't deleted immediately. A queue is used instead.
    /// This handles the queues, call at the end of the frame.
    pub fn delete_queued(&mut self) {
        self.midi_delete_queue.sort();
        self.midi_delete_queue.reverse();
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

        self.font_delete_queue.sort();
        self.font_delete_queue.reverse();
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
            font_idx: None,
            font_list_mode: FileListMode::Manual,
            font_dir: None,
            font_delete_queue: vec![],

            midis: vec![],
            midi_idx: None,
            midi_list_mode: FileListMode::Manual,
            midi_dir: None,
            midi_delete_queue: vec![],

            queue: vec![],
            queue_idx: None,
        }
    }
}
