use rand::seq::SliceRandom;
use std::{fs, path::PathBuf, time::Duration, vec};
use walkdir::WalkDir;

use super::soundfont_list::{FontList, FontListError, FontSort};
pub use enums::{FileListMode, SongSort};
pub use error::{MetaError, PlaylistError};
pub use font_meta::FontMeta;
pub use midi_meta::MidiMeta;

mod enums;
mod error;
mod font_meta;
mod midi_meta;
mod serialize_playlist;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DeletionStatus {
    None,
    /// Queued for deletion.
    Queued,
    /// Queued, and delete even if unsaved changes.
    QueuedDiscard,
}

#[derive(Clone)]
pub struct Playlist {
    pub name: String,
    /// If None, this is a normal playlist. If Some, this is a portable playlist.
    portable_filepath: Option<PathBuf>,
    /// Only applicable to portable file
    unsaved_changes: bool,
    pub deletion_status: DeletionStatus,

    fonts: FontList,
    font_list_mode: FileListMode,
    font_dir: Option<PathBuf>,

    midis: Vec<MidiMeta>,
    midi_idx: Option<usize>,
    song_list_mode: FileListMode,
    midi_dir: Option<PathBuf>,
    song_sort: SongSort,

    pub queue: Vec<usize>,
    pub queue_idx: Option<usize>,
}
impl Playlist {
    pub fn add_file(&mut self, path: PathBuf) -> Result<(), PlaylistError> {
        // Fast quess
        if path.ends_with(".mid") {
            let midimeta = MidiMeta::new(path.clone());
            if midimeta.get_status().is_ok() {
                return self.add_song(path);
            }
        }
        // Try all types
        let fontmeta = FontMeta::new(path.clone());
        if fontmeta.get_status().is_ok() {
            return self.add_font(path);
        }
        let midimeta = MidiMeta::new(path.clone());
        if midimeta.get_status().is_ok() {
            return self.add_song(path);
        }

        Err(PlaylistError::UnknownFileFormat { path })
    }

    // --- FontList

    pub fn get_font(&self, index: usize) -> Result<&FontMeta, FontListError> {
        self.fonts.get_font(index)
    }

    pub fn get_font_mut(&mut self, index: usize) -> Result<&mut FontMeta, FontListError> {
        self.fonts.get_font_mut(index)
    }

    pub fn get_selected_font(&self) -> Option<&FontMeta> {
        self.fonts.get_selected()
    }

    pub fn get_selected_font_mut(&mut self) -> Option<&mut FontMeta> {
        self.fonts.get_selected_mut()
    }

    pub const fn get_fonts(&self) -> &Vec<FontMeta> {
        self.fonts.get_fonts()
    }

    pub const fn get_font_idx(&self) -> Option<usize> {
        self.fonts.get_selected_index()
    }

    pub fn select_font(&mut self, index: usize) -> Result<(), PlaylistError> {
        self.fonts.set_selected_index(Some(index))?;
        Ok(())
    }

    pub fn deselect_font(&mut self) {
        self.fonts.set_selected_index(None).expect("unreachable");
    }

    pub fn add_font(&mut self, path: PathBuf) -> Result<(), PlaylistError> {
        if self.font_list_mode != FileListMode::Manual {
            return Err(PlaylistError::ModifyDirList);
        }
        self.force_add_font(path);
        self.recrawl_fonts();
        Ok(())
    }

    /// Bypasses extra correctness checks meant for gui.
    fn force_add_font(&mut self, path: PathBuf) {
        if !self.contains_font(&path) {
            let _ = self.fonts.add(FontMeta::new(path));
        }
        self.unsaved_changes = true;
    }

    pub fn remove_font(&mut self, index: usize) -> Result<(), PlaylistError> {
        if self.font_list_mode != FileListMode::Manual {
            return Err(PlaylistError::ModifyDirList);
        }
        self.force_remove_font(index)
    }

    /// Bypasses extra correctness checks meant for gui.
    fn force_remove_font(&mut self, index: usize) -> Result<(), PlaylistError> {
        self.unsaved_changes = true;
        self.fonts.remove(index)?;
        Ok(())
    }

    pub fn clear_fonts(&mut self) {
        self.fonts.clear();
        self.unsaved_changes = true;
    }

    pub fn contains_font(&self, filepath: &PathBuf) -> bool {
        self.fonts.contains(filepath)
    }

    fn sort_fonts(&mut self) {
        self.fonts.sort();
    }

    pub const fn get_font_sort(&self) -> FontSort {
        self.fonts.get_sort()
    }

    pub fn set_font_sort(&mut self, sort: FontSort) {
        self.fonts.set_sort(sort);
        self.recrawl_fonts();
    }

    // --- Font Management

    pub const fn get_font_list_mode(&self) -> FileListMode {
        self.font_list_mode
    }

    pub const fn get_font_dir(&self) -> Option<&PathBuf> {
        self.font_dir.as_ref()
    }

    pub fn set_font_dir(&mut self, path: PathBuf) {
        if self.font_list_mode == FileListMode::Manual {
            return;
        }
        self.font_dir = Some(path);
        self.recrawl_fonts();
        self.unsaved_changes = true;
    }

    pub fn set_font_list_mode(&mut self, mode: FileListMode) {
        self.font_list_mode = mode;
        self.recrawl_fonts();
        self.unsaved_changes = true;
    }

    pub fn recrawl_fonts(&mut self) {
        if self.font_list_mode == FileListMode::Manual {
            self.sort_fonts();
            return;
        }

        // Remove files
        let mut to_be_removed = vec![];
        for (i, font) in self.fonts.get_fonts().iter().enumerate() {
            let filepath = font.get_path();
            // File doesn't exist anymore
            if !filepath.exists() {
                to_be_removed.push(i);
            }
            match self.font_list_mode {
                FileListMode::Directory => {
                    // Delete if dir is not immediate parent
                    if filepath.parent() != self.font_dir.as_deref() {
                        to_be_removed.push(i);
                    }
                }
                FileListMode::Subdirectories => {
                    // Delete if dir is not a parent
                    if let Some(dir) = &self.font_dir {
                        if !filepath.starts_with(dir) {
                            to_be_removed.push(i);
                        }
                    }
                }
                FileListMode::Manual => unreachable!(),
            }
        }
        for i in to_be_removed {
            self.force_remove_font(i).expect("refresh: Font rm failed‽");
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
                            self.force_add_font(path);
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
                        self.force_add_font(path.into());
                    }
                }
            }
            FileListMode::Manual => unreachable!(),
        }
        self.sort_fonts();
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
    pub fn set_song_idx(&mut self, value: Option<usize>) -> Result<(), PlaylistError> {
        match value {
            Some(index) => {
                self.midi_idx = if index < self.midis.len() {
                    self.midis[index].refresh();
                    Some(index)
                } else {
                    return Err(PlaylistError::InvalidIndex);
                }
            }
            None => self.midi_idx = None,
        }
        Ok(())
    }
    pub fn add_song(&mut self, path: PathBuf) -> Result<(), PlaylistError> {
        if self.song_list_mode != FileListMode::Manual {
            return Err(PlaylistError::ModifyDirList);
        }
        self.force_add_song(path);
        self.refresh_song_list();
        Ok(())
    }
    /// Bypasses extra correctness checks meant for gui.
    fn force_add_song(&mut self, path: PathBuf) {
        if !self.contains_song(&path) {
            self.midis.push(MidiMeta::new(path));
        }
        self.unsaved_changes = true;
    }
    pub fn remove_song(&mut self, index: usize) -> Result<(), PlaylistError> {
        if self.song_list_mode != FileListMode::Manual {
            return Err(PlaylistError::ModifyDirList);
        }
        self.force_remove_song(index)
    }
    /// Bypasses extra correctness checks meant for gui.
    fn force_remove_song(&mut self, index: usize) -> Result<(), PlaylistError> {
        if index >= self.midis.len() {
            return Err(PlaylistError::InvalidIndex);
        }
        self.midis[index].is_queued_for_deletion = true;
        self.unsaved_changes = true;
        Ok(())
    }
    pub fn clear_songs(&mut self) {
        self.midis.clear();
        self.midi_idx = None;
        self.unsaved_changes = true;
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
        self.song_list_mode
    }
    pub const fn get_song_dir(&self) -> Option<&PathBuf> {
        self.midi_dir.as_ref()
    }
    pub fn set_song_dir(&mut self, path: PathBuf) {
        if self.song_list_mode == FileListMode::Manual {
            return;
        }
        self.midi_dir = Some(path);
        self.refresh_song_list();
        self.unsaved_changes = true;
    }
    pub fn set_song_list_mode(&mut self, mode: FileListMode) {
        self.song_list_mode = mode;
        self.refresh_song_list();
        self.unsaved_changes = true;
    }
    /// Refresh midi file list
    pub fn refresh_song_list(&mut self) {
        if self.song_list_mode == FileListMode::Manual {
            self.sort_songs();
            return;
        }

        // Remove files
        for i in 0..self.midis.len() {
            let filepath = self.midis[i].get_path();
            // File doesn't exist anymore
            if !filepath.exists() {
                self.force_remove_song(i).expect("refresh: Song rm failed‽");
            }
            match self.song_list_mode {
                FileListMode::Directory => {
                    // Delete if dir is not immediate parent
                    if filepath.parent() != self.midi_dir.as_deref() {
                        self.force_remove_song(i).expect("refresh: Song rm failed‽");
                    }
                }
                FileListMode::Subdirectories => {
                    // Delete if dir is not a parent
                    if let Some(dir) = &self.midi_dir {
                        if !filepath.starts_with(dir) {
                            self.force_remove_song(i).expect("refresh: Song rm failed‽");
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
        match self.song_list_mode {
            FileListMode::Directory => {
                if let Ok(paths) = fs::read_dir(dir) {
                    for entry in paths.filter_map(std::result::Result::ok) {
                        let path = entry.path();
                        if self.contains_song(&path) {
                            continue;
                        }
                        if path.is_file() && path.extension().is_some_and(|s| s == "mid") {
                            self.force_add_song(path);
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
                        self.force_add_song(path.into());
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
            SongSort::TimeDesc => {
                self.midis
                    .sort_by_key(|f| f.get_duration().unwrap_or(Duration::ZERO));
                self.midis.reverse();
            }
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
        self.unsaved_changes = true;
    }
    pub const fn has_unsaved_changes(&self) -> bool {
        self.is_portable() && self.unsaved_changes
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

        self.fonts.delete_queued();
    }
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            name: "Playlist".to_owned(),
            portable_filepath: None,
            unsaved_changes: true,
            deletion_status: DeletionStatus::None,

            fonts: FontList::default(),
            font_list_mode: FileListMode::Manual,
            font_dir: None,

            midis: vec![],
            midi_idx: None,
            song_list_mode: FileListMode::Manual,
            midi_dir: None,
            song_sort: SongSort::default(),

            queue: vec![],
            queue_idx: None,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_addfont_listmodes() {
        let mut playlist_man = Playlist::default();
        let mut playlist_dir = Playlist::default();
        let mut playlist_sub = Playlist::default();
        playlist_man.font_list_mode = FileListMode::Manual;
        playlist_dir.font_list_mode = FileListMode::Directory;
        playlist_sub.font_list_mode = FileListMode::Subdirectories;
        playlist_man.add_font("fakepath".into()).unwrap();
        assert!(matches!(
            playlist_dir.add_font("fakepath".into()).unwrap_err(),
            PlaylistError::ModifyDirList
        ));
        assert!(matches!(
            playlist_sub.add_font("fakepath".into()).unwrap_err(),
            PlaylistError::ModifyDirList
        ));
        assert_eq!(playlist_man.fonts.get_fonts().len(), 1);
        assert_eq!(playlist_dir.fonts.get_fonts().len(), 0);
        assert_eq!(playlist_sub.fonts.get_fonts().len(), 0);
    }
    #[test]
    fn test_rmfont_listmodes() {
        let mut playlist_man = Playlist::default();
        let mut playlist_dir = Playlist::default();
        let mut playlist_sub = Playlist::default();
        playlist_man.add_font("fakepath".into()).unwrap();
        playlist_dir.add_font("fakepath".into()).unwrap();
        playlist_sub.add_font("fakepath".into()).unwrap();
        playlist_man.font_list_mode = FileListMode::Manual;
        playlist_dir.font_list_mode = FileListMode::Directory;
        playlist_sub.font_list_mode = FileListMode::Subdirectories;

        playlist_man.remove_font(0).unwrap();
        assert!(matches!(
            playlist_dir.remove_font(0).unwrap_err(),
            PlaylistError::ModifyDirList
        ));
        assert!(matches!(
            playlist_sub.remove_font(0).unwrap_err(),
            PlaylistError::ModifyDirList
        ));
        playlist_man.delete_queued();
        playlist_dir.delete_queued();
        playlist_sub.delete_queued();

        assert_eq!(playlist_man.fonts.get_fonts().len(), 0);
        assert_eq!(playlist_dir.fonts.get_fonts().len(), 1);
        assert_eq!(playlist_sub.fonts.get_fonts().len(), 1);
    }
    #[test]
    fn test_addsong_listmodes() {
        let mut playlist_man = Playlist::default();
        let mut playlist_dir = Playlist::default();
        let mut playlist_sub = Playlist::default();
        playlist_man.song_list_mode = FileListMode::Manual;
        playlist_dir.song_list_mode = FileListMode::Directory;
        playlist_sub.song_list_mode = FileListMode::Subdirectories;
        playlist_man.add_song("fakepath".into()).unwrap();
        assert!(matches!(
            playlist_dir.add_song("fakepath".into()).unwrap_err(),
            PlaylistError::ModifyDirList
        ));
        assert!(matches!(
            playlist_sub.add_song("fakepath".into()).unwrap_err(),
            PlaylistError::ModifyDirList
        ));
        assert_eq!(playlist_man.midis.len(), 1);
        assert_eq!(playlist_dir.midis.len(), 0);
        assert_eq!(playlist_sub.midis.len(), 0);
    }
    #[test]
    fn test_rmsong_listmodes() {
        let mut playlist_man = Playlist::default();
        let mut playlist_dir = Playlist::default();
        let mut playlist_sub = Playlist::default();
        playlist_man.add_song("fakepath".into()).unwrap();
        playlist_dir.add_song("fakepath".into()).unwrap();
        playlist_sub.add_song("fakepath".into()).unwrap();
        playlist_man.song_list_mode = FileListMode::Manual;
        playlist_dir.song_list_mode = FileListMode::Directory;
        playlist_sub.song_list_mode = FileListMode::Subdirectories;

        playlist_man.remove_song(0).unwrap();
        assert!(matches!(
            playlist_dir.remove_song(0).unwrap_err(),
            PlaylistError::ModifyDirList
        ));
        assert!(matches!(
            playlist_sub.remove_song(0).unwrap_err(),
            PlaylistError::ModifyDirList
        ));
        playlist_man.delete_queued();
        playlist_dir.delete_queued();
        playlist_sub.delete_queued();

        assert_eq!(playlist_man.midis.len(), 0);
        assert_eq!(playlist_dir.midis.len(), 1);
        assert_eq!(playlist_sub.midis.len(), 1);
    }

    #[test]
    fn test_unsaved_flag_fontsong_idx() {
        // (Doesn't count, not stored in playlist)
        let mut playlist = Playlist::default();
        playlist.add_font("fakefont_a".into()).unwrap();
        playlist.add_font("fakefont_b".into()).unwrap();
        playlist.add_song("fakesong_a".into()).unwrap();
        playlist.add_song("fakesong_b".into()).unwrap();
        playlist.unsaved_changes = false;
        playlist.select_font(0).unwrap();
        playlist.select_font(1).unwrap();
        assert!(!playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.set_song_idx(None).unwrap();
        assert!(!playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_add_rm() {
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.add_font("fakepath".into()).unwrap();
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.remove_font(0).unwrap();
        assert!(playlist.unsaved_changes);
        playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.add_song("fakepath".into()).unwrap();
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.remove_song(0).unwrap();
        assert!(playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_force_add_rm() {
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.force_add_font("fakepath".into());
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.force_remove_font(0).unwrap();
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.force_add_song("fakepath".into());
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.force_remove_song(0).unwrap();
        assert!(playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_clear() {
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.clear_fonts();
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.clear_songs();
        assert!(playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_listmode() {
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.set_font_list_mode(FileListMode::Manual);
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.set_song_list_mode(FileListMode::Manual);
        assert!(playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_listdir() {
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.set_font_dir("fakepath".into());
        assert!(!playlist.unsaved_changes);
        playlist.font_list_mode = FileListMode::Directory;
        playlist.set_font_dir("fakepath".into());
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.set_song_dir("fakepath".into());
        assert!(!playlist.unsaved_changes);
        playlist.song_list_mode = FileListMode::Directory;
        playlist.set_song_dir("fakepath".into());
        assert!(playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_refresh_list() {
        // (Doesn't count, refreshed automatically)
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.recrawl_fonts();
        playlist.refresh_song_list();
        assert!(!playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_sort() {
        // (Doesn't count, refreshed automatically)
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.sort_fonts();
        playlist.sort_songs();
        assert!(!playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_sortmode() {
        // (Doesn't count, not stored in playlist)
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.set_font_sort(FontSort::NameAsc);
        assert!(!playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.set_song_sort(SongSort::NameAsc);
        assert!(!playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_setportable() {
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.set_portable_path(None);
        assert!(playlist.unsaved_changes);
    }
}
