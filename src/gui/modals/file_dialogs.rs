use crate::{gui::GuiState, player::Player};
use rfd::FileDialog;

pub fn save_workspace_as(player: &mut Player, idx: usize, gui: &mut GuiState) {
    if let Some(filepath) = FileDialog::new()
        .add_filter("Workspace file", &["sfontspace"])
        .set_title("Save Workspace As")
        .set_file_name(format!("{}.sfontspace", &player.get_workspace().name))
        .save_file()
    {
        if let Err(e) = player.save_workspace_as(idx, filepath) {
            gui.toast_error(e.to_string());
        }
    }
}