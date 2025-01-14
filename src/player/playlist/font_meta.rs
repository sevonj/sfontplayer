use std::{error, fmt, fs, path::PathBuf};

use anyhow::bail;
use rustysynth::SoundFont;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
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
    pub fn get_status(&self) -> Result<(), FontMetaError> {
        if let Some(e) = &self.error {
            return Err(e.clone());
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
    use crate::player::playlist::Playlist;
    use serde_json::Value;

    fn run_serialize(playlist: Playlist) -> Playlist {
        Playlist::from(Value::from(&playlist))
    }

    #[test]
    fn test_serialize_filepath() {
        let mut playlist = Playlist::default();
        let font = FontMeta {
            filepath: "Fakepath".into(),
            ..Default::default()
        };
        playlist.fonts.push(font);
        let new_playlist = run_serialize(playlist);
        assert_eq!(
            new_playlist.fonts[0].get_path().to_str().unwrap(),
            "Fakepath"
        );
    }

    #[test]
    fn test_serialize_filesize() {
        let mut playlist = Playlist::default();
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
        playlist.fonts.push(font_none);
        playlist.fonts.push(font_420);
        let new_playlist = run_serialize(playlist);
        assert_eq!(new_playlist.fonts[0].get_size(), None);
        assert_eq!(new_playlist.fonts[1].get_size().unwrap(), 420);
    }
}
