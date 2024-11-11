//! Player's built in soundfont library
//!

use serde::Serialize;
use std::{error, fmt, path::PathBuf};

use super::workspace::font_meta::{FontList, FontListError, FontMeta, FontSort};

#[derive(Debug, Clone, Serialize)]
pub enum FontLibraryError {
    PathAlreadyExists { path: PathBuf },
    PathDoesntExist { path: PathBuf },
    PathInaccessible { path: PathBuf },
    PathNotAFile { path: PathBuf },
    PathNotADir { path: PathBuf },
    IndexOutOfRange,
}
impl error::Error for FontLibraryError {}
impl fmt::Display for FontLibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathAlreadyExists { path } => {
                write!(f, "This path already exists in the library: {path:?}")
            }
            Self::PathDoesntExist { path } => {
                write!(f, "This path doesn't exist in the library: {path:?}")
            }
            Self::PathInaccessible { path } => {
                write!(f, "This path was inaccessible: {path:?}")
            }
            Self::PathNotAFile { path } => {
                write!(f, "This path wasn't a file: {path:?}")
            }
            Self::PathNotADir { path } => {
                write!(f, "This path wasn't a directory: {path:?}")
            }
            Self::IndexOutOfRange => {
                write!(f, "Path index out of range")
            }
        }
    }
}

/// The FontLibrary is a wrapper around FontList.
/// It abstracts manual font management to auto-crawling files from paths.
pub struct FontLibrary {
    paths: Vec<PathBuf>,
    delet: Vec<bool>,
    pub crawl_subdirs: bool,
    fontlist: FontList,
}
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
        self.fontlist.sort()
    }
    pub fn get_sort(&self) -> FontSort {
        self.fontlist.get_sort()
    }
    pub fn set_sort(&mut self, sort: FontSort) {
        self.fontlist.set_sort(sort)
    }
    pub fn get_fonts(&self) -> &Vec<FontMeta> {
        self.fontlist.get_fonts()
    }
    pub fn get_selected(&self) -> Option<&FontMeta> {
        self.fontlist.get_selected()
    }
    pub fn get_selected_mut(&mut self) -> Option<&mut FontMeta> {
        self.fontlist.get_selected_mut()
    }
    pub fn get_selected_index(&self) -> Option<usize> {
        self.fontlist.get_selected_index()
    }
    pub fn select(&mut self, value: Option<usize>) -> Result<(), FontListError> {
        self.fontlist.select(value)
    }
    pub fn contains_font(&self, filepath: &PathBuf) -> bool {
        self.fontlist.contains(filepath)
    }

    // --- Paths --- //

    pub fn get_paths(&self) -> &Vec<PathBuf> {
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
    pub fn add_path(&mut self, path: PathBuf) -> Result<(), FontLibraryError> {
        if self.contains_path(&path) {
            return Err(FontLibraryError::PathAlreadyExists { path });
        }
        if !path.exists() {
            return Err(FontLibraryError::PathInaccessible { path });
        }
        self.paths.push(path);
        self.delet.push(false);
        self.refresh_files();
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
    pub fn refresh_files(&mut self) {
        for path in &self.paths {
            if !path.exists() {
                continue;
            }
            if path.is_dir() {
                //
            } else if path.is_file() {
                let _ = self.fontlist.add(FontMeta::new(path.to_owned()));
            }
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
        if self.paths.len() != self.delet.len() {
            panic!("Delete queue length != paths length!")
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add_path_delete_queue() {
        let font_lib = FontLibrary::default();
    }
    
    #[test]
    fn test_rm_path_delete_queue() {
        let font_lib = FontLibrary::default();
    }
}
