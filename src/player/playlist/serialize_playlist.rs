//! Playlist (de)serialization Into / From JSON.
//!

use std::{convert::Into, fs::File, io::Write, path::PathBuf};

use crate::player::{soundfont_list::FontList, PlayerError};

use super::{enums::FileListMode, font_meta::FontMeta, midi_meta::MidiMeta, Playlist};
use anyhow::bail;
use relative_path::{PathExt, RelativePath};
use serde_json::{json, Value};

// Reference because we don't want to consume the playlist during autosave.
impl From<&Playlist> for Value {
    fn from(playlist: &Playlist) -> Self {
        playlist.get_portable_path().map_or_else(
            || {
                // Normal playlist: save as is
                json! ({"name": playlist.name,

                     "fonts": playlist.fonts.get_fonts(),
                     "font_list_mode": playlist.font_list_mode as u8,
                     "font_dir": playlist.font_dir,

                     "songs": playlist.midis,
                     "song_list_mode": playlist.song_list_mode as u8,
                     "song_dir": playlist.midi_dir,
                    }
                )
            },
            |root| {
                // Portable file: translate all paths into relative

                let mut fonts = playlist.fonts.get_fonts().clone();
                for font in &mut fonts {
                    let absolute_path = font.get_path();
                    if let Ok(relative_path) = absolute_path.relative_to(&root) {
                        font.set_path(relative_path.to_path("."));
                    }
                }
                let mut songs = playlist.midis.clone();
                for song in &mut songs {
                    let absolute_path = song.get_path();
                    if let Ok(relative_path) = absolute_path.relative_to(&root) {
                        song.set_path(relative_path.to_path("."));
                    }
                }
                let font_dir = playlist.font_dir.as_ref().and_then(|dir| {
                    dir.relative_to(&root)
                        .map_or(None, |relative_path| Some(relative_path.to_path(".")))
                });
                let song_dir = playlist.midi_dir.as_ref().and_then(|dir| {
                    dir.relative_to(&root)
                        .map_or(None, |relative_path| Some(relative_path.to_path(".")))
                });

                json! ({
                     "name": playlist.name,

                     "fonts": fonts,
                     "font_list_mode": playlist.font_list_mode as u8,
                     "font_dir": font_dir,

                     "songs": songs,
                     "song_list_mode": playlist.song_list_mode as u8,
                     "song_dir": song_dir,
                    }
                )
            },
        )
    }
}

impl From<Value> for Playlist {
    /// Deserialize from json.
    fn from(value: Value) -> Self {
        let mut playlist = Self {
            name: value["name"].as_str().unwrap_or("Name Missing!").into(),

            fonts: FontList::default(),
            font_list_mode: value["font_list_mode"]
                .as_u64()
                .map_or_else(FileListMode::default, |int| {
                    FileListMode::try_from(int as u8).unwrap_or_default()
                }),
            font_dir: value["font_dir"].as_str().map(Into::into),

            midis: vec![],
            song_list_mode: value["song_list_mode"]
                .as_u64()
                .map_or_else(FileListMode::default, |int| {
                    FileListMode::try_from(int as u8).unwrap_or_default()
                }),
            midi_dir: value["song_dir"].as_str().map(Into::into),

            ..Default::default()
        };

        if let Some(fonts_json) = value["fonts"].as_array() {
            for data in fonts_json {
                let fontmeta = if let Ok(meta) = FontMeta::try_from(data) {
                    meta
                } else {
                    // Fallback
                    let Some(meta) = data["filepath"]
                        .as_str()
                        .map(|path| FontMeta::new(path.into()))
                    else {
                        continue;
                    };
                    meta
                };
                let _ = playlist.fonts.add(fontmeta);
            }
        }

        if let Some(songs_json) = value["songs"].as_array() {
            for data in songs_json {
                let midimeta = if let Ok(meta) = MidiMeta::try_from(data) {
                    meta
                } else {
                    // Fallback
                    let Some(meta) = data["filepath"]
                        .as_str()
                        .map(|path| MidiMeta::new(path.into()))
                    else {
                        continue;
                    };
                    meta
                };
                playlist.midis.push(midimeta);
            }
        }

        playlist
    }
}

impl Playlist {
    pub fn open_portable(filepath: PathBuf) -> Result<Self, PlayerError> {
        let Ok(json_str) = std::fs::read_to_string(&filepath) else {
            return Err(PlayerError::PlaylistOpenFailed { path: filepath });
        };
        let Ok(data): Result<Value, _> = serde_json::from_str(&json_str) else {
            return Err(PlayerError::PlaylistOpenFailed { path: filepath });
        };

        let mut playlist = Self::from(data);

        // Make paths absolute
        let root: &PathBuf = &filepath;
        for i in 0..playlist.get_fonts().len() {
            let Ok(font) = playlist.get_font_mut(i) else {
                continue;
            };
            if let Ok(relative_path) = RelativePath::from_path(&font.get_path()) {
                font.set_path(relative_path.to_logical_path(root));
            };
        }
        for song in &mut playlist.midis {
            if let Ok(relative_path) = RelativePath::from_path(&song.get_path()) {
                song.set_path(relative_path.to_logical_path(root));
            };
        }
        if let Some(dir) = &playlist.font_dir {
            if let Ok(relative_path) = RelativePath::from_path(dir) {
                playlist.font_dir = Some(relative_path.to_logical_path(root));
            };
        }
        if let Some(dir) = &playlist.midi_dir {
            if let Ok(relative_path) = RelativePath::from_path(dir) {
                playlist.midi_dir = Some(relative_path.to_logical_path(root));
            };
        }

        playlist.portable_filepath = Some(filepath);
        playlist.unsaved_changes = false;

        Ok(playlist)
    }

    /// Save function for portable playlists.
    // The weird scope is here to block accidentally calling it form gui.
    pub(in super::super) fn save_portable(&mut self) -> anyhow::Result<()> {
        let Some(filepath) = self.get_portable_path() else {
            let name = &self.name;
            bail!("Can't save non-portable playlist as a portable file. Make it portable first! name:{name}")
        };

        let mut playlist_file = File::create(&filepath)?;
        let self_immutable = &*self;
        playlist_file.write_all(Value::from(self_immutable).to_string().as_bytes())?;

        self.unsaved_changes = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    //! These tests convert data into JSON and back, and then assert that it's unchanged.

    use std::fs;

    use super::*;

    fn run_serialize(playlist: Playlist) -> Playlist {
        Playlist::from(Value::from(&playlist))
    }

    #[test]
    fn test_fontlistmode() {
        let mut playlist_man = Playlist::default();
        let mut playlist_dir = Playlist::default();
        let mut playlist_sub = Playlist::default();
        playlist_man.font_list_mode = FileListMode::Manual;
        playlist_dir.font_list_mode = FileListMode::Directory;
        playlist_sub.font_list_mode = FileListMode::Subdirectories;
        let new_playlist_man = run_serialize(playlist_man);
        let new_playlist_dir = run_serialize(playlist_dir);
        let new_playlist_sub = run_serialize(playlist_sub);
        assert_eq!(new_playlist_man.font_list_mode, FileListMode::Manual);
        assert_eq!(new_playlist_dir.font_list_mode, FileListMode::Directory);
        assert_eq!(
            new_playlist_sub.font_list_mode,
            FileListMode::Subdirectories
        );
    }

    #[test]
    fn test_songlistmode() {
        let mut playlist_man = Playlist::default();
        let mut playlist_dir = Playlist::default();
        let mut playlist_sub = Playlist::default();
        playlist_man.song_list_mode = FileListMode::Manual;
        playlist_dir.song_list_mode = FileListMode::Directory;
        playlist_sub.song_list_mode = FileListMode::Subdirectories;
        let new_playlist_man = run_serialize(playlist_man);
        let new_playlist_dir = run_serialize(playlist_dir);
        let new_playlist_sub = run_serialize(playlist_sub);
        assert_eq!(new_playlist_man.song_list_mode, FileListMode::Manual);
        assert_eq!(new_playlist_dir.song_list_mode, FileListMode::Directory);
        assert_eq!(
            new_playlist_sub.song_list_mode,
            FileListMode::Subdirectories
        );
    }

    #[test]
    fn test_fontdir() {
        let mut playlist_non = Playlist::default();
        let mut playlist_dir = Playlist::default();
        playlist_non.font_dir = None;
        playlist_dir.font_dir = Some("Fakepath".into());
        let new_playlist_non = run_serialize(playlist_non);
        let new_playlist_dir = run_serialize(playlist_dir);
        assert_eq!(new_playlist_non.font_dir, None);
        let dir_path = new_playlist_dir.font_dir.unwrap();
        assert_eq!(dir_path.to_str().unwrap(), "Fakepath");
    }

    #[test]
    fn test_songdir() {
        let mut playlist_non = Playlist::default();
        let mut playlist_dir = Playlist::default();
        playlist_non.midi_dir = None;
        playlist_dir.midi_dir = Some("Fakepath".into());
        let new_playlist_non = run_serialize(playlist_non);
        let new_playlist_dir = run_serialize(playlist_dir);
        assert_eq!(new_playlist_non.midi_dir, None);
        let dir_path = new_playlist_dir.midi_dir.unwrap();
        assert_eq!(dir_path.to_str().unwrap(), "Fakepath");
    }

    #[test]
    fn test_save_portable_unchecks_flag() {
        fs::create_dir_all("temp").unwrap();
        let mut playlist = Playlist::default();
        playlist.set_portable_path(Some(PathBuf::from("temp/testfile.midpl")));
        assert!(playlist.has_unsaved_changes());
        playlist.save_portable().unwrap();
        assert!(!playlist.has_unsaved_changes());
    }
}
