use std::{error, fmt, fs, path::PathBuf};

use anyhow::bail;
use rustysynth::SoundFont;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
#[serde(default)]
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
