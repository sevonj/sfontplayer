use std::{fmt, path::PathBuf};

use rustysynth::{MidiFileError, SoundFontError};

#[derive(Debug)]
pub enum PlayerError {
    NoFont,
    NoMidi,
    CantAccessFile {
        path: PathBuf,
        source: std::io::Error,
    },
    InvalidFont {
        source: SoundFontError,
    },
    InvalidMidi {
        source: MidiFileError,
    },
}
impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFont => write!(f, "No soundfont!"),
            Self::NoMidi => write!(f, "No midi file!"),
            Self::CantAccessFile { path, source } => {
                write!(f, "Can't access {path:?}: {source}")
            }
            Self::InvalidFont { source } => {
                write!(f, "Invalid soundfont: {source}")
            }
            Self::InvalidMidi { source } => {
                write!(f, "Invalid midi file: {source}")
            }
        }
    }
}
