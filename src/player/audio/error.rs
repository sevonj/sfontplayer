use std::{fmt, path::PathBuf};

use midi_msg::MidiFileParseError;
use rustysynth::{MidiFileError, SoundFontError};

#[derive(Debug)]
pub enum PlayerError {
    NoFont,
    NoMidi,
    NoSink,
    CantAccessFile {
        path: PathBuf,
        source: std::io::Error,
    },
    IOError {
        source: std::io::Error,
    },
    InvalidFont {
        source: SoundFontError,
    },
    InvalidMidi {
        source: MidiFileError,
    },
    InvalidMidi2 {
        source: MidiFileParseError,
    },
}
impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFont => write!(f, "No soundfont!"),
            Self::NoMidi => write!(f, "No midi file!"),
            Self::NoSink => write!(f, "No audio sink assigned!"),
            Self::CantAccessFile { path, source } => {
                write!(f, "Can't access {path:?}: {source}")
            }
            Self::IOError { source } => {
                write!(f, "IO Error: {source}")
            }
            Self::InvalidFont { source } => {
                write!(f, "Invalid soundfont: {source}")
            }
            Self::InvalidMidi { source } => {
                write!(f, "Invalid midi file: {source}")
            }
            Self::InvalidMidi2 { source } => {
                write!(f, "Invalid midi file: {source}")
            }
        }
    }
}
impl From<std::io::Error> for PlayerError {
    fn from(source: std::io::Error) -> Self {
        Self::IOError { source }
    }
}
impl From<MidiFileParseError> for PlayerError {
    fn from(source: MidiFileParseError) -> Self {
        Self::InvalidMidi2 { source }
    }
}
