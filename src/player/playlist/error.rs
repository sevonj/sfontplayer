//! Playlist errors

use std::{error::Error, fmt, path::PathBuf};

use crate::player::soundfont_list::FontListError;

#[derive(Debug, Clone)]
pub enum PlaylistError {
    InvalidIndex,
    AlreadyExists,
    ModifyDirList,
    UnknownFileFormat { path: PathBuf },
    FailedToOpen { path: PathBuf },
}

impl Error for PlaylistError {}

impl fmt::Display for PlaylistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIndex => {
                write!(f, "Index out of range")
            }
            Self::AlreadyExists => {
                write!(f, "Already in playlist")
            }
            Self::ModifyDirList => {
                write!(f, "Cant modify a directory-tracking list manually.")
            }
            Self::UnknownFileFormat { path } => write!(f, "Unknown file format: {path:?}"),
            Self::FailedToOpen { path } => write!(f, "Failed to open file: {path:?}"),
        }
    }
}

impl From<FontListError> for PlaylistError {
    fn from(value: FontListError) -> Self {
        match value {
            FontListError::AlreadyExists => Self::AlreadyExists,
            FontListError::IndexOutOfRange => Self::InvalidIndex,
        }
    }
}
