//! Workspace errors

use std::{error::Error, fmt, path::PathBuf};

use super::enums::FileListMode;

#[derive(Debug, Clone)]
pub enum WorkspaceError {
    InvalidFontIndex { index: usize },
    InvalidSongIndex { index: usize },
    ModifyAutoFontList { mode: FileListMode },
    ModifyAutoSongList { mode: FileListMode },
    UnknownFileFormat { path: PathBuf },
}

impl Error for WorkspaceError {}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFontIndex { index } => {
                write!(f, "Font index out of range: {index}")
            }
            Self::InvalidSongIndex { index } => {
                write!(f, "Song index out of range: {index}")
            }
            Self::ModifyAutoFontList { mode } => {
                write!(
                    f,
                    "Can't modify font list, it's in auto-managed mode: {mode:?}"
                )
            }
            Self::ModifyAutoSongList { mode } => {
                write!(
                    f,
                    "Can't modify song list, it's in auto-managed mode: {mode:?}"
                )
            }
            Self::UnknownFileFormat { path } => write!(f, "Unknown file format: {path:?}"),
        }
    }
}
