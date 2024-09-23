use std::{default, path::PathBuf, vec};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct Workspace {
    pub name: String,
    pub soundfonts: Vec<PathBuf>,
    pub midis: Vec<PathBuf>,
    pub selected_sf: Option<usize>,
    pub selected_midi: Option<usize>,
    pub queue: Vec<usize>,
    #[serde(skip)]
    pub queue_idx: Option<usize>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            name: "Workspace".to_owned(),
            soundfonts: vec![],
            midis: vec![],
            selected_sf: None,
            selected_midi: None,
            queue: vec![],
            queue_idx: None,
        }
    }
}
