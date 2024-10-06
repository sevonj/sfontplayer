//! Workspace (de)serialization Into / From JSON.
//!

use std::convert::Into;

use super::{
    enums::{FileListMode, FontSort, SongSort},
    font_meta::FontMeta,
    midi_meta::MidiMeta,
    Workspace,
};
use serde_json::{json, Value};

// Reference because we don't want to consume the workspace during autosave.
impl From<&Workspace> for Value {
    fn from(workspace: &Workspace) -> Self {
        json! ({
             "name": workspace.name,

             "fonts": workspace.fonts,
             "font_idx": workspace.font_idx,
             "font_list_mode": workspace.font_list_mode as u8,
             "font_dir": workspace.font_dir,
             "font_sort": workspace.font_sort as u8,

             "songs": workspace.midis,
             "song_idx": workspace.midi_idx,
             "song_list_mode": workspace.midi_list_mode as u8,
             "song_dir": workspace.midi_dir,
             "song_sort": workspace.song_sort as u8
            }
        )
    }
}

impl From<Value> for Workspace {
    /// Deserialize from json.
    fn from(value: Value) -> Self {
        let mut workspace = Self {
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
            midi_list_mode: value["song_list_mode"]
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
                        match workspace.font_idx {
                            Some(idx) if idx > i => workspace.font_idx = Some(idx - 1),
                            Some(idx) if idx == i => workspace.font_idx = None,
                            _ => (),
                        }
                        continue;
                    };
                    meta
                };
                workspace.fonts.push(fontmeta);
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
                        match workspace.font_idx {
                            Some(idx) if idx > i => workspace.midi_idx = Some(idx - 1),
                            Some(idx) if idx == i => workspace.midi_idx = None,
                            _ => (),
                        }
                        continue;
                    };
                    meta
                };
                workspace.midis.push(midimeta);
            }
        }

        let font_out_of_bounds = workspace
            .font_idx
            .is_some_and(|i| i >= workspace.fonts.len());
        let song_out_of_bounds = workspace
            .midi_idx
            .is_some_and(|i| i >= workspace.midis.len());

        if workspace.fonts.is_empty() || font_out_of_bounds {
            workspace.font_idx = None;
        }
        if workspace.midis.is_empty() || song_out_of_bounds {
            workspace.midi_idx = None;
        }

        workspace
    }
}

#[cfg(test)]
mod tests {
    //! These tests convert data into JSON and back, and then assert that it's unchanged.

    use super::*;

    fn run_serialize(workspace: Workspace) -> Workspace {
        Workspace::from(Value::from(&workspace))
    }

    #[test]
    fn test_fontlistmode() {
        let mut workspace_man = Workspace::default();
        let mut workspace_dir = Workspace::default();
        let mut workspace_sub = Workspace::default();
        workspace_man.font_list_mode = FileListMode::Manual;
        workspace_dir.font_list_mode = FileListMode::Directory;
        workspace_sub.font_list_mode = FileListMode::Subdirectories;
        let new_workspace_man = run_serialize(workspace_man);
        let new_workspace_dir = run_serialize(workspace_dir);
        let new_workspace_sub = run_serialize(workspace_sub);
        assert_eq!(new_workspace_man.font_list_mode, FileListMode::Manual);
        assert_eq!(new_workspace_dir.font_list_mode, FileListMode::Directory);
        assert_eq!(
            new_workspace_sub.font_list_mode,
            FileListMode::Subdirectories
        );
    }

    #[test]
    fn test_songlistmode() {
        let mut workspace_man = Workspace::default();
        let mut workspace_dir = Workspace::default();
        let mut workspace_sub = Workspace::default();
        workspace_man.midi_list_mode = FileListMode::Manual;
        workspace_dir.midi_list_mode = FileListMode::Directory;
        workspace_sub.midi_list_mode = FileListMode::Subdirectories;
        let new_workspace_man = run_serialize(workspace_man);
        let new_workspace_dir = run_serialize(workspace_dir);
        let new_workspace_sub = run_serialize(workspace_sub);
        assert_eq!(new_workspace_man.midi_list_mode, FileListMode::Manual);
        assert_eq!(new_workspace_dir.midi_list_mode, FileListMode::Directory);
        assert_eq!(
            new_workspace_sub.midi_list_mode,
            FileListMode::Subdirectories
        );
    }

    #[test]
    fn test_fontdir() {
        let mut workspace_non = Workspace::default();
        let mut workspace_dir = Workspace::default();
        workspace_non.font_dir = None;
        workspace_dir.font_dir = Some("Fakepath".into());
        let new_workspace_non = run_serialize(workspace_non);
        let new_workspace_dir = run_serialize(workspace_dir);
        assert_eq!(new_workspace_non.font_dir, None);
        let dir_path = new_workspace_dir.font_dir.unwrap();
        assert_eq!(dir_path.to_str().unwrap(), "Fakepath");
    }

    #[test]
    fn test_songdir() {
        let mut workspace_non = Workspace::default();
        let mut workspace_dir = Workspace::default();
        workspace_non.midi_dir = None;
        workspace_dir.midi_dir = Some("Fakepath".into());
        let new_workspace_non = run_serialize(workspace_non);
        let new_workspace_dir = run_serialize(workspace_dir);
        assert_eq!(new_workspace_non.midi_dir, None);
        let dir_path = new_workspace_dir.midi_dir.unwrap();
        assert_eq!(dir_path.to_str().unwrap(), "Fakepath");
    }

    #[test]
    fn test_fontsort() {
        let mut workspace_name_asc = Workspace::default();
        let mut workspace_name_desc = Workspace::default();
        let mut workspace_size_asc = Workspace::default();
        let mut workspace_size_desc = Workspace::default();
        workspace_name_asc.font_sort = FontSort::NameAsc;
        workspace_name_desc.font_sort = FontSort::NameDesc;
        workspace_size_asc.font_sort = FontSort::SizeAsc;
        workspace_size_desc.font_sort = FontSort::SizeDesc;
        let new_workspace_name_asc = run_serialize(workspace_name_asc);
        let new_workspace_name_desc = run_serialize(workspace_name_desc);
        let new_workspace_size_asc = run_serialize(workspace_size_asc);
        let new_workspace_size_desc = run_serialize(workspace_size_desc);
        assert_eq!(new_workspace_name_asc.font_sort, FontSort::NameAsc);
        assert_eq!(new_workspace_name_desc.font_sort, FontSort::NameDesc);
        assert_eq!(new_workspace_size_asc.font_sort, FontSort::SizeAsc);
        assert_eq!(new_workspace_size_desc.font_sort, FontSort::SizeDesc);
    }

    #[test]
    fn test_songsort() {
        let mut workspace_name_asc = Workspace::default();
        let mut workspace_name_desc = Workspace::default();
        let mut workspace_time_asc = Workspace::default();
        let mut workspace_time_desc = Workspace::default();
        let mut workspace_size_asc = Workspace::default();
        let mut workspace_size_desc = Workspace::default();
        workspace_name_asc.song_sort = SongSort::NameAsc;
        workspace_name_desc.song_sort = SongSort::NameDesc;
        workspace_time_asc.song_sort = SongSort::TimeAsc;
        workspace_time_desc.song_sort = SongSort::TimeDesc;
        workspace_size_asc.song_sort = SongSort::SizeAsc;
        workspace_size_desc.song_sort = SongSort::SizeDesc;
        let new_workspace_name_asc = run_serialize(workspace_name_asc);
        let new_workspace_name_desc = run_serialize(workspace_name_desc);
        let new_workspace_time_asc = run_serialize(workspace_time_asc);
        let new_workspace_time_desc = run_serialize(workspace_time_desc);
        let new_workspace_size_asc = run_serialize(workspace_size_asc);
        let new_workspace_size_desc = run_serialize(workspace_size_desc);
        assert_eq!(new_workspace_name_asc.song_sort, SongSort::NameAsc);
        assert_eq!(new_workspace_name_desc.song_sort, SongSort::NameDesc);
        assert_eq!(new_workspace_time_asc.song_sort, SongSort::TimeAsc);
        assert_eq!(new_workspace_time_desc.song_sort, SongSort::TimeDesc);
        assert_eq!(new_workspace_size_asc.song_sort, SongSort::SizeAsc);
        assert_eq!(new_workspace_size_desc.song_sort, SongSort::SizeDesc);
    }

    #[test]
    fn test_fontidx_valid_is_unchanged() {
        let mut workspace_69 = Workspace::default();
        let mut workspace_none = Workspace::default();
        for i in 0..=70 {
            workspace_69.add_font(format!("Fakepath{i}").into());
        }
        workspace_69.font_idx = Some(69);
        workspace_none.font_idx = None;
        let new_workspace_69 = run_serialize(workspace_69);
        let new_workspace_none = run_serialize(workspace_none);
        assert_eq!(new_workspace_69.font_idx, Some(69));
        assert_eq!(new_workspace_none.font_idx, None);
    }

    #[test]
    fn test_songidx_valid_is_unchanged() {
        let mut workspace_69 = Workspace::default();
        let mut workspace_none = Workspace::default();
        for i in 0..=70 {
            workspace_69.add_song(format!("Fakepath{i}").into());
        }
        workspace_69.midi_idx = Some(69);
        workspace_none.midi_idx = None;
        let new_workspace_69 = run_serialize(workspace_69);
        let new_workspace_none = run_serialize(workspace_none);
        assert_eq!(new_workspace_69.midi_idx, Some(69));
        assert_eq!(new_workspace_none.midi_idx, None);
    }

    #[test]
    fn test_fontidx_outofbounds_becomes_none() {
        let mut workspace_69 = Workspace::default();
        for i in 0..=7 {
            workspace_69.add_font(format!("Fakepath{i}").into());
        }
        workspace_69.font_idx = Some(69);
        let new_workspace_69 = run_serialize(workspace_69);
        assert_eq!(new_workspace_69.font_idx, None);
    }

    #[test]
    fn test_songidx_outofbounds_becomes_none() {
        let mut workspace_69 = Workspace::default();
        for i in 0..=7 {
            workspace_69.add_song(format!("Fakepath{i}").into());
        }
        workspace_69.midi_idx = Some(69);
        let new_workspace_69 = run_serialize(workspace_69);
        assert_eq!(new_workspace_69.midi_idx, None);
    }
}
