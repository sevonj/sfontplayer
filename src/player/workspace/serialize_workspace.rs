//! Playlist (de)serialization Into / From JSON.
//!

use std::{convert::Into, fs::File, io::Write, path::PathBuf};

use super::{
    enums::{FileListMode, SongSort},
    font_meta::FontMeta,
    midi_meta::MidiMeta,
    Playlist,
};
use crate::player::soundfont_list::FontSort;
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

                     "fonts": playlist.fonts,
                     "font_idx": playlist.font_idx,
                     "font_list_mode": playlist.font_list_mode as u8,
                     "font_dir": playlist.font_dir,
                     "font_sort": playlist.font_sort as u8,

                     "songs": playlist.midis,
                     "song_idx": playlist.midi_idx,
                     "song_list_mode": playlist.song_list_mode as u8,
                     "song_dir": playlist.midi_dir,
                     "song_sort": playlist.song_sort as u8
                    }
                )
            },
            |root| {
                // Portable file: translate all paths into relative

                let mut fonts = playlist.fonts.clone();
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
                     "font_idx": playlist.font_idx,
                     "font_list_mode": playlist.font_list_mode as u8,
                     "font_dir": font_dir,
                     "font_sort": playlist.font_sort as u8,

                     "songs": songs,
                     "song_idx": playlist.midi_idx,
                     "song_list_mode": playlist.song_list_mode as u8,
                     "song_dir": song_dir,
                     "song_sort": playlist.song_sort as u8
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

            fonts: vec![],
            font_idx: value["font_idx"].as_u64().map(|idx| idx as usize),
            font_list_mode: value["font_list_mode"]
                .as_u64()
                .map_or_else(FileListMode::default, |int| {
                    FileListMode::try_from(int as u8).unwrap_or_default()
                }),
            font_dir: value["font_dir"].as_str().map(Into::into),
            font_sort: value["font_sort"]
                .as_u64()
                .map_or_else(FontSort::default, |int| {
                    FontSort::try_from(int as u8).unwrap_or_default()
                }),

            midis: vec![],
            midi_idx: value["song_idx"].as_u64().map(|idx| idx as usize),
            song_list_mode: value["song_list_mode"]
                .as_u64()
                .map_or_else(FileListMode::default, |int| {
                    FileListMode::try_from(int as u8).unwrap_or_default()
                }),
            midi_dir: value["song_dir"].as_str().map(Into::into),
            song_sort: value["song_sort"]
                .as_u64()
                .map_or_else(SongSort::default, |int| {
                    SongSort::try_from(int as u8).unwrap_or_default()
                }),

            ..Default::default()
        };

        if let Some(fonts_json) = value["fonts"].as_array() {
            for (i, data) in fonts_json.iter().enumerate() {
                let fontmeta = if let Ok(meta) = FontMeta::try_from(data) {
                    meta
                } else {
                    // Fallback
                    let Some(meta) = data["filepath"]
                        .as_str()
                        .map(|path| FontMeta::new(path.into()))
                    else {
                        // Give up
                        match playlist.font_idx {
                            Some(idx) if idx > i => playlist.font_idx = Some(idx - 1),
                            Some(idx) if idx == i => playlist.font_idx = None,
                            _ => (),
                        }
                        continue;
                    };
                    meta
                };
                playlist.fonts.push(fontmeta);
            }
        }

        if let Some(songs_json) = value["songs"].as_array() {
            for (i, data) in songs_json.iter().enumerate() {
                let midimeta = if let Ok(meta) = MidiMeta::try_from(data) {
                    meta
                } else {
                    // Fallback
                    let Some(meta) = data["filepath"]
                        .as_str()
                        .map(|path| MidiMeta::new(path.into()))
                    else {
                        // Give up
                        match playlist.font_idx {
                            Some(idx) if idx > i => playlist.midi_idx = Some(idx - 1),
                            Some(idx) if idx == i => playlist.midi_idx = None,
                            _ => (),
                        }
                        continue;
                    };
                    meta
                };
                playlist.midis.push(midimeta);
            }
        }

        let font_out_of_bounds = playlist.font_idx.is_some_and(|i| i >= playlist.fonts.len());
        let song_out_of_bounds = playlist.midi_idx.is_some_and(|i| i >= playlist.midis.len());

        if playlist.fonts.is_empty() || font_out_of_bounds {
            playlist.font_idx = None;
        }
        if playlist.midis.is_empty() || song_out_of_bounds {
            playlist.midi_idx = None;
        }

        playlist
    }
}

impl Playlist {
    pub fn open_portable(filepath: PathBuf) -> anyhow::Result<Self> {
        let json_str = std::fs::read_to_string(&filepath)?;
        let data: Value = serde_json::from_str(&json_str)?;
        let mut playlist = Self::from(data);

        // Make paths absolute
        let root: &PathBuf = &filepath;
        for font in &mut playlist.fonts {
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
    fn test_fontsort() {
        let mut playlist_name_asc = Playlist::default();
        let mut playlist_name_desc = Playlist::default();
        let mut playlist_size_asc = Playlist::default();
        let mut playlist_size_desc = Playlist::default();
        playlist_name_asc.font_sort = FontSort::NameAsc;
        playlist_name_desc.font_sort = FontSort::NameDesc;
        playlist_size_asc.font_sort = FontSort::SizeAsc;
        playlist_size_desc.font_sort = FontSort::SizeDesc;
        let new_playlist_name_asc = run_serialize(playlist_name_asc);
        let new_playlist_name_desc = run_serialize(playlist_name_desc);
        let new_playlist_size_asc = run_serialize(playlist_size_asc);
        let new_playlist_size_desc = run_serialize(playlist_size_desc);
        assert_eq!(new_playlist_name_asc.font_sort, FontSort::NameAsc);
        assert_eq!(new_playlist_name_desc.font_sort, FontSort::NameDesc);
        assert_eq!(new_playlist_size_asc.font_sort, FontSort::SizeAsc);
        assert_eq!(new_playlist_size_desc.font_sort, FontSort::SizeDesc);
    }

    #[test]
    fn test_songsort() {
        let mut playlist_name_asc = Playlist::default();
        let mut playlist_name_desc = Playlist::default();
        let mut playlist_time_asc = Playlist::default();
        let mut playlist_time_desc = Playlist::default();
        let mut playlist_size_asc = Playlist::default();
        let mut playlist_size_desc = Playlist::default();
        playlist_name_asc.song_sort = SongSort::NameAsc;
        playlist_name_desc.song_sort = SongSort::NameDesc;
        playlist_time_asc.song_sort = SongSort::TimeAsc;
        playlist_time_desc.song_sort = SongSort::TimeDesc;
        playlist_size_asc.song_sort = SongSort::SizeAsc;
        playlist_size_desc.song_sort = SongSort::SizeDesc;
        let new_playlist_name_asc = run_serialize(playlist_name_asc);
        let new_playlist_name_desc = run_serialize(playlist_name_desc);
        let new_playlist_time_asc = run_serialize(playlist_time_asc);
        let new_playlist_time_desc = run_serialize(playlist_time_desc);
        let new_playlist_size_asc = run_serialize(playlist_size_asc);
        let new_playlist_size_desc = run_serialize(playlist_size_desc);
        assert_eq!(new_playlist_name_asc.song_sort, SongSort::NameAsc);
        assert_eq!(new_playlist_name_desc.song_sort, SongSort::NameDesc);
        assert_eq!(new_playlist_time_asc.song_sort, SongSort::TimeAsc);
        assert_eq!(new_playlist_time_desc.song_sort, SongSort::TimeDesc);
        assert_eq!(new_playlist_size_asc.song_sort, SongSort::SizeAsc);
        assert_eq!(new_playlist_size_desc.song_sort, SongSort::SizeDesc);
    }

    #[test]
    fn test_fontidx_valid_is_unchanged() {
        let mut playlist_69 = Playlist::default();
        let mut playlist_none = Playlist::default();
        for i in 0..=70 {
            playlist_69.add_font(format!("Fakepath{i}").into()).unwrap();
        }
        playlist_69.font_idx = Some(69);
        playlist_none.font_idx = None;
        let new_playlist_69 = run_serialize(playlist_69);
        let new_playlist_none = run_serialize(playlist_none);
        assert_eq!(new_playlist_69.font_idx, Some(69));
        assert_eq!(new_playlist_none.font_idx, None);
    }

    #[test]
    fn test_songidx_valid_is_unchanged() {
        let mut playlist_69 = Playlist::default();
        let mut playlist_none = Playlist::default();
        for i in 0..=70 {
            playlist_69.add_song(format!("Fakepath{i}").into()).unwrap();
        }
        playlist_69.midi_idx = Some(69);
        playlist_none.midi_idx = None;
        let new_playlist_69 = run_serialize(playlist_69);
        let new_playlist_none = run_serialize(playlist_none);
        assert_eq!(new_playlist_69.midi_idx, Some(69));
        assert_eq!(new_playlist_none.midi_idx, None);
    }

    #[test]
    fn test_fontidx_outofbounds_becomes_none() {
        let mut playlist_69 = Playlist::default();
        for i in 0..=7 {
            playlist_69.add_font(format!("Fakepath{i}").into()).unwrap();
        }
        playlist_69.font_idx = Some(69);
        let new_playlist_69 = run_serialize(playlist_69);
        assert_eq!(new_playlist_69.font_idx, None);
    }

    #[test]
    fn test_songidx_outofbounds_becomes_none() {
        let mut playlist_69 = Playlist::default();
        for i in 0..=7 {
            playlist_69.add_song(format!("Fakepath{i}").into()).unwrap();
        }
        playlist_69.midi_idx = Some(69);
        let new_playlist_69 = run_serialize(playlist_69);
        assert_eq!(new_playlist_69.midi_idx, None);
    }

    #[test]
    fn test_save_portable_unchecks_flag() {
        fs::create_dir_all("temp").unwrap();
        let mut playlist = Playlist::default();
        playlist.set_portable_path(Some(PathBuf::from("temp/testfile.sfontspace")));
        assert!(playlist.has_unsaved_changes());
        playlist.save_portable().unwrap();
        assert!(!playlist.has_unsaved_changes());
    }
}
