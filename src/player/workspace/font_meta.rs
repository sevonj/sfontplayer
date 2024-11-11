use std::{error, fmt, fs, path::PathBuf};

use anyhow::bail;
use rustysynth::SoundFont;
use serde::Serialize;

#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
#[repr(u8)]
pub enum FontSort {
    #[default]
    NameAsc = 0,
    NameDesc = 1,
    SizeAsc = 2,
    SizeDesc = 3,
}
impl TryFrom<u8> for FontSort {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::NameAsc as u8 => Ok(Self::NameAsc),
            x if x == Self::NameDesc as u8 => Ok(Self::NameDesc),
            x if x == Self::SizeAsc as u8 => Ok(Self::SizeAsc),
            x if x == Self::SizeDesc as u8 => Ok(Self::SizeDesc),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum FontListError {
    AlreadyExists,
    IndexOutOfRange,
}
impl error::Error for FontListError {}
impl fmt::Display for FontListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists => {
                write!(f, "This soundfont already exists in the list.")
            }
            Self::IndexOutOfRange => {
                write!(f, "FontList index out of range.")
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct FontList {
    fonts: Vec<FontMeta>,
    sort: FontSort,
    selected: Option<usize>,
}

impl FontList {
    pub fn sort(&mut self) {
        // Store the selected
        let selected = if let Some(index) = self.selected {
            Some(self.fonts[index].clone())
        } else {
            None
        };
        // Sort
        match self.sort {
            FontSort::NameAsc => self.fonts.sort_by_key(|f| f.get_name().to_lowercase()),
            FontSort::NameDesc => {
                self.fonts.sort_by_key(|f| f.get_name().to_lowercase());
                self.fonts.reverse();
            }
            FontSort::SizeAsc => self.fonts.sort_by_key(FontMeta::get_size),
            FontSort::SizeDesc => {
                self.fonts.sort_by_key(FontMeta::get_size);
                self.fonts.reverse();
            }
        };
        // Find the selected again
        if let Some(selected) = selected {
            for i in 0..self.fonts.len() {
                if self.fonts[i].get_path() == selected.get_path() {
                    self.selected = Some(i);
                }
            }
        }
    }
    pub fn get_sort(&self) -> FontSort {
        self.sort
    }
    pub fn set_sort(&mut self, sort: FontSort) {
        self.sort = sort;
        self.sort();
    }
    pub fn contains(&self, filepath: &PathBuf) -> bool {
        for i in 0..self.fonts.len() {
            if self.fonts[i].get_path() == *filepath {
                return true;
            }
        }
        false
    }
    pub fn add(&mut self, font: FontMeta) -> Result<(), FontListError> {
        if self.contains(&font.get_path()) {
            return Err(FontListError::AlreadyExists);
        }
        self.fonts.push(font);
        Ok(())
    }
    pub fn remove(&mut self, index: usize) -> Result<(), FontListError> {
        if index >= self.fonts.len() {
            return Err(FontListError::IndexOutOfRange);
        }
        self.fonts.remove(index);
        Ok(())
    }
    pub fn clear(&mut self) {
        self.fonts.clear();
    }
    pub fn get_fonts(&self) -> &Vec<FontMeta> {
        &self.fonts
    }
    pub fn get(&self, index: usize) -> Result<&FontMeta, FontListError> {
        if index >= self.fonts.len() {
            return Err(FontListError::IndexOutOfRange);
        }
        Ok(&self.fonts[index])
    }
    pub fn get_mut(&mut self, index: usize) -> Result<&mut FontMeta, FontListError> {
        if index >= self.fonts.len() {
            return Err(FontListError::IndexOutOfRange);
        }
        Ok(&mut self.fonts[index])
    }
    pub fn get_selected(&self) -> Option<&FontMeta> {
        let Some(index) = self.selected else {
            return None;
        };
        Some(&self.fonts[index])
    }
    pub fn get_selected_mut(&mut self) -> Option<&mut FontMeta> {
        let Some(index) = self.selected else {
            return None;
        };
        Some(&mut self.fonts[index])
    }
    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected
    }
    pub fn select(&mut self, value: Option<usize>) -> Result<(), FontListError> {
        let Some(index) = value else {
            self.selected = None;
            return Ok(());
        };
        if index >= self.fonts.len() {
            return Err(FontListError::IndexOutOfRange);
        }
        self.selected = Some(index);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum FontMetaError {
    CantAccessFile { filename: String, message: String },
    InvalidFile { filename: String, message: String },
}
impl error::Error for FontMetaError {}
impl fmt::Display for FontMetaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CantAccessFile { filename, message } => {
                write!(f, "Can't access {filename}: {message}")
            }
            Self::InvalidFile { filename, message } => {
                write!(f, "{filename} is not a valid soundfont: {message}")
            }
        }
    }
}

/// Reference to a font file with metadata
#[derive(Debug, Default, Clone, Serialize)]
pub struct FontMeta {
    filepath: PathBuf,
    filesize: Option<u64>,
    error: Option<FontMetaError>,
    pub is_queued_for_deletion: bool,
}

impl FontMeta {
    /// Create from file path
    pub fn new(filepath: PathBuf) -> Self {
        let mut this = Self {
            filepath,
            filesize: None,
            error: None,
            is_queued_for_deletion: false,
        };
        this.refresh();
        this
    }

    /// Refresh file metadata
    pub fn refresh(&mut self) {
        self.filesize =
            fs::metadata(&self.filepath).map_or(None, |file_meta| Some(file_meta.len()));

        let error;
        match fs::File::open(&self.filepath) {
            Ok(mut file) => match SoundFont::new(&mut file) {
                Ok(_) => error = None,
                Err(e) => {
                    error = Some(FontMetaError::InvalidFile {
                        filename: self.get_name(),
                        message: e.to_string(),
                    });
                }
            },
            Err(e) => {
                error = Some(FontMetaError::CantAccessFile {
                    filename: self.get_name(),
                    message: e.to_string(),
                });
            }
        }
        self.error = error;
    }

    // --- Getters

    pub fn get_path(&self) -> PathBuf {
        self.filepath.clone()
    }
    pub fn set_path(&mut self, filepath: PathBuf) {
        self.filepath = filepath;
    }
    pub fn get_name(&self) -> String {
        self.filepath
            .file_name()
            .expect("No filename")
            .to_str()
            .expect("Invalid filename")
            .to_owned()
    }
    pub const fn get_size(&self) -> Option<u64> {
        self.filesize
    }
    pub fn get_status(&self) -> anyhow::Result<()> {
        if let Some(e) = &self.error {
            bail!(e.clone())
        }
        Ok(())
    }
}

impl TryFrom<&serde_json::Value> for FontMeta {
    type Error = anyhow::Error;

    fn try_from(json: &serde_json::Value) -> Result<Self, Self::Error> {
        let Some(path_str) = json["filepath"].as_str() else {
            bail!("No filepath.")
        };
        let filesize = json["filesize"].as_u64();

        Ok(Self {
            filepath: path_str.into(),
            filesize,
            error: None,
            is_queued_for_deletion: false,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::player::workspace::Workspace;
    use serde_json::Value;

    fn run_serialize(workspace: Workspace) -> Workspace {
        Workspace::from(Value::from(&workspace))
    }

    #[test]
    fn test_serialize_filepath() {
        let mut workspace = Workspace::default();
        let font = FontMeta {
            filepath: "Fakepath".into(),
            ..Default::default()
        };
        workspace.fonts.push(font);
        let new_workspace = run_serialize(workspace);
        assert_eq!(
            new_workspace.fonts[0].get_path().to_str().unwrap(),
            "Fakepath"
        );
    }

    #[test]
    fn test_serialize_filesize() {
        let mut workspace = Workspace::default();
        let font_none = FontMeta {
            filepath: "unused".into(),
            filesize: None,
            ..Default::default()
        };
        let font_420 = FontMeta {
            filepath: "unused".into(),
            filesize: Some(420),
            ..Default::default()
        };
        workspace.fonts.push(font_none);
        workspace.fonts.push(font_420);
        let new_workspace = run_serialize(workspace);
        assert_eq!(new_workspace.fonts[0].get_size(), None);
        assert_eq!(new_workspace.fonts[1].get_size().unwrap(), 420);
    }
}
