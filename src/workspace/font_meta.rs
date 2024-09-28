use std::{
    fs::{self, File},
    path::PathBuf,
};

use rustysynth::SoundFont;

/// Reference to a font file with metadata
#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
#[serde(default)]
pub(crate) struct FontMeta {
    filepath: PathBuf,
    filesize: u64,
    error: bool,
}
impl FontMeta {
    pub fn new(filepath: PathBuf) -> Self {
        let mut this = Self {
            filepath,
            filesize: 0,
            error: false,
        };
        this.refresh();
        this
    }
    pub fn refresh(&mut self) {
        if let Ok(file_meta) = fs::metadata(&self.filepath) {
            self.filesize = file_meta.len();
        }
        if let Ok(mut file) = File::open(&self.filepath) {
            self.error = SoundFont::new(&mut file).is_err();
        }
    }
    pub fn get_path(&self) -> PathBuf {
        self.filepath.clone()
    }
    pub fn get_name(&self) -> String {
        self.filepath
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }
    pub fn get_size(&self) -> u64 {
        self.filesize
    }
    pub fn is_error(&self) -> bool {
        self.error
    }
}
