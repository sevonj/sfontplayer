//! Workspace errors

use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub enum WorkspaceError {
    InvalidFontIndex { index: usize },
    InvalidSongIndex { index: usize },
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
        }
    }
}
