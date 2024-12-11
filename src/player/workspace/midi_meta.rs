use std::{error, fmt, fs, path::PathBuf, time::Duration};

use anyhow::bail;
use rustysynth::MidiFile;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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
#[derive(Default, Clone, Serialize)]
pub struct MidiMeta {
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

        self.filesize =
            fs::metadata(&self.filepath).map_or(None, |file_meta| Some(file_meta.len()));

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
    pub const fn get_duration(&self) -> Option<Duration> {
        self.duration
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

impl TryFrom<&serde_json::Value> for MidiMeta {
    type Error = anyhow::Error;

    fn try_from(json: &serde_json::Value) -> Result<Self, Self::Error> {
        let Some(path_str) = json["filepath"].as_str() else {
            bail!("No filepath.")
        };
        let filesize = json["filesize"].as_u64();
        let duration = json["duration"]["secs"].as_u64().map(Duration::from_secs);

        Ok(Self {
            filepath: path_str.into(),
            filesize,
            duration,
            error: None,
            is_queued_for_deletion: false,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::player::workspace::Playlist;
    use serde_json::Value;

    fn run_serialize(playlist: Playlist) -> Playlist {
        Playlist::from(Value::from(&playlist))
    }

    #[test]
    fn test_serialize_filepath() {
        let mut playlist = Playlist::default();
        let song = MidiMeta {
            filepath: "Fakepath".into(),
            ..Default::default()
        };
        playlist.midis.push(song);
        let new_playlist = run_serialize(playlist);
        assert_eq!(
            new_playlist.midis[0].get_path().to_str().unwrap(),
            "Fakepath"
        );
    }

    #[test]
    fn test_serialize_filesize() {
        let mut playlist = Playlist::default();
        let song_none = MidiMeta {
            filepath: "unused".into(),
            filesize: None,
            ..Default::default()
        };
        let song_420 = MidiMeta {
            filepath: "unused".into(),
            filesize: Some(420),
            ..Default::default()
        };
        playlist.midis.push(song_none);
        playlist.midis.push(song_420);
        let new_playlist = run_serialize(playlist);
        assert_eq!(new_playlist.midis[0].get_size(), None);
        assert_eq!(new_playlist.midis[1].get_size().unwrap(), 420);
    }

    #[test]
    fn test_serialize_duration() {
        let mut playlist = Playlist::default();
        let song_none = MidiMeta {
            filepath: "unused".into(),
            duration: None,
            ..Default::default()
        };
        let song_420 = MidiMeta {
            filepath: "unused".into(),
            duration: Some(Duration::from_secs(420)),
            ..Default::default()
        };
        playlist.midis.push(song_none);
        playlist.midis.push(song_420);
        let new_playlist = run_serialize(playlist);
        assert_eq!(new_playlist.midis[0].get_duration(), None);
        assert_eq!(
            new_playlist.midis[1].get_duration().unwrap(),
            Duration::from_secs(420)
        );
    }
}
