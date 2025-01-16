use std::{error, fmt};

use super::{audio::error::AudioPlayerError, playlist::MetaError};

#[derive(Debug, PartialEq, Eq)]
pub enum PlayerError {
    InvalidPlaylistIndex { index: usize },
    InvalidMidiIndex,
    CantMovePlaylist,
    CantSwitchPlaylist,
    NoQueueIndex,
    NoSoundfont,
    PlaylistAlreadyOpen,
    PlaylistOpenFailed,
    PlaylistSaveFailed,
    DebugBlockSaving,
    MidiOverride,
    Meta(MetaError),
    AudioBackendError,
}

impl error::Error for PlayerError {}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlaylistIndex { index } => {
                write!(f, "Playlist index {index} is out of bounds.")
            }
            Self::InvalidMidiIndex => write!(f, "Invalid midi index"),
            Self::CantMovePlaylist => write!(f, "Can't move this playlist further."),
            Self::CantSwitchPlaylist => write!(f, "Can't switch playlists further."),
            Self::NoQueueIndex => write!(f, "No queue index!"),
            Self::NoSoundfont => write!(f, "No soundfont!"),
            Self::PlaylistAlreadyOpen => write!(f, "Playlist is already open."),
            Self::PlaylistOpenFailed => write!(f, "Failed to open playlist."),
            Self::PlaylistSaveFailed => write!(f, "Failed to save playlist."),
            Self::DebugBlockSaving => write!(f, "debug_block_saving == true"),
            Self::MidiOverride => write!(f, "Blocked by MIDI file override"),
            Self::Meta(source) => source.fmt(f),
            Self::AudioBackendError => write!(f, "Error in audio player."),
        }
    }
}

impl From<AudioPlayerError> for PlayerError {
    fn from(_: AudioPlayerError) -> Self {
        Self::AudioBackendError
    }
}

impl From<MetaError> for PlayerError {
    fn from(e: MetaError) -> Self {
        Self::Meta(e)
    }
}
