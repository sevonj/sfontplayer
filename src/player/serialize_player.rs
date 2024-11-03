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
    workspace::{font_meta::FontMeta, Workspace},
    Player, RepeatMode,
};
use crate::player::PlayerError;

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceListEntry {
    pub filepath: String,
    pub portable: bool,
}

impl Player {
    pub fn save_state(&mut self) -> anyhow::Result<()> {
        if self.debug_block_saving {
            bail!(PlayerError::DebugBlockSaving)
        }
        if let Err(e) = self.save_workspaces() {
            bail!(format!("save_workspaces(): {e}"))
        }
        if let Err(e) = self.save_config() {
            bail!(format!("save_config(): {e}"))
        }

        Ok(())
    }

    pub fn load_state(&mut self) -> anyhow::Result<()> {
        if let Err(e) = self.load_workspaces() {
            bail!(format!("load_workspaces(): {e}"))
        }
        if let Err(e) = self.load_config() {
            bail!(format!("load_config(): {e}"))
        }

        Ok(())
    }

    fn save_config(&self) -> Result<(), anyhow::Error> {
        let state_dir = state_dir();
        fs::create_dir_all(&state_dir)?;

        let mut data = json! ({
            "shuffle": self.shuffle,
            "repeat": self.repeat,
            "workspace_idx": self.workspace_idx,
            "autosave": self.autosave,
        });
        if let Some(default) = &self.default_soundfont {
            data["default_soundfont_path"] = Value::from(default.get_path().to_str());
        }
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
        self.workspace_idx = match data["workspace_idx"].as_u64() {
            Some(x) if (x as usize) < self.workspaces.len() => x as usize,
            _ => 0,
        };
        self.autosave = data["autosave"].as_bool().is_some_and(|value| value);

        self.default_soundfont = data["default_soundfont_path"]
            .as_str()
            .map(|filepath| FontMeta::new(filepath.into()));

        Ok(())
    }

    fn save_workspaces(&mut self) -> anyhow::Result<()> {
        let project_dirs = project_dirs();
        let data_dir = project_dirs.data_dir();
        let workspace_dir = data_dir.join("workspaces");
        let workspace_dir_rel = PathBuf::from(".").join("workspaces");
        fs::create_dir_all(&workspace_dir)?;

        for file in fs::read_dir(&workspace_dir)? {
            let filepath = file?.path();
            remove_file(filepath)?;
        }

        let mut workspace_list = vec![];
        for i in 0..self.workspaces.len() {
            let workspace = &mut self.workspaces[i];
            let filename = generate_workspace_filename(workspace, i);

            // Relative if builtin storage ("./workspaces/filename.json"), absolute if portable
            let filepath = workspace
                .get_portable_path()
                .unwrap_or_else(|| workspace_dir_rel.join(&filename))
                .to_str()
                .expect("Workspace filepath string conversion failed.")
                .to_owned();
            workspace_list.push(WorkspaceListEntry {
                filepath,
                portable: workspace.is_portable(),
            });

            if !workspace.is_portable() {
                let abs_path = workspace
                    .get_portable_path()
                    .unwrap_or_else(|| workspace_dir.join(filename));
                let mut workspace_file = File::create(&abs_path)?;
                workspace_file.write_all(Value::from(&*workspace).to_string().as_bytes())?;
            } else if self.autosave {
                let _ = workspace.save_portable();
            }
        }

        let filepath = data_dir.join("workspaces.json");
        let data = json!(workspace_list);
        let mut file = File::create(filepath)?;
        file.write_all(data.to_string().as_bytes())?;

        Ok(())
    }

    fn load_workspaces(&mut self) -> anyhow::Result<()> {
        let project_dirs = project_dirs();
        let data_dir = project_dirs.data_dir();

        let filepath = data_dir.join("workspaces.json");
        let data_string = std::fs::read_to_string(filepath)?;
        let data: Vec<WorkspaceListEntry> = serde_json::from_str(&data_string)?;

        for entry in data {
            let workspace = if entry.portable {
                Workspace::open_portable(entry.filepath.into())?
            } else {
                let wksp_path = data_dir.join(entry.filepath);
                let wksp_str = std::fs::read_to_string(wksp_path)?;
                let wksp_data: Value = serde_json::from_str(&wksp_str)?;
                Workspace::from(wksp_data)
            };
            self.workspaces.push(workspace);
        }
        Ok(())
    }
}

fn generate_workspace_filename(workspace: &Workspace, idx: usize) -> String {
    format!(
        "{idx:02}_{}.json",
        workspace
            .name
            .replace(|c: char| !c.is_ascii_alphanumeric(), "")
            .to_lowercase()
    )
}

fn state_dir() -> PathBuf {
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
