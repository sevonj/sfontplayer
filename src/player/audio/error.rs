use midi_msg::MidiFileParseError;
use rustysynth::SoundFontError;
use std::{fmt, path::PathBuf};

#[derive(Debug)]
pub enum AudioPlayerError {
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
        source: MidiFileParseError,
    },
}
impl fmt::Display for AudioPlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoFont => write!(f, "Audio/No soundfont!"),
            Self::NoMidi => write!(f, "Audio/No midi file!"),
            Self::NoSink => write!(f, "Audio/No sink assigned!"),
            Self::CantAccessFile { path, source } => {
                write!(f, "Audio/Can't access {path:?}: {source}")
            }
            Self::IOError { source } => {
                write!(f, "Audio/IO Error: {source}")
            }
            Self::InvalidFont { source } => {
                write!(f, "Audio/SoundFont: {source}")
            }
            Self::InvalidMidi { source } => {
                write!(f, "Audio/MIDI file: {source}")
            }
        }
    }
}
impl From<std::io::Error> for AudioPlayerError {
    fn from(source: std::io::Error) -> Self {
        Self::IOError { source }
    }
}
impl From<MidiFileParseError> for AudioPlayerError {
    fn from(source: MidiFileParseError) -> Self {
        Self::InvalidMidi { source }
    }
}
