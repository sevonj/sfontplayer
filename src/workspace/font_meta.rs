use std::{error, fmt, fs, path::PathBuf};

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
                write!(f, "Can't access {}: {}", filename, message)
            }
            Self::InvalidFile { filename, message } => {
                write!(f, "{} is not a valid soundfont: {}", filename, message)
            }
        }
    }
}

/// Reference to a font file with metadata
#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
#[serde(default)]
pub(crate) struct FontMeta {
    filepath: PathBuf,
    filesize: Option<u64>,
    error: Option<FontMetaError>,
}

impl FontMeta {
    /// Create from file path
    pub fn new(filepath: PathBuf) -> Self {
        let mut this = Self {
            filepath,
            filesize: None,
            error: None,
        };
        this.refresh();
        this
    }

    /// Refresh file metadata
    pub fn refresh(&mut self) {
        self.filesize = if let Ok(file_meta) = fs::metadata(&self.filepath) {
            Some(file_meta.len())
        } else {
            None
        };

        let error;
        match fs::File::open(&self.filepath) {
            Ok(mut file) => match SoundFont::new(&mut file) {
                Ok(_) => error = None,
                Err(e) => {
                    error = Some(FontMetaError::InvalidFile {
                        filename: self.get_name(),
                        message: e.to_string(),
                    })
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
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }
    pub fn get_size(&self) -> Option<u64> {
        self.filesize
    }
    pub fn get_error(&self) -> Option<FontMetaError> {
        self.error.clone()
    }
}
