use rand::seq::SliceRandom;
use std::{fs, path::PathBuf, time::Duration, vec};
use walkdir::WalkDir;

use super::soundfont_list::{FontList, FontSort};
use super::PlayerError;
pub use enums::{FileListMode, PlaylistState, SongSort};
pub use font_meta::FontMeta;
pub use midi_meta::MidiMeta;

mod enums;
mod font_meta;
mod midi_meta;
mod serialize_playlist;

#[derive(Clone, Debug)]
pub struct Playlist {
    pub name: String,
    /// If None, this is a normal playlist. If Some, this is a portable playlist.
    portable_filepath: Option<PathBuf>,
    /// Only applicable to portable file
    unsaved_changes: bool,
    /// Is this playlist waiting to be closed?
    pub state: PlaylistState,

    fonts: FontList,
    font_list_mode: FileListMode,
    font_dir: Option<PathBuf>,

    midis: Vec<MidiMeta>,
    midi_idx: Option<usize>,
    song_list_mode: FileListMode,
    midi_dir: Option<PathBuf>,
    song_sort: SongSort,

    /// Playback queue
    pub queue: Vec<usize>,
    pub queue_idx: Option<usize>,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            name: "New Playlist".to_owned(),
            portable_filepath: None,
            unsaved_changes: true,
            state: PlaylistState::None,

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

impl Playlist {
    pub fn add_file(&mut self, path: PathBuf) -> Result<(), PlayerError> {
        // Fast guess
        if path.ends_with(".mid") {
            let midimeta = MidiMeta::new(path.clone());
            if midimeta.status().is_ok() {
                return self.add_song(path);
            }
        }
        // Try all types
        let fontmeta = FontMeta::new(path.clone());
        if fontmeta.status().is_ok() {
            return self.add_font(path);
        }
        let midimeta = MidiMeta::new(path.clone());
        if midimeta.status().is_ok() {
            return self.add_song(path);
        }

        Err(PlayerError::UnknownFileFormat { path })
    }

    // --- FontList

    pub fn get_font(&self, index: usize) -> Result<&FontMeta, PlayerError> {
        self.fonts.get_font(index)
    }

    pub fn get_font_mut(&mut self, index: usize) -> Result<&mut FontMeta, PlayerError> {
        self.fonts.get_font_mut(index)
    }

    pub fn get_selected_font(&self) -> Option<&FontMeta> {
        self.fonts.selected()
    }

    pub fn get_selected_font_mut(&mut self) -> Option<&mut FontMeta> {
        self.fonts.selected_mut()
    }

    pub const fn get_fonts(&self) -> &Vec<FontMeta> {
        self.fonts.fonts()
    }

    pub const fn get_font_idx(&self) -> Option<usize> {
        self.fonts.selected_index()
    }

    pub fn select_font(&mut self, index: usize) -> Result<(), PlayerError> {
        self.fonts.set_selected_index(Some(index))?;
        Ok(())
    }

    pub fn deselect_font(&mut self) {
        self.fonts.set_selected_index(None).expect("unreachable");
    }

    pub fn add_font(&mut self, filepath: PathBuf) -> Result<(), PlayerError> {
        if self.font_list_mode != FileListMode::Manual {
            return Err(PlayerError::ModifyDirList);
        }
        self.force_add_font(filepath)?;
        self.recrawl_fonts();
        Ok(())
    }

    /// Bypasses extra correctness checks meant for gui.
    fn force_add_font(&mut self, filepath: PathBuf) -> Result<(), PlayerError> {
        self.fonts.add(filepath)?;
        self.unsaved_changes = true;
        Ok(())
    }

    pub fn mark_font_for_removal(&mut self, index: usize) -> Result<(), PlayerError> {
        if self.font_list_mode != FileListMode::Manual {
            return Err(PlayerError::ModifyDirList);
        }
        self.force_mark_font_for_removal(index)?;
        Ok(())
    }

    /// Bypasses extra correctness checks meant for gui.
    fn force_mark_font_for_removal(&mut self, index: usize) -> Result<(), PlayerError> {
        self.unsaved_changes = true;
        self.fonts.mark_for_removal(index)?;
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
        self.fonts.sort_mode()
    }

    pub fn set_font_sort(&mut self, sort: FontSort) {
        self.fonts.set_sort_mode(sort);
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
        for (i, font) in self.fonts.fonts().iter().enumerate() {
            let filepath = font.filepath();
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
            self.force_mark_font_for_removal(i)
                .expect("refresh: Font rm failed‽");
        }
        self.remove_marked();

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
                        if path.is_file() && path.extension().is_some_and(|s| s == "sf2") {
                            // Okay to ignore FontAlreadyExists
                            let _ = self.force_add_font(path);
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
                        // Okay to ignore FontAlreadyExists
                        let _ = self.force_add_font(path.into());
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

    pub const fn get_songs_mut(&mut self) -> &mut Vec<MidiMeta> {
        &mut self.midis
    }

    pub const fn get_song_idx(&self) -> Option<usize> {
        self.midi_idx
    }

    pub fn set_song_idx(&mut self, value: Option<usize>) -> Result<(), PlayerError> {
        match value {
            Some(index) => {
                self.midi_idx = if index < self.midis.len() {
                    self.midis[index].refresh();
                    Some(index)
                } else {
                    return Err(PlayerError::MidiIndex { index });
                }
            }
            None => self.midi_idx = None,
        }
        Ok(())
    }

    pub fn add_song(&mut self, filepath: PathBuf) -> Result<(), PlayerError> {
        if self.song_list_mode != FileListMode::Manual {
            return Err(PlayerError::ModifyDirList);
        }
        self.force_add_song(filepath)?;
        self.refresh_song_list();
        Ok(())
    }

    /// Bypasses extra correctness checks meant for gui.
    fn force_add_song(&mut self, filepath: PathBuf) -> Result<(), PlayerError> {
        if self.contains_song(&filepath) {
            return Err(PlayerError::MidiAlreadyExists);
        }
        self.midis.push(MidiMeta::new(filepath));
        self.unsaved_changes = true;
        Ok(())
    }

    pub fn mark_song_for_removal(&mut self, index: usize) -> Result<(), PlayerError> {
        if self.song_list_mode != FileListMode::Manual {
            return Err(PlayerError::ModifyDirList);
        }
        self.force_mark_song_for_removal(index)
    }

    /// Bypasses extra correctness checks meant for gui.
    fn force_mark_song_for_removal(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.midis.len() {
            return Err(PlayerError::MidiIndex { index });
        }
        self.midis[index].marked_for_removal = true;
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
            if *self.midis[i].filepath() == *filepath {
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
            let filepath = self.midis[i].filepath().to_owned();
            // File doesn't exist anymore
            if !filepath.exists() {
                self.force_mark_song_for_removal(i)
                    .expect("refresh: Song rm failed‽");
            }
            match self.song_list_mode {
                FileListMode::Directory => {
                    // Delete if dir is not immediate parent
                    if filepath.parent() != self.midi_dir.as_deref() {
                        self.force_mark_song_for_removal(i)
                            .expect("refresh: Song rm failed‽");
                    }
                }
                FileListMode::Subdirectories => {
                    // Delete if dir is not a parent
                    if let Some(dir) = &self.midi_dir {
                        if !filepath.starts_with(dir) {
                            self.force_mark_song_for_removal(i)
                                .expect("refresh: Song rm failed‽");
                        }
                    }
                }
                FileListMode::Manual => unreachable!(),
            }
        }
        self.remove_marked();

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
                            // Okay to ignore MidiAlreadyExists
                            let _ = self.force_add_song(path);
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
                        // Okay to ignore MidiAlreadyExists
                        let _ = self.force_add_song(path.into());
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
            SongSort::NameAsc => self.midis.sort_by_key(|f| f.filename().to_lowercase()),
            SongSort::NameDesc => {
                self.midis.sort_by_key(|f| f.filename().to_lowercase());
                self.midis.reverse();
            }

            SongSort::TimeAsc => self
                .midis
                .sort_by_key(|f| f.duration().unwrap_or(Duration::ZERO)),
            SongSort::TimeDesc => {
                self.midis
                    .sort_by_key(|f| f.duration().unwrap_or(Duration::ZERO));
                self.midis.reverse();
            }
            SongSort::SizeAsc => self.midis.sort_by_key(midi_meta::MidiMeta::filesize),
            SongSort::SizeDesc => {
                self.midis.sort_by_key(midi_meta::MidiMeta::filesize);
                self.midis.reverse();
            }
        }

        // Find the selected again
        if let Some(selected) = selected_song {
            for i in 0..self.midis.len() {
                if self.midis[i].filepath() == selected.filepath() {
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
            self.queue.shuffle(&mut rand::rng());

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
    pub fn remove_marked(&mut self) {
        // Songs
        for i in (0..self.midis.len()).rev() {
            if !self.midis[i].marked_for_removal {
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

        self.fonts.remove_marked();
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
            PlayerError::ModifyDirList
        ));
        assert!(matches!(
            playlist_sub.add_font("fakepath".into()).unwrap_err(),
            PlayerError::ModifyDirList
        ));
        assert_eq!(playlist_man.fonts.fonts().len(), 1);
        assert_eq!(playlist_dir.fonts.fonts().len(), 0);
        assert_eq!(playlist_sub.fonts.fonts().len(), 0);
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

        playlist_man.mark_font_for_removal(0).unwrap();
        assert!(matches!(
            playlist_dir.mark_font_for_removal(0).unwrap_err(),
            PlayerError::ModifyDirList
        ));
        assert!(matches!(
            playlist_sub.mark_font_for_removal(0).unwrap_err(),
            PlayerError::ModifyDirList
        ));
        playlist_man.remove_marked();
        playlist_dir.remove_marked();
        playlist_sub.remove_marked();

        assert_eq!(playlist_man.fonts.fonts().len(), 0);
        assert_eq!(playlist_dir.fonts.fonts().len(), 1);
        assert_eq!(playlist_sub.fonts.fonts().len(), 1);
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
            PlayerError::ModifyDirList
        ));
        assert!(matches!(
            playlist_sub.add_song("fakepath".into()).unwrap_err(),
            PlayerError::ModifyDirList
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

        playlist_man.mark_song_for_removal(0).unwrap();
        assert!(matches!(
            playlist_dir.mark_song_for_removal(0).unwrap_err(),
            PlayerError::ModifyDirList
        ));
        assert!(matches!(
            playlist_sub.mark_song_for_removal(0).unwrap_err(),
            PlayerError::ModifyDirList
        ));
        playlist_man.remove_marked();
        playlist_dir.remove_marked();
        playlist_sub.remove_marked();

        assert_eq!(playlist_man.midis.len(), 0);
        assert_eq!(playlist_dir.midis.len(), 1);
        assert_eq!(playlist_sub.midis.len(), 1);
    }

    #[test]
    fn test_add_duplicate_font() {
        let mut playlist = Playlist::default();
        playlist.add_font("fakefont_a".into()).unwrap();
        playlist.add_font("fakefont_b".into()).unwrap();
        assert!(matches!(
            playlist.add_font("fakefont_b".into()).unwrap_err(),
            PlayerError::FontAlreadyExists
        ));
    }

    #[test]
    fn test_add_duplicate_song() {
        let mut playlist = Playlist::default();
        playlist.add_song("fakesong_a".into()).unwrap();
        playlist.add_song("fakesong_b".into()).unwrap();
        assert!(matches!(
            playlist.add_song("fakesong_b".into()).unwrap_err(),
            PlayerError::MidiAlreadyExists
        ));
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
        playlist.mark_font_for_removal(0).unwrap();
        assert!(playlist.unsaved_changes);
        playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.add_song("fakepath".into()).unwrap();
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.mark_song_for_removal(0).unwrap();
        assert!(playlist.unsaved_changes);
    }

    #[test]
    fn test_unsaved_flag_fontsong_force_add_rm() {
        let mut playlist = Playlist::default();
        playlist.unsaved_changes = false;
        playlist.force_add_font("fakepath1".into()).unwrap();
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.force_mark_font_for_removal(0).unwrap();
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.force_add_song("fakepath2".into()).unwrap();
        assert!(playlist.unsaved_changes);
        playlist.unsaved_changes = false;
        playlist.force_mark_song_for_removal(0).unwrap();
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
