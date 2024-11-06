//! Common actions for context menus and such
//!

use std::path::Path;

use eframe::egui::{Button, Label, TextEdit, Ui};
use rfd::FileDialog;

use super::{
    keyboard_shortcuts::{
        WORKSPACE_CREATE, WORKSPACE_DUPLICATE, WORKSPACE_MOVELEFT, WORKSPACE_MOVERIGHT,
        WORKSPACE_OPEN, WORKSPACE_REFRESH, WORKSPACE_REMOVE, WORKSPACE_REOPEN, WORKSPACE_SAVE,
        WORKSPACE_SAVEAS, WORKSPACE_SWITCHLEFT, WORKSPACE_SWITCHRIGHT,
    },
    modals::file_dialogs,
    GuiState,
};
use crate::player::{workspace::enums::FileListMode, Player};

// --- Common File Actions --- //

pub fn open_dir(ui: &mut Ui, dir: &Path, gui: &mut GuiState) {
    if ui.button("Go to directory").clicked() {
        if let Err(e) = open::that(dir) {
            gui.toast_error(e.to_string());
        }
        ui.close_menu();
    }
}

pub fn open_file_dir(ui: &mut Ui, filepath: &Path, gui: &mut GuiState) {
    if ui.button("Go to directory").clicked() {
        let Some(dir) = filepath.parent() else {
            gui.toast_error("Failed to get file parent.");
            return;
        };
        if let Err(e) = open::that(dir) {
            gui.toast_error(e.to_string());
        }
        ui.close_menu();
    }
}

// --- Workspace File Actions --- //

pub fn new_workspace(ui: &mut Ui, player: &mut Player) {
    if ui
        .add(Button::new("New").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_CREATE)))
        .on_hover_text("Create a new workspace")
        .clicked()
    {
        player.new_workspace();
        ui.close_menu();
    }
}

pub fn open_workspace(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    if ui
        .add(Button::new("Open").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_OPEN)))
        .on_hover_text("Load a workspace file")
        .clicked()
    {
        file_dialogs::open_workspace(player, gui);
        ui.close_menu();
    }
}

pub fn save_workspace(ui: &mut Ui, player: &mut Player, index: usize, gui: &mut GuiState) {
    ui.add_enabled_ui(
        player.get_workspaces()[index].is_portable() && !player.autosave,
        |ui| {
            let hover_text = get_save_workspace_tooltip(player, index);
            if ui
                .add(Button::new("Save"))
                .on_hover_text(hover_text)
                .on_disabled_hover_text(hover_text)
                .clicked()
            {
                if let Err(e) = player.save_portable_workspace(index) {
                    gui.toast_error(e.to_string());
                }
                ui.close_menu();
            }
        },
    );
}

pub fn save_current_workspace(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.add_enabled_ui(
        player.get_workspace().is_portable() && !player.autosave,
        |ui| {
            let hover_text = get_save_workspace_tooltip(player, player.get_workspace_idx());
            if ui
                .add(Button::new("Save").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SAVE)))
                .on_hover_text(hover_text)
                .on_disabled_hover_text(hover_text)
                .clicked()
            {
                if let Err(e) = player.save_portable_workspace(player.get_workspace_idx()) {
                    gui.toast_error(e.to_string());
                }
                ui.close_menu();
            }
        },
    );
}

fn get_save_workspace_tooltip(player: &Player, index: usize) -> &str {
    if !player.get_workspaces()[index].is_portable() {
        "Workspaces in app memory are saved automatically."
    } else if player.autosave {
        "Autosave is enabled."
    } else {
        "Save unsaved changes."
    }
}

pub fn save_workspace_as(ui: &mut Ui, player: &mut Player, index: usize, gui: &mut GuiState) {
    if ui
        .add(Button::new("Save as"))
        .on_hover_text("Save a copy to a new file")
        .clicked()
    {
        file_dialogs::save_workspace_as(player, index, gui);
        ui.close_menu();
    }
}

pub fn save_current_workspace_as(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    if ui
        .add(Button::new("Save as").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SAVEAS)))
        .on_hover_text("Save a copy to a new file")
        .clicked()
    {
        file_dialogs::save_workspace_as(player, player.get_workspace_idx(), gui);
        ui.close_menu();
    }
}

pub fn duplicate_workspace(ui: &mut Ui, player: &mut Player, index: usize) {
    if ui
        .add(Button::new("Duplicate"))
        .on_hover_text("Create a copy of this workspace")
        .clicked()
    {
        let _ = player.duplicate_workspace(index);
        ui.close_menu();
    }
}

pub fn duplicate_current_workspace(ui: &mut Ui, player: &mut Player) {
    if ui
        .add(Button::new("Duplicate").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_DUPLICATE)))
        .on_hover_text("Create a copy of current workspace")
        .clicked()
    {
        let _ = player.duplicate_workspace(player.get_workspace_idx());
        ui.close_menu();
    }
}

pub fn close_workspace(ui: &mut Ui, player: &mut Player, index: usize) {
    if ui
        .add(Button::new("Close"))
        .on_hover_text("Close workspace")
        .clicked()
    {
        let _ = player.remove_workspace(index);
        ui.close_menu();
    }
}

pub fn close_current_workspace(ui: &mut Ui, player: &mut Player) {
    if ui
        .add(Button::new("Close").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_REMOVE)))
        .on_hover_text("Close workspace")
        .clicked()
    {
        let _ = player.remove_workspace(player.get_workspace_idx());
        ui.close_menu();
    }
}

pub fn reopen_workspace(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.has_removed_workspaces(),
            Button::new("Reopen closed").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_REOPEN)),
        )
        .on_hover_text("Reopen last closed workspace")
        .on_disabled_hover_text("Reopen last closed workspace")
        .clicked()
    {
        player.reopen_removed_workspace();
        ui.close_menu();
    }
}

// --- Workspace Content Actions --- //

pub fn rename_workspace(ui: &mut Ui, player: &mut Player, index: usize) {
    ui.add(Label::new("Name:").selectable(false));
    ui.add(TextEdit::singleline(&mut player.get_workspaces_mut()[index].name).desired_width(128.));
}

pub fn rename_current_workspace(ui: &mut Ui, player: &mut Player) {
    ui.menu_button("Rename Workspace", |ui| {
        if ui
            .add(TextEdit::singleline(&mut player.get_workspace_mut().name).desired_width(128.))
            .lost_focus()
        {
            ui.close_menu();
        }
        if ui.button("OK").clicked() {
            ui.close_menu();
        }
    });
}

pub fn refresh_workspace(player: &mut Player, index: usize, ui: &mut Ui) {
    let workspace = &mut player.get_workspaces_mut()[index];
    let can_refresh = workspace.get_font_list_mode() != FileListMode::Manual
        || workspace.get_song_list_mode() != FileListMode::Manual;
    ui.add_enabled_ui(can_refresh, |ui| {
        if ui
            .button("Refresh content")
            .on_hover_text("Refresh directory contents")
            .on_disabled_hover_text("This workspace uses manual listing.")
            .clicked()
        {
            workspace.refresh_font_list();
            workspace.refresh_song_list();
            ui.close_menu();
        }
    });
}

pub fn refresh_current_workspace(player: &mut Player, ui: &mut Ui) {
    let can_refresh = player.get_workspace().get_font_list_mode() != FileListMode::Manual
        || player.get_workspace().get_song_list_mode() != FileListMode::Manual;
    if ui
        .add_enabled(
            can_refresh,
            Button::new("Refresh content")
                .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_REFRESH)),
        )
        .on_hover_text("Refresh directory contents")
        .on_disabled_hover_text("This workspace uses manual listing.")
        .clicked()
    {
        player.get_workspace_mut().refresh_font_list();
        player.get_workspace_mut().refresh_song_list();
        ui.close_menu();
    }
}

pub fn current_workspace_fonts_action(ui: &mut Ui, player: &mut Player) {
    ui.menu_button("Soundfonts", |ui| {
        let mut list_mode = player.get_workspace().get_font_list_mode();
        ui.add_enabled_ui(list_mode == FileListMode::Manual, |ui| {
            if ui.button("Add soundfonts").clicked() {
                if let Some(paths) = FileDialog::new()
                    .add_filter("Soundfonts", &["sf2"])
                    .pick_files()
                {
                    for path in paths {
                        let _ = player.get_workspace_mut().add_font(path);
                    }
                    ui.close_menu();
                }
            }
            if ui.button("Clear soundfonts").clicked() {
                player.get_workspace_mut().clear_fonts();
                ui.close_menu();
            }
        });
        ui.label("Content mode");
        let response1 = ui.radio_value(&mut list_mode, FileListMode::Manual, "Manual");
        let response2 = ui.radio_value(&mut list_mode, FileListMode::Directory, "Directory");
        let response3 = ui.radio_value(
            &mut list_mode,
            FileListMode::Subdirectories,
            "Subdirectories",
        );
        if response1.clicked() || response2.clicked() || response3.clicked() {
            player.get_workspace_mut().set_font_list_mode(list_mode);
        }
    });
}
pub fn current_workspace_songs_action(ui: &mut Ui, player: &mut Player) {
    ui.menu_button("Songs", |ui| {
        let mut list_mode = player.get_workspace().get_song_list_mode();
        ui.add_enabled_ui(list_mode == FileListMode::Manual, |ui| {
            if ui.button("Add songs").clicked() {
                if let Some(paths) = FileDialog::new()
                    .add_filter("Midi files", &["mid"])
                    .pick_files()
                {
                    for path in paths {
                        let _ = player.get_workspace_mut().add_song(path);
                    }
                    ui.close_menu();
                }
            }
            if ui.button("Clear songs").clicked() {
                player.get_workspace_mut().clear_songs();
                ui.close_menu();
            }
        });
        ui.label("Content mode");
        let response1 = ui.radio_value(&mut list_mode, FileListMode::Manual, "Manual");
        let response2 = ui.radio_value(&mut list_mode, FileListMode::Directory, "Directory");
        let response3 = ui.radio_value(
            &mut list_mode,
            FileListMode::Subdirectories,
            "Subdirectories",
        );
        if response1.clicked() || response2.clicked() || response3.clicked() {
            player.get_workspace_mut().set_song_list_mode(list_mode);
        }
    });
}

// --- Workspace Navigation --- //

pub fn switch_workspace_left(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.get_workspace_idx() > 0,
            Button::new("Switch one left")
                .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SWITCHLEFT)),
        )
        .on_hover_text("Switch to previous workspace")
        .on_disabled_hover_text("Switch to previous workspace")
        .clicked()
    {
        let _ = player.switch_workspace_left();
        ui.close_menu();
    }
}

pub fn switch_workspace_right(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.get_workspace_idx() < player.get_workspaces().len() - 1,
            Button::new("Switch one right")
                .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SWITCHRIGHT)),
        )
        .on_hover_text("Switch to next workspace")
        .on_disabled_hover_text("Switch to next workspace")
        .clicked()
    {
        let _ = player.switch_workspace_right();
        ui.close_menu();
    }
}

pub fn move_workspace_left(ui: &mut Ui, player: &mut Player, index: usize) {
    if ui
        .add_enabled(index > 0, Button::new("Move left"))
        .on_hover_text("Move workspace left")
        .on_disabled_hover_text("Move workspace left")
        .clicked()
    {
        let _ = player.move_workspace(index, index - 1);
        ui.close_menu();
    }
}

pub fn move_current_workspace_left(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.get_workspace_idx() > 0,
            Button::new("Move left").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_MOVELEFT)),
        )
        .on_hover_text("Move workspace left")
        .on_disabled_hover_text("Move workspace left")
        .clicked()
    {
        let _ = player.move_workspace_left();
        ui.close_menu();
    }
}

pub fn move_workspace_right(ui: &mut Ui, player: &mut Player, index: usize) {
    if ui
        .add_enabled(
            index < player.get_workspaces().len() - 1,
            Button::new("Move right"),
        )
        .on_hover_text("Move workspace right")
        .on_disabled_hover_text("Move workspace right")
        .clicked()
    {
        let _ = player.move_workspace(index, index + 1);
        ui.close_menu();
    }
}

pub fn move_current_workspace_right(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.get_workspace_idx() < player.get_workspaces().len() - 1,
            Button::new("Move right").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_MOVERIGHT)),
        )
        .on_hover_text("Move workspace right")
        .on_disabled_hover_text("Move workspace right")
        .clicked()
    {
        let _ = player.move_workspace_right();
        ui.close_menu();
    }
}
