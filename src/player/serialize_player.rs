//! Player state save / load.
//!

use std::{
    fs::{self, remove_file, File},
    io::Write,
    path::PathBuf,
};

use anyhow::bail;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{
    playlist::{Playlist, SongSort},
    soundfont_list::FontSort,
    Player, RepeatMode,
};
use crate::player::{playlist::FontMeta, PlayerError};

#[derive(Debug, Serialize, Deserialize)]
struct PlaylistListEntry {
    pub filepath: String,
    pub portable: bool,
    /// Playlist state is packed into a JSON string to avoid invalidating the
    /// more important members if/when the scheme changes.
    pub state: String,
}

impl Player {
    pub fn save_state(&mut self) -> anyhow::Result<()> {
        if self.debug_block_saving {
            bail!(PlayerError::DebugBlockSaving)
        }
        if let Err(e) = self.save_playlists() {
            bail!(format!("save_playlists(): {e}"))
        }
        if let Err(e) = self.save_config() {
            bail!(format!("save_config(): {e}"))
        }
        if let Err(e) = self.save_fontlib() {
            bail!(format!("save_fontlib(): {e}"))
        }

        Ok(())
    }

    pub fn load_state(&mut self) -> anyhow::Result<()> {
        if let Err(e) = self.load_playlists() {
            bail!(format!("load_playlists(): {e}"))
        }
        if let Err(e) = self.load_config() {
            bail!(format!("load_config(): {e}"))
        }
        if let Err(e) = self.load_fontlib() {
            bail!(format!("load_fontlib(): {e}"))
        }

        Ok(())
    }

    fn save_config(&self) -> Result<(), anyhow::Error> {
        let state_dir = state_dir();
        fs::create_dir_all(&state_dir)?;

        let data = json! ({
            "shuffle": self.shuffle,
            "repeat": self.repeat,
            "playlist_idx": self.playlist_idx,
            "autosave": self.autosave,
        });
        let config_file = state_dir.join("state.json");
        let mut file = File::create(config_file)?;
        file.write_all(data.to_string().as_bytes())?;

        Ok(())
    }

    fn load_config(&mut self) -> anyhow::Result<()> {
        let state_filepath = state_dir().join("state.json");
        let data_string = std::fs::read_to_string(state_filepath)?;
        let data: Value = serde_json::from_str(&data_string)?;

        self.shuffle = data["shuffle"].as_bool().is_some_and(|value| value);
        if let Some(repeat) = data["repeat"].as_u64() {
            self.repeat = RepeatMode::try_from(repeat as u8).unwrap_or_default();
        }
        self.playlist_idx = match data["playlist_idx"].as_u64() {
            Some(x) if (x as usize) < self.playlists.len() => x as usize,
            _ => 0,
        };
        self.autosave = data["autosave"].as_bool().is_some_and(|value| value);

        Ok(())
    }

    fn save_fontlib(&self) -> anyhow::Result<()> {
        let state_dir = state_dir();
        fs::create_dir_all(&state_dir)?;

        let filepath = state_dir.join("fontlib.json");
        let mut file = File::create(filepath)?;

        let data = json!({
            "paths": self.font_lib.get_paths(),
            "selected": self.font_lib.get_selected().map(FontMeta::filepath)
        });

        file.write_all(data.to_string().as_bytes())?;

        Ok(())
    }

    fn load_fontlib(&mut self) -> anyhow::Result<()> {
        let filepath = state_dir().join("fontlib.json");
        let data_string = std::fs::read_to_string(filepath)?;
        let data: Value = serde_json::from_str(&data_string)?;
        let Some(paths) = data["paths"].as_array() else {
            bail!("Couldn't parse paths");
        };
        for value in paths {
            let Some(path_str) = value.as_str() else {
                bail!("Couldn't parse path: {value}")
            };
            let _ = self.font_lib.add_path(PathBuf::from(path_str));
        }
        let Some(selected) = data["selected"].as_str().map(std::convert::Into::into) else {
            bail!("Couldn't parse paths");
        };
        let _ = self.font_lib.select_by_filepath(&selected);

        Ok(())
    }

    fn save_playlists(&mut self) -> anyhow::Result<()> {
        let data_dir = data_dir();
        let playlist_dir = data_dir.join("playlists");
        let playlist_dir_rel = PathBuf::from(".").join("playlists");
        fs::create_dir_all(&playlist_dir)?;

        for file in fs::read_dir(&playlist_dir)? {
            let filepath = file?.path();
            remove_file(filepath)?;
        }

        let mut playlist_list = vec![];
        for i in 0..self.playlists.len() {
            let playlist = &mut self.playlists[i];
            let filename = generate_playlist_filename(playlist, i);

            // Relative if builtin storage ("./playlists/filename.json"), absolute if portable
            let filepath = playlist
                .get_portable_path()
                .unwrap_or_else(|| playlist_dir_rel.join(&filename))
                .to_str()
                .expect("Playlist filepath string conversion failed.")
                .to_owned();

            let state = json!({
                "font_idx": playlist.get_font_idx(),
                "font_sort": playlist.get_font_sort() as u8,
                "song_idx": playlist.get_song_idx(),
                "song_sort": playlist.get_song_sort() as u8,
            })
            .to_string();

            playlist_list.push(PlaylistListEntry {
                filepath,
                portable: playlist.is_portable(),
                state,
            });

            if !playlist.is_portable() {
                let abs_path = playlist
                    .get_portable_path()
                    .unwrap_or_else(|| playlist_dir.join(filename));
                let mut playlist_file = File::create(&abs_path)?;
                playlist_file.write_all(Value::from(&*playlist).to_string().as_bytes())?;
            } else if self.autosave {
                let _ = playlist.save_portable();
            }
        }

        let filepath = data_dir.join("playlists.json");
        let data = json!(playlist_list);
        let mut file = File::create(filepath)?;
        file.write_all(data.to_string().as_bytes())?;

        Ok(())
    }

    fn load_playlists(&mut self) -> anyhow::Result<()> {
        let data_dir = data_dir();

        let filepath = data_dir.join("playlists.json");
        let data_string = std::fs::read_to_string(filepath)?;
        let data: Vec<PlaylistListEntry> = serde_json::from_str(&data_string)?;

        for entry in data {
            let mut playlist = if entry.portable {
                Playlist::open_portable(entry.filepath.into())?
            } else {
                let wksp_path = data_dir.join(entry.filepath);
                let wksp_str = std::fs::read_to_string(wksp_path)?;
                let wksp_data: Value = serde_json::from_str(&wksp_str)?;
                Playlist::from(wksp_data)
            };

            let entry_state: Result<Value, serde_json::Error> = serde_json::from_str(&entry.state);
            if let Ok(state) = entry_state {
                if let Some(font_idx) = state["font_idx"].as_u64() {
                    let _ = playlist.select_font(font_idx as usize);
                }
                if let Some(font_sort) = state["font_sort"].as_u64() {
                    if let Ok(sort) = FontSort::try_from(font_sort as u8) {
                        playlist.set_font_sort(sort);
                    }
                }
                if let Some(song_idx) = state["song_idx"].as_u64() {
                    let _ = playlist.set_song_idx(Some(song_idx as usize));
                }
                if let Some(song_sort) = state["song_sort"].as_u64() {
                    if let Ok(sort) = SongSort::try_from(song_sort as u8) {
                        playlist.set_song_sort(sort);
                    }
                }
            }

            self.playlists.push(playlist);
        }

        Ok(())
    }
}

fn generate_playlist_filename(playlist: &Playlist, idx: usize) -> String {
    format!(
        "{idx:02}_{}.json",
        playlist
            .name
            .replace(|c: char| !c.is_ascii_alphanumeric(), "")
            .to_lowercase()
    )
}

pub fn data_dir() -> PathBuf {
    project_dirs().data_dir().into()
}

pub fn state_dir() -> PathBuf {
    project_dirs()
        .state_dir()
        .map(std::borrow::ToOwned::to_owned)
        .map_or_else(
            || project_dirs().data_dir().join("fallback_state_dir"),
            |path| path,
        )
}

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("fi", "sevonj", env!("CARGO_PKG_NAME"))
        .expect("Failed to create project dirs.")
}
