use midi_msg::MidiFileParseError;
use rustysynth::SoundFontError;
use std::{error, fmt, path::PathBuf};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PlayerError {
    PlaylistIndex { index: usize },
    PlaylistCantMove,
    PlaylistCantSwitch,
    PlaylistAlreadyOpen,
    PlaylistOpenFailed { path: PathBuf },
    PlaylistSaveFailed,

    PlaybackNoQueueIndex,
    PlaybackNoSoundfont,

    AudioNoSink,
    AudioNoFont,
    AudioNoMidi,

    DebugBlockSaving,
    MidiOverride,

    FontlibPathIndex { index: usize },
    FontlibPathAlreadyExists { path: PathBuf },
    FontlibNoSuchFont { path: PathBuf },

    ModifyDirList,
    UnknownFileFormat { path: PathBuf },
    PathInaccessible { path: PathBuf },

    FontAlreadyExists, // TODO: https://github.com/sevonj/sfontplayer/issues/271
    FontIndex { index: usize },
    FontMetaParse,
    FontFileError { msg: String },

    // TODO: MidiAlreadyExists https://github.com/sevonj/sfontplayer/issues/271
    MidiIndex { index: usize },
    MidiMetaParse,
    MidiFileError { msg: String },
}

impl error::Error for PlayerError {}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlaylistIndex { index } => write!(f, "Playlist index {index} is out of range."),
            Self::PlaylistCantMove => write!(f, "Can't move this playlist further."),
            Self::PlaylistCantSwitch => write!(f, "Can't switch playlists further."),
            Self::PlaylistAlreadyOpen => write!(f, "Playlist is already open."),
            Self::PlaylistOpenFailed { path } => write!(f, "Couldn't open playlist from {path:?}."),
            Self::PlaylistSaveFailed => write!(f, "Couldn't save playlist."),

            Self::PlaybackNoQueueIndex => write!(f, "No queue index!"),
            Self::PlaybackNoSoundfont => write!(f, "No soundfont!"),

            Self::AudioNoSink => write!(f, "Audio player has no sink?!."),
            Self::AudioNoFont => write!(f, "Audio player has no soundfont?!."),
            Self::AudioNoMidi => write!(f, "Audio player has no MIDI file?!."),

            Self::DebugBlockSaving => write!(f, "debug_block_saving == true"),
            Self::MidiOverride => write!(f, "Action blocked by MIDI file override."),

            Self::FontlibPathIndex { index } => {
                write!(f, "Soundfont library path index {index} is out of range.")
            }
            Self::FontlibPathAlreadyExists { path } => {
                write!(f, "This path is already in the library: {path:?}")
            }
            Self::FontlibNoSuchFont { path } => write!(f, "No such font in library: {path:?}"),

            Self::ModifyDirList => write!(f, "Cant modify a directory-tracking list manually."),
            Self::UnknownFileFormat { path } => write!(f, "Unknown file format: {path:?}."),
            Self::PathInaccessible { path } => write!(f, "Path inaccessible: {path:?}."),

            Self::FontAlreadyExists => write!(f, "This soundfont is already in the list."),
            Self::FontIndex { index } => write!(f, "Soundfont index {index} is out of range."),
            Self::FontMetaParse => write!(f, "Failed to parse imported soundfont meta."),
            Self::FontFileError { msg } => write!(f, "Invalid soundfont: {msg}."),

            // TODO: MidiAlreadyExists https://github.com/sevonj/sfontplayer/issues/271
            Self::MidiIndex { index } => write!(f, "MIDI file index {index} is out of range."),
            Self::MidiMetaParse => write!(f, "Failed to parse imported MIDI meta."),
            Self::MidiFileError { msg } => write!(f, "Invalid midi file: {msg}."),
        }
    }
}

impl From<SoundFontError> for PlayerError {
    fn from(e: SoundFontError) -> Self {
        Self::FontFileError { msg: e.to_string() }
    }
}

impl From<MidiFileParseError> for PlayerError {
    fn from(e: MidiFileParseError) -> Self {
        Self::MidiFileError { msg: e.to_string() }
    }
}
