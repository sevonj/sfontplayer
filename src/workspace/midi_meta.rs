use std::{error, fmt, fs, path::PathBuf, time::Duration};

use rustysynth::MidiFile;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MidiMetaError {
    CantAccessFile { filename: String, message: String },
    InvalidFile { filename: String, message: String },
}
impl error::Error for MidiMetaError {}
impl fmt::Display for MidiMetaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CantAccessFile { filename, message } => {
                write!(f, "Can't access {filename}: {message}")
            }
            Self::InvalidFile { filename, message } => {
                write!(f, "{filename} is not a valid midi file: {message}")
            }
        }
    }
}

/// Reference to a midi file with metadata
#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
#[serde(default)]
pub(crate) struct MidiMeta {
    filepath: PathBuf,
    filesize: Option<u64>,
    duration: Option<Duration>,
    error: Option<MidiMetaError>,
    pub is_queued_for_deletion: bool,
}

impl MidiMeta {
    /// Create from file path
    pub fn new(filepath: PathBuf) -> Self {
        let mut this = Self {
            filepath,
            filesize: None,
            duration: None,
            error: None,
            is_queued_for_deletion: false,
        };
        this.refresh();
        this
    }

    /// Refresh file metadata
    pub fn refresh(&mut self) {
        let error;
        let mut duration = None;

        self.filesize = if let Ok(file_meta) = fs::metadata(&self.filepath) {
            Some(file_meta.len())
        } else {
            None
        };

        match fs::File::open(&self.filepath) {
            Ok(mut file) => match MidiFile::new(&mut file) {
                Ok(midifile) => {
                    duration = Some(Duration::from_secs_f64(midifile.get_length()));
                    error = None;
                }
                Err(e) => {
                    error = Some(MidiMetaError::InvalidFile {
                        filename: self.get_name(),
                        message: e.to_string(),
                    });
                }
            },
            Err(e) => {
                error = Some(MidiMetaError::CantAccessFile {
                    filename: self.get_name(),
                    message: e.to_string(),
                });
            }
        }
        self.duration = duration;
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
    pub fn get_duration(&self) -> Option<Duration> {
        self.duration
    }
    pub fn get_size(&self) -> Option<u64> {
        self.filesize
    }
    pub fn get_error(&self) -> Option<MidiMetaError> {
        self.error.clone()
    }
}
