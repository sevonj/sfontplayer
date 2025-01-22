//! Player's built in soundfont library
//!

use std::{fs, path::PathBuf};
use walkdir::WalkDir;

use super::playlist::FontMeta;
use super::soundfont_list::{FontList, FontSort};
use super::PlayerError;

/// `FontLibrary` is a wrapper around `FontList`.
/// It abstracts manual font management into paths that will be auto-crawled for files.
#[derive(Debug, Default)]
pub struct FontLibrary {
    /// List of paths to look at. Can be a file or a dir.
    paths: Vec<PathBuf>,
    /// Deletion queue.
    delet: Vec<bool>,
    /// True to search subdirs
    pub crawl_subdirs: bool,
    /// Wrapped kind of opaquely
    fontlist: FontList,
}

impl FontLibrary {
    // --- Wrap --- //

    pub fn sort(&mut self) {
        self.fontlist.sort();
    }

    pub const fn get_sort(&self) -> FontSort {
        self.fontlist.get_sort()
    }

    pub fn set_sort(&mut self, sort: FontSort) {
        self.fontlist.set_sort(sort);
    }

    pub const fn get_fonts(&self) -> &Vec<FontMeta> {
        self.fontlist.get_fonts()
    }

    pub fn get_font(&self, index: usize) -> Result<&FontMeta, PlayerError> {
        self.fontlist.get_font(index)
    }

    pub fn get_font_mut(&mut self, index: usize) -> Result<&mut FontMeta, PlayerError> {
        self.fontlist.get_font_mut(index)
    }

    pub fn get_selected(&self) -> Option<&FontMeta> {
        self.fontlist.get_selected()
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut FontMeta> {
        self.fontlist.get_selected_mut()
    }

    pub const fn get_selected_index(&self) -> Option<usize> {
        self.fontlist.get_selected_index()
    }

    pub fn select(&mut self, index: usize) -> Result<(), PlayerError> {
        self.fontlist.set_selected_index(Some(index))
    }

    pub fn contains_font(&self, filepath: &PathBuf) -> bool {
        self.fontlist.contains(filepath)
    }

    // --- Paths --- //

    pub const fn get_paths(&self) -> &Vec<PathBuf> {
        &self.paths
    }

    pub fn contains_path(&self, path: &PathBuf) -> bool {
        for existing_path in &self.paths {
            if *existing_path == *path {
                return true;
            }
        }
        false
    }

    pub fn select_by_path(&mut self, path: PathBuf) -> Result<(), PlayerError> {
        for (i, font) in self.get_fonts().iter().enumerate() {
            if font.get_path() == path {
                let _ = self.fontlist.set_selected_index(Some(i));
                return Ok(());
            }
        }
        Err(PlayerError::FontlibNoSuchFont { path })
    }

    pub fn add_path(&mut self, path: PathBuf) -> Result<(), PlayerError> {
        if self.contains_path(&path) {
            return Err(PlayerError::FontlibPathAlreadyExists { path });
        }
        if !path.exists() {
            return Err(PlayerError::PathDoesntExist { path });
        }
        self.paths.push(path);
        self.delet.push(false);
        self.refresh();
        Ok(())
    }

    pub fn remove_path(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.paths.len() {
            return Err(PlayerError::FontlibPathIndex { index });
        }

        self.delet[index] = true;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.paths.clear();
        self.delet.clear();
        self.fontlist.clear();
    }

    pub fn refresh(&mut self) {
        let mut found_files = vec![];
        let selected_font_path = self.get_selected().map(FontMeta::get_path);

        self.fontlist.clear();
        for input_path in &self.paths {
            if !input_path.exists() {
                continue;
            }
            if input_path.is_dir() {
                if self.crawl_subdirs {
                    for entry in WalkDir::new(input_path)
                        .into_iter()
                        .filter_map(std::result::Result::ok)
                    {
                        let filepath = entry.path().to_owned();
                        if filepath.is_file() && filepath.extension().is_some_and(|s| s == "sf2") {
                            found_files.push(filepath);
                        }
                    }
                } else if let Ok(paths) = fs::read_dir(input_path) {
                    for entry in paths.filter_map(std::result::Result::ok) {
                        let filepath = entry.path().clone();
                        if self.contains_font(&filepath) {
                            continue;
                        }
                        if filepath.is_file() && filepath.extension().is_some_and(|s| s == "sf2") {
                            found_files.push(filepath);
                        }
                    }
                }
            } else if input_path.is_file() {
                found_files.push(input_path.to_owned());
            }
        }

        for filepath in found_files {
            let _ = self.fontlist.add(FontMeta::new(filepath));
        }

        if let Some(path) = selected_font_path {
            let _ = self.fontlist.set_selected_index(None);
            let _ = self.select_by_path(path);
        }

        self.sort();
    }

    pub fn update(&mut self) {
        self.assert_delete_queue_len();
        self.delete_queued();
    }

    fn delete_queued(&mut self) {
        for index in (0..self.paths.len()).rev() {
            if self.delet[index] {
                self.paths.remove(index);
                self.delet.remove(index);
            }
        }
    }

    fn assert_delete_queue_len(&self) {
        assert_eq!(
            self.paths.len(),
            self.delet.len(),
            "Sanity check failed: Delete queue length != paths length!"
        );
    }
}
