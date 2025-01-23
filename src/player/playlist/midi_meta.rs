use rustysynth::MidiFile;
use serde::Serialize;
use std::{fs, path::PathBuf, time::Duration};

use crate::player::PlayerError;

/// Reference to a MIDI file with metadata
#[derive(Debug, Default, Clone, Serialize)]
pub struct MidiMeta {
    filepath: PathBuf,
    filesize: Option<u64>,
    duration: Option<Duration>,
    #[serde(skip)]
    error: Option<String>,
    pub marked_for_removal: bool,
}

impl MidiMeta {
    /// Create from file path
    pub fn new(filepath: PathBuf) -> Self {
        let mut this = Self {
            filepath,
            filesize: None,
            duration: None,
            error: None,
            marked_for_removal: false,
        };
        this.refresh();
        this
    }

    /// Refresh file metadata
    pub fn refresh(&mut self) {
        let error;
        let mut duration = None;

        self.filesize =
            fs::metadata(&self.filepath).map_or(None, |file_meta| Some(file_meta.len()));

        match fs::File::open(&self.filepath) {
            Ok(mut file) => match MidiFile::new(&mut file) {
                Ok(midifile) => {
                    duration = Some(Duration::from_secs_f64(midifile.get_length()));
                    error = None;
                }
                Err(e) => {
                    error = Some(e.to_string());
                }
            },
            Err(e) => {
                error = Some(e.to_string());
            }
        }
        self.duration = duration;
        self.error = error;
    }

    /// Load the MIDI file
    pub fn fetch_midifile(&self) -> Result<midi_msg::MidiFile, PlayerError> {
        let bytes = fs::read(self.filepath())?;
        Ok(midi_msg::MidiFile::from_midi(bytes.as_slice())?)
    }

    pub const fn filepath(&self) -> &PathBuf {
        &self.filepath
    }

    pub fn set_filepath(&mut self, filepath: PathBuf) {
        self.filepath = filepath;
    }

    pub fn filename(&self) -> String {
        self.filepath
            .file_name()
            .expect("No filename")
            .to_str()
            .expect("Invalid filename")
            .to_owned()
    }

    /// Duration of the MIDI file
    pub const fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// Size of the MIDI file
    pub const fn filesize(&self) -> Option<u64> {
        self.filesize
    }

    /// Is the actual MIDI file accessible and OK?
    pub fn status(&self) -> Result<(), PlayerError> {
        if let Some(e) = &self.error {
            return Err(PlayerError::MidiFileError { msg: e.to_owned() });
        }
        Ok(())
    }
}

impl TryFrom<&serde_json::Value> for MidiMeta {
    type Error = PlayerError;

    fn try_from(json: &serde_json::Value) -> Result<Self, Self::Error> {
        let Some(path_str) = json["filepath"].as_str() else {
            return Err(PlayerError::MidiMetaParse);
        };
        let filesize = json["filesize"].as_u64();
        let duration = json["duration"]["secs"].as_u64().map(Duration::from_secs);

        Ok(Self {
            filepath: path_str.into(),
            filesize,
            duration,
            error: None,
            marked_for_removal: false,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::player::playlist::Playlist;
    use serde_json::Value;

    fn run_serialize(playlist: Playlist) -> Playlist {
        Playlist::from(Value::from(&playlist))
    }

    #[test]
    fn test_serialize_filepath() {
        let mut playlist = Playlist::default();
        let song = MidiMeta {
            filepath: "Fakepath".into(),
            ..Default::default()
        };
        playlist.midis.push(song);
        let new_playlist = run_serialize(playlist);
        assert_eq!(
            new_playlist.midis[0].filepath().to_str().unwrap(),
            "Fakepath"
        );
    }

    #[test]
    fn test_serialize_filesize() {
        let mut playlist = Playlist::default();
        let song_none = MidiMeta {
            filepath: "unused".into(),
            filesize: None,
            ..Default::default()
        };
        let song_420 = MidiMeta {
            filepath: "unused".into(),
            filesize: Some(420),
            ..Default::default()
        };
        playlist.midis.push(song_none);
        playlist.midis.push(song_420);
        let new_playlist = run_serialize(playlist);
        assert_eq!(new_playlist.midis[0].filesize(), None);
        assert_eq!(new_playlist.midis[1].filesize().unwrap(), 420);
    }

    #[test]
    fn test_serialize_duration() {
        let mut playlist = Playlist::default();
        let song_none = MidiMeta {
            filepath: "unused".into(),
            duration: None,
            ..Default::default()
        };
        let song_420 = MidiMeta {
            filepath: "unused".into(),
            duration: Some(Duration::from_secs(420)),
            ..Default::default()
        };
        playlist.midis.push(song_none);
        playlist.midis.push(song_420);
        let new_playlist = run_serialize(playlist);
        assert_eq!(new_playlist.midis[0].duration(), None);
        assert_eq!(
            new_playlist.midis[1].duration().unwrap(),
            Duration::from_secs(420)
        );
    }
}
