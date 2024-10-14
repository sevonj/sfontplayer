use enums::{FileListMode, FontSort, SongSort};
use error::WorkspaceError;
use font_meta::FontMeta;
use midi_meta::MidiMeta;
use rand::seq::SliceRandom;
use std::{fs, path::PathBuf, time::Duration, vec};
use walkdir::WalkDir;

pub mod enums;
mod error;
mod font_meta;
mod midi_meta;
mod serialize;

#[derive(Clone)]
pub struct Workspace {
    pub name: String,
    /// If None, this is a normal workspace. If Some, this is a portable workspace.
    portable_filepath: Option<PathBuf>,

    fonts: Vec<FontMeta>,
    font_idx: Option<usize>,
    font_list_mode: FileListMode,
    font_dir: Option<PathBuf>,
    font_sort: FontSort,

    midis: Vec<MidiMeta>,
    midi_idx: Option<usize>,
    midi_list_mode: FileListMode,
    midi_dir: Option<PathBuf>,
    song_sort: SongSort,

    pub queue: Vec<usize>,
    pub queue_idx: Option<usize>,
}
impl Workspace {
    // --- Soundfonts

    pub const fn get_fonts(&self) -> &Vec<FontMeta> {
        &self.fonts
    }
    pub fn get_fonts_mut(&mut self) -> &mut Vec<FontMeta> {
        &mut self.fonts
    }
    pub const fn get_font_idx(&self) -> Option<usize> {
        self.font_idx
    }
    pub fn set_font_idx(&mut self, value: Option<usize>) -> anyhow::Result<()> {
        if let Some(index) = value {
            self.font_idx = if index < self.fonts.len() {
                self.fonts[index].refresh();
                Some(index)
            } else {
                anyhow::bail!(WorkspaceError::InvalidFontIndex { index });
            }
        }
        Ok(())
    }
    pub fn add_font(&mut self, path: PathBuf) {
        if !self.contains_font(&path) {
            self.fonts.push(FontMeta::new(path));
            self.refresh_font_list();
        }
    }
    pub fn remove_font(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.fonts.len() {
            anyhow::bail!(WorkspaceError::InvalidFontIndex { index });
        }
        self.fonts[index].is_queued_for_deletion = true;
        Ok(())
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
    pub const fn get_font_list_mode(&self) -> FileListMode {
        self.font_list_mode
    }
    pub const fn get_font_dir(&self) -> &Option<PathBuf> {
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
        self.refresh_font_list();
    }
    /// Refresh font file list
    pub fn refresh_font_list(&mut self) {
        if self.font_list_mode == FileListMode::Manual {
            self.sort_fonts();
            return;
        }

        // Remove files
        for i in 0..self.fonts.len() {
            let filepath = self.fonts[i].get_path();
            // File doesn't exist anymore
            if !filepath.exists() {
                self.remove_font(i).expect("refresh: Font rm failed‽");
            }
            match self.font_list_mode {
                FileListMode::Directory => {
                    // Delete if dir is not immediate parent
                    if filepath.parent() != self.font_dir.as_deref() {
                        self.remove_font(i).expect("refresh: Font rm failed‽");
                    }
                }
                FileListMode::Subdirectories => {
                    // Delete if dir is not a parent
                    if let Some(dir) = &self.font_dir {
                        if !filepath.starts_with(dir) {
                            self.remove_font(i).expect("refresh: Font rm failed‽");
                        }
                    }
                }
                FileListMode::Manual => unreachable!(),
            }
        }
        self.delete_queued();

        // Look for new files
        let Some(dir) = &self.font_dir else {
            self.clear_fonts();
            return;
        };
        match self.font_list_mode {
            FileListMode::Directory => {
                if let Ok(paths) = fs::read_dir(dir) {
                    for entry in paths.filter_map(std::result::Result::ok) {
                        let path = entry.path();
                        if self.contains_font(&path) {
                            continue;
                        }
                        if path.is_file() && path.extension().is_some_and(|s| s == "sf2") {
                            self.add_font(path);
                        }
                    }
                }
            }
            FileListMode::Subdirectories => {
                for entry in WalkDir::new(dir)
                    .into_iter()
                    .filter_map(std::result::Result::ok)
                {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|s| s == "sf2") {
                        self.add_font(path.into());
                    }
                }
            }
            FileListMode::Manual => unreachable!(),
        }
        self.sort_fonts();
    }
    fn sort_fonts(&mut self) {
        // Remember the selected
        let selected_font = if let Some(index) = self.font_idx {
            Some(self.fonts[index].clone())
        } else {
            None
        };

        // Sort
        match self.font_sort {
            FontSort::NameAsc => self.fonts.sort_by_key(|f| f.get_name().to_lowercase()),
            FontSort::NameDesc => {
                self.fonts.sort_by_key(|f| f.get_name().to_lowercase());
                self.fonts.reverse();
            }

            FontSort::SizeAsc => self.fonts.sort_by_key(font_meta::FontMeta::get_size),
            FontSort::SizeDesc => {
                self.fonts.sort_by_key(font_meta::FontMeta::get_size);
                self.fonts.reverse();
            }
        };

        // Find the selected again
        if let Some(selected) = selected_font {
            for i in 0..self.fonts.len() {
                if self.fonts[i].get_path() == selected.get_path() {
                    self.font_idx = Some(i);
                }
            }
        }
    }
    pub const fn get_font_sort(&self) -> FontSort {
        self.font_sort
    }
    pub fn set_font_sort(&mut self, sort: FontSort) {
        self.font_sort = sort;
        self.refresh_font_list();
    }

    // --- Midi files

    pub const fn get_songs(&self) -> &Vec<MidiMeta> {
        &self.midis
    }
    pub fn get_songs_mut(&mut self) -> &mut Vec<MidiMeta> {
        &mut self.midis
    }
    pub const fn get_song_idx(&self) -> Option<usize> {
        self.midi_idx
    }
    pub fn set_song_idx(&mut self, value: Option<usize>) -> anyhow::Result<()> {
        if let Some(index) = value {
            self.midi_idx = if index < self.midis.len() {
                self.midis[index].refresh();
                Some(index)
            } else {
                anyhow::bail!(WorkspaceError::InvalidSongIndex { index });
            }
        }
        Ok(())
    }
    pub fn add_song(&mut self, path: PathBuf) {
        if !self.contains_song(&path) {
            self.midis.push(MidiMeta::new(path));
            self.refresh_song_list();
        }
    }
    pub fn remove_song(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.midis.len() {
            anyhow::bail!(WorkspaceError::InvalidSongIndex { index });
        }
        self.midis[index].is_queued_for_deletion = true;
        Ok(())
    }
    pub fn clear_songs(&mut self) {
        self.midis.clear();
        self.midi_idx = None;
    }
    pub fn contains_song(&self, filepath: &PathBuf) -> bool {
        for i in 0..self.midis.len() {
            if self.midis[i].get_path() == *filepath {
                return true;
            }
        }
        false
    }
    pub const fn get_song_list_mode(&self) -> FileListMode {
        self.midi_list_mode
    }
    pub const fn get_song_dir(&self) -> &Option<PathBuf> {
        &self.midi_dir
    }
    pub fn set_song_dir(&mut self, path: PathBuf) {
        if self.midi_list_mode == FileListMode::Manual {
            return;
        }
        self.midi_dir = Some(path);
        self.refresh_song_list();
    }
    pub fn set_song_list_mode(&mut self, mode: FileListMode) {
        self.midi_list_mode = mode;
        self.refresh_song_list();
    }
    /// Refresh midi file list
    pub fn refresh_song_list(&mut self) {
        if self.midi_list_mode == FileListMode::Manual {
            self.sort_songs();
            return;
        }

        // Remove files
        for i in 0..self.midis.len() {
            let filepath = self.midis[i].get_path();
            // File doesn't exist anymore
            if !filepath.exists() {
                self.remove_song(i).expect("refresh: Song rm failed‽");
            }
            match self.midi_list_mode {
                FileListMode::Directory => {
                    // Delete if dir is not immediate parent
                    if filepath.parent() != self.midi_dir.as_deref() {
                        self.remove_song(i).expect("refresh: Song rm failed‽");
                    }
                }
                FileListMode::Subdirectories => {
                    // Delete if dir is not a parent
                    if let Some(dir) = &self.midi_dir {
                        if !filepath.starts_with(dir) {
                            self.remove_song(i).expect("refresh: Song rm failed‽");
                        }
                    }
                }
                FileListMode::Manual => unreachable!(),
            }
        }
        self.delete_queued();

        // Look for new files
        let Some(dir) = &self.midi_dir else {
            self.clear_songs();
            return;
        };
        match self.midi_list_mode {
            FileListMode::Directory => {
                if let Ok(paths) = fs::read_dir(dir) {
                    for entry in paths.filter_map(std::result::Result::ok) {
                        let path = entry.path();
                        if self.contains_song(&path) {
                            continue;
                        }
                        if path.is_file() && path.extension().is_some_and(|s| s == "mid") {
                            self.add_song(path);
                        }
                    }
                }
            }
            FileListMode::Subdirectories => {
                for entry in WalkDir::new(dir)
                    .into_iter()
                    .filter_map(std::result::Result::ok)
                {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|s| s == "mid") {
                        self.add_song(path.into());
                    }
                }
            }
            FileListMode::Manual => unreachable!(),
        }
        self.sort_songs();
    }
    fn sort_songs(&mut self) {
        // Remember the  selected
        let selected_song = if let Some(index) = self.midi_idx {
            Some(self.midis[index].clone())
        } else {
            None
        };

        // Sort
        match self.song_sort {
            SongSort::NameAsc => self.midis.sort_by_key(|f| f.get_name().to_lowercase()),
            SongSort::NameDesc => {
                self.midis.sort_by_key(|f| f.get_name().to_lowercase());
                self.midis.reverse();
            }

            SongSort::TimeAsc => self
                .midis
                .sort_by_key(|f| f.get_duration().unwrap_or(Duration::ZERO)),
            SongSort::TimeDesc => self
                .midis
                .sort_by_key(|f| f.get_duration().unwrap_or(Duration::ZERO)),

            SongSort::SizeAsc => self.midis.sort_by_key(midi_meta::MidiMeta::get_size),
            SongSort::SizeDesc => {
                self.midis.sort_by_key(midi_meta::MidiMeta::get_size);
                self.midis.reverse();
            }
        };

        // Find the selected again
        if let Some(selected) = selected_song {
            for i in 0..self.midis.len() {
                if self.midis[i].get_path() == selected.get_path() {
                    self.midi_idx = Some(i);
                }
            }
        }
    }
    pub const fn get_song_sort(&self) -> SongSort {
        self.song_sort
    }
    pub fn set_song_sort(&mut self, sort: SongSort) {
        self.song_sort = sort;
        self.refresh_song_list();
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

    pub const fn is_portable(&self) -> bool {
        self.portable_filepath.is_some()
    }
    pub fn get_portable_path(&self) -> Option<PathBuf> {
        self.portable_filepath.clone()
    }
    pub fn set_portable_path(&mut self, portable_filepath: Option<PathBuf>) {
        self.portable_filepath = portable_filepath;
    }

    /// Midis and fonts aren't deleted immediately. A queue is used instead.
    /// This handles the queues, call at the end of the frame.
    pub fn delete_queued(&mut self) {
        // Songs
        for i in (0..self.midis.len()).rev() {
            if !self.midis[i].is_queued_for_deletion {
                continue;
            }
            self.midis.remove(i);

            // Check if deletion affected selected index
            if let Some(current) = self.midi_idx {
                match i {
                    deleted if deleted == current => self.midi_idx = None,
                    deleted if deleted < current => self.midi_idx = Some(current - 1),
                    _ => (),
                }
            }
        }

        // Fonts
        for i in (0..self.fonts.len()).rev() {
            if !self.fonts[i].is_queued_for_deletion {
                continue;
            }
            self.fonts.remove(i);

            // Check if deletion affected index
            if let Some(current) = self.font_idx {
                match i {
                    deleted if deleted == current => self.font_idx = None,
                    deleted if deleted < current => self.font_idx = Some(current - 1),
                    _ => (),
                }
            }
        }
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            name: "Workspace".to_owned(),
            portable_filepath: None,

            fonts: vec![],
            font_idx: None,
            font_list_mode: FileListMode::Manual,
            font_dir: None,
            font_sort: FontSort::default(),

            midis: vec![],
            midi_idx: None,
            midi_list_mode: FileListMode::Manual,
            midi_dir: None,
            song_sort: SongSort::default(),

            queue: vec![],
            queue_idx: None,
        }
    }
}
