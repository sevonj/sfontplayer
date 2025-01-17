use serde_repr::{Deserialize_repr, Serialize_repr};

/// Events from `Player` to gui
pub enum PlayerEvent {
    /// Bring window to focus
    Raise,
    /// Exit app
    Quit,
    /// Alert user through gui
    NotifyError(String),
}

/// Playback repeat setting
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Default, Clone, Copy)]
#[repr(u8)]
pub enum RepeatMode {
    /// No repeat.
    #[default]
    Disabled = 0,
    /// Loop the current queue
    Queue = 1,
    /// Keep repeating the same song
    Song = 2,
}

impl TryFrom<u8> for RepeatMode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::Disabled as u8 => Ok(Self::Disabled),
            x if x == Self::Queue as u8 => Ok(Self::Queue),
            x if x == Self::Song as u8 => Ok(Self::Song),
            _ => Err(()),
        }
    }
}
