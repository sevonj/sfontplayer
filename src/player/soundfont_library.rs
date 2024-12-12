//! Player's built in soundfont library
//!

use serde::Serialize;
use std::{error, fmt, fs, path::PathBuf};
use walkdir::WalkDir;

use super::{
    playlist::font_meta::FontMeta,
    soundfont_list::{FontList, FontListError, FontSort},
};

#[derive(Debug, Clone, Serialize)]
pub enum FontLibraryError {
    PathAlreadyExists { path: PathBuf },
    PathInaccessible { path: PathBuf },
    NoSuchFont { path: PathBuf },
    IndexOutOfRange,
}
impl error::Error for FontLibraryError {}
impl fmt::Display for FontLibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathAlreadyExists { path } => {
                write!(f, "This path already exists in the library: {path:?}")
            }
            Self::PathInaccessible { path } => {
                write!(f, "This path was inaccessible: {path:?}")
            }
            Self::NoSuchFont { path } => {
                write!(f, "No such font: {path:?}")
            }
            Self::IndexOutOfRange => {
                write!(f, "Path index out of range")
            }
        }
    }
}

/// `FontLibrary` is a wrapper around `FontList`.
/// It abstracts manual font management into paths that will be auto-crawled for files.
pub struct FontLibrary {
    paths: Vec<PathBuf>,
    delet: Vec<bool>,
    pub crawl_subdirs: bool,
    fontlist: FontList,
}
#[allow(clippy::derivable_impls)]
impl Default for FontLibrary {
    fn default() -> Self {
        Self {
            paths: vec![],
            delet: vec![],
            crawl_subdirs: false,
            fontlist: FontList::default(),
        }
    }
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
    pub fn get_font(&self, index: usize) -> Result<&FontMeta, FontListError> {
        self.fontlist.get_font(index)
    }
    pub fn get_font_mut(&mut self, index: usize) -> Result<&mut FontMeta, FontListError> {
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
    pub fn select(&mut self, value: Option<usize>) -> Result<(), FontListError> {
        self.fontlist.select(value)
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
    pub fn select_by_path(&mut self, path: PathBuf) -> Result<(), FontLibraryError> {
        for (i, font) in self.get_fonts().iter().enumerate() {
            if font.get_path() == path {
                let _ = self.fontlist.select(Some(i));
                return Ok(());
            }
        }
        Err(FontLibraryError::NoSuchFont { path })
    }
    pub fn add_path(&mut self, path: PathBuf) -> Result<(), FontLibraryError> {
        if self.contains_path(&path) {
            return Err(FontLibraryError::PathAlreadyExists { path });
        }
        if !path.exists() {
            return Err(FontLibraryError::PathInaccessible { path });
        }
        self.paths.push(path);
        self.delet.push(false);
        self.refresh();
        Ok(())
    }
    pub fn remove_path(&mut self, index: usize) -> Result<(), FontLibraryError> {
        if index >= self.paths.len() {
            return Err(FontLibraryError::IndexOutOfRange);
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
                        let filepath = entry.path().to_owned();
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
            let _ = self.select(None);
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
