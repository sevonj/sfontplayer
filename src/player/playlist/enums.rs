use std::fmt::Display;

/// Is a playlist waiting to be closed?
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlaylistState {
    /// Don't delet me pls.
    None,
    /// Death row: Queued for closing, but not until unsaved changes are resolved.
    Queued,
    /// Close at first opportunity, even if there are unsaved changes.
    QueuedDiscard,
}

/// Option for how soundfonts or midi files are managed
#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
#[repr(u8)]
pub enum FileListMode {
    /// The contents are added and removed manually.
    #[default]
    Manual = 0,
    /// The contents are fetched automatically from a directory.
    Directory = 1,
    /// The contents are fetched automatically from a directory and subdirectories.
    Subdirectories = 2,
}

impl Display for FileListMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual => write!(f, "Individual files"),
            Self::Directory => write!(f, "Directory"),
            Self::Subdirectories => write!(f, "Subdirectories"),
        }
    }
}

impl TryFrom<u8> for FileListMode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::Manual as u8 => Ok(Self::Manual),
            x if x == Self::Directory as u8 => Ok(Self::Directory),
            x if x == Self::Subdirectories as u8 => Ok(Self::Subdirectories),
            _ => Err(()),
        }
    }
}

/// Option for how songs are sorted
#[derive(PartialEq, Eq, Default, Clone, Copy, Debug)]
#[repr(u8)]
pub enum SongSort {
    #[default]
    NameAsc = 0,
    NameDesc = 1,
    TimeAsc = 2,
    TimeDesc = 3,
    SizeAsc = 4,
    SizeDesc = 5,
}

impl TryFrom<u8> for SongSort {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::NameAsc as u8 => Ok(Self::NameAsc),
            x if x == Self::NameDesc as u8 => Ok(Self::NameDesc),
            x if x == Self::TimeAsc as u8 => Ok(Self::TimeAsc),
            x if x == Self::TimeDesc as u8 => Ok(Self::TimeDesc),
            x if x == Self::SizeAsc as u8 => Ok(Self::SizeAsc),
            x if x == Self::SizeDesc as u8 => Ok(Self::SizeDesc),
            _ => Err(()),
        }
    }
}
