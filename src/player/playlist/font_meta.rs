use rustysynth::SoundFont;
use serde::Serialize;
use std::{
    fs::{self, File},
    path::PathBuf,
};

use crate::player::PlayerError;

/// Reference to a font file with metadata
#[derive(Debug, Default, Clone, Serialize)]
pub struct FontMeta {
    filepath: PathBuf,
    filesize: Option<u64>,
    #[serde(skip)]
    error: Option<PlayerError>,
    pub is_queued_for_deletion: bool,
}

impl FontMeta {
    /// Create from file path
    pub fn new(filepath: PathBuf) -> Self {
        let mut this = Self {
            filepath,
            filesize: None,
            error: None,
            is_queued_for_deletion: false,
        };
        this.refresh();
        this
    }

    /// Refresh file metadata
    pub fn refresh(&mut self) {
        self.filesize =
            fs::metadata(&self.filepath).map_or(None, |file_meta| Some(file_meta.len()));

        self.error = match self.get_soundfont() {
            Err(e) => Some(e),
            Ok(_) => None,
        }
    }

    pub fn get_soundfont(&self) -> Result<SoundFont, PlayerError> {
        let Ok(mut fontfile) = File::open(self.get_path()) else {
            return Err(PlayerError::PathInaccessible {
                path: self.get_path(),
            });
        };
        Ok(SoundFont::new(&mut fontfile)?)
    }

    pub fn get_path(&self) -> PathBuf {
        self.filepath.clone()
    }

    pub fn set_path(&mut self, filepath: PathBuf) {
        self.filepath = filepath;
    }

    pub fn get_name(&self) -> String {
        self.filepath
            .file_name()
            .expect("No filename")
            .to_str()
            .expect("Invalid filename")
            .to_owned()
    }

    pub const fn get_size(&self) -> Option<u64> {
        self.filesize
    }

    pub fn get_status(&self) -> Result<(), PlayerError> {
        if let Some(e) = &self.error {
            return Err(e.clone());
        }
        Ok(())
    }
}

impl TryFrom<&serde_json::Value> for FontMeta {
    type Error = PlayerError;

    fn try_from(json: &serde_json::Value) -> Result<Self, Self::Error> {
        let Some(path_str) = json["filepath"].as_str() else {
            return Err(PlayerError::FontMetaParse);
        };
        let filesize = json["filesize"].as_u64();

        Ok(Self {
            filepath: path_str.into(),
            filesize,
            error: None,
            is_queued_for_deletion: false,
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
        let font = FontMeta {
            filepath: "Fakepath".into(),
            ..Default::default()
        };
        playlist.fonts.add(font).unwrap();
        let new_playlist = run_serialize(playlist);
        let font = new_playlist.get_font(0).unwrap();
        assert_eq!(font.get_path().to_str().unwrap(), "Fakepath");
    }

    #[test]
    fn test_serialize_filesize() {
        let mut playlist = Playlist::default();
        let font_none = FontMeta {
            filepath: "unused".into(),
            filesize: None,
            ..Default::default()
        };
        let font_420 = FontMeta {
            filepath: "unused2".into(),
            filesize: Some(420),
            ..Default::default()
        };
        playlist.fonts.add(font_none).unwrap();
        playlist.fonts.add(font_420).unwrap();
        let new_playlist = run_serialize(playlist);
        let font_0 = new_playlist.get_font(0).unwrap();
        let font_1 = new_playlist.get_font(1).unwrap();
        assert_eq!(font_0.get_size(), None);
        assert_eq!(font_1.get_size().unwrap(), 420);
    }
}
