//! Player's built in soundfont library
//!

use serde::Serialize;
use std::{error, fmt, path::PathBuf};

use super::workspace::font_meta::{FontList, FontListError, FontMeta};

#[derive(Debug, Clone, Serialize)]
pub enum FontLibraryError {
    PathAlreadyExists { path: PathBuf },
    PathDoesntExist { path: PathBuf },
    PathInaccessible { path: PathBuf },
    PathNotAFile { path: PathBuf },
    PathNotADir { path: PathBuf },
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
        }
    }
}

/// The FontLibrary is a wrapper around FontList.
/// It abstracts manual font management to auto-crawling files from paths.
pub struct FontLibrary {
    input_files: Vec<PathBuf>,
    input_dirs: Vec<PathBuf>,
    crawl_subdirs: bool,
    fontlist: FontList,
}
impl Default for FontLibrary {
    fn default() -> Self {
        Self {
            input_files: vec![],
            input_dirs: vec![],
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
    pub fn get_fonts(&self) -> &Vec<FontMeta> {
        self.fontlist.get_fonts()
    }
    pub fn get_selected(&self) -> Option<&FontMeta> {
        self.fontlist.get_selected()
    }
    pub fn get_selected_mut(&mut self) -> Option<&mut FontMeta> {
        self.fontlist.get_selected_mut()
    }
    pub fn select(&mut self, value: Option<usize>) -> Result<(), FontListError> {
        self.fontlist.select(value)
    }

    // --- Paths --- //

    pub fn get_file_paths(&self) -> &Vec<PathBuf> {
        &self.input_files
    }
    pub fn get_dir_paths(&self) -> &Vec<PathBuf> {
        &self.input_dirs
    }
    pub fn contains(&self, path: &PathBuf) -> bool {
        for existing_path in &self.input_files {
            if *existing_path == *path {
                return true;
            }
        }
        for existing_path in &self.input_dirs {
            if *existing_path == *path {
                return true;
            }
        }
        false
    }
    pub fn add_file(&mut self, path: PathBuf) -> Result<(), FontLibraryError> {
        if self.contains(&path) {
            return Err(FontLibraryError::PathAlreadyExists { path });
        }
        if !path.exists() {
            return Err(FontLibraryError::PathDoesntExist { path });
        }
        if !path.is_file() {
            return Err(FontLibraryError::PathNotAFile { path });
        }
        self.input_files.push(path);
        Ok(())
    }
    pub fn add_dir(&mut self, path: PathBuf) -> Result<(), FontLibraryError> {
        if self.contains(&path) {
            return Err(FontLibraryError::PathAlreadyExists { path });
        }
        if !path.exists() {
            return Err(FontLibraryError::PathDoesntExist { path });
        }
        if !path.is_dir() {
            return Err(FontLibraryError::PathNotADir { path });
        }
        self.input_dirs.push(path);
        Ok(())
    }
    pub fn remove_path(&mut self, path: &PathBuf) -> Result<(), FontLibraryError> {
        for i in 0..self.input_files.len() {
            if *path == self.input_files[i] {
                self.input_files.remove(i);
                return Ok(());
            }
        }
        for i in 0..self.input_dirs.len() {
            if *path == self.input_dirs[i] {
                self.input_dirs.remove(i);
                return Ok(());
            }
        }
        Err(FontLibraryError::PathDoesntExist {
            path: path.to_owned(),
        })
    }
    pub fn clear(&mut self) {
        self.clear_files();
        self.clear_dirs();
    }
    pub fn clear_files(&mut self) {
        self.input_files.clear();
    }
    pub fn clear_dirs(&mut self) {
        self.input_dirs.clear();
    }
    pub fn refresh(&mut self) {
        for input_dir in &self.input_dirs {}
        for input_file in &self.input_files {
            let _ = self.fontlist.add(FontMeta::new(input_file.to_owned()));
        }
    }
}
