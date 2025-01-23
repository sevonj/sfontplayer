use midi_msg::MidiFileParseError;
use rustysynth::SoundFontError;
use std::{error, fmt, path::PathBuf};

#[derive(Debug)]
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
    AudioRenderError,

    DebugBlockSaving,
    MidiOverride,

    FontlibPathIndex { index: usize },
    FontlibPathAlreadyExists { path: PathBuf },
    FontlibNoSuchFont { filepath: PathBuf },

    ModifyDirList,
    UnknownFileFormat { path: PathBuf },
    PathDoesntExist { path: PathBuf },
    IoError { source: std::io::Error },

    FontAlreadyExists,
    FontIndex { index: usize },
    FontMetaParse,
    FontFileError { msg: String },

    MidiAlreadyExists,
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
            Self::AudioRenderError => write!(f, "Render failed."),

            Self::DebugBlockSaving => write!(f, "debug_block_saving == true"),
            Self::MidiOverride => write!(f, "Action blocked by MIDI file override."),

            Self::FontlibPathIndex { index } => {
                write!(f, "Soundfont library path index {index} is out of range.")
            }
            Self::FontlibPathAlreadyExists { path } => {
                write!(f, "This path is already in the library: {path:?}")
            }
            Self::FontlibNoSuchFont { filepath: path } => {
                write!(f, "No such font in library: {path:?}")
            }

            Self::ModifyDirList => write!(f, "Cant modify a directory-tracking list manually."),
            Self::UnknownFileFormat { path } => write!(f, "Unknown file format: {path:?}."),
            Self::PathDoesntExist { path } => write!(f, "Path doesn't exist: {path:?}."),
            Self::IoError { source } => source.fmt(f),

            Self::FontAlreadyExists => write!(f, "This soundfont is already in the list."),
            Self::FontIndex { index } => write!(f, "Soundfont index {index} is out of range."),
            Self::FontMetaParse => write!(f, "Failed to parse imported soundfont meta."),
            Self::FontFileError { msg } => write!(f, "Invalid soundfont: {msg}."),

            Self::MidiAlreadyExists => write!(f, "This MIDI file is already in the list."),
            Self::MidiIndex { index } => write!(f, "MIDI file index {index} is out of range."),
            Self::MidiMetaParse => write!(f, "Failed to parse imported MIDI meta."),
            Self::MidiFileError { msg } => write!(f, "Invalid midi file: {msg}."),
        }
    }
}

impl From<SoundFontError> for PlayerError {
    fn from(source: SoundFontError) -> Self {
        Self::FontFileError {
            msg: source.to_string(),
        }
    }
}

impl From<MidiFileParseError> for PlayerError {
    fn from(source: MidiFileParseError) -> Self {
        Self::MidiFileError {
            msg: source.to_string(),
        }
    }
}

impl From<std::io::Error> for PlayerError {
    fn from(source: std::io::Error) -> Self {
        Self::IoError { source }
    }
}
