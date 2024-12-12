use crate::{
    gui::GuiState,
    player::{soundfont_library::FontLibrary, Player},
};
use rfd::FileDialog;

pub fn open_playlist(player: &mut Player, gui: &mut GuiState) {
    if let Some(path) = FileDialog::new()
        .add_filter("Midi playlist", &["midpl"])
        .pick_file()
    {
        if let Err(e) = player.open_portable_playlist(path) {
            gui.toast_error(e.to_string());
        }
    }
}

pub fn save_playlist_as(player: &mut Player, idx: usize, gui: &mut GuiState) {
    if let Some(filepath) = FileDialog::new()
        .add_filter("Midi playlist", &["midpl"])
        .set_title("Save Playlist As")
        .set_file_name(format!("{}.midpl", &player.get_playlist().name))
        .save_file()
    {
        if let Err(e) = player.save_playlist_as(idx, filepath) {
            gui.toast_error(e.to_string());
        }
    }
}

// Add files and add dirs are separate because file dialog doesn't support mixed picking.
pub fn add_font_lib_files(font_lib: &mut FontLibrary /* , gui: &mut GuiState */) {
    if let Some(paths) = FileDialog::new()
        .add_filter("Soundfonts", &["sf2"])
        .set_title("Add files")
        .pick_files()
    {
        for path in paths {
            if let Err(_e) = font_lib.add_path(path) {
                // gui.toast_error(e.to_string());
            }
        }
        font_lib.refresh();
    }
}
pub fn add_font_lib_dirs(font_lib: &mut FontLibrary /* , gui: &mut GuiState */) {
    if let Some(paths) = FileDialog::new()
        .set_title("Add directories")
        .pick_folders()
    {
        for path in paths {
            if let Err(_e) = font_lib.add_path(path) {
                // gui.toast_error(e.to_string());
            }
        }
        font_lib.refresh();
    }
}
