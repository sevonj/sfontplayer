use eframe::egui::{Button, TextEdit, Theme, ThemePreference, Ui, ViewportCommand};
use rfd::FileDialog;

use crate::{
    player::{workspace::enums::FileListMode, Player},
    GuiState,
};

use super::keyboard_shortcuts::{
    GUI_SHOWFONTS, WORKSPACE_CREATE, WORKSPACE_MOVELEFT, WORKSPACE_MOVERIGHT, WORKSPACE_REMOVE,
    WORKSPACE_SWITCHLEFT, WORKSPACE_SWITCHRIGHT,
};

/// The topmost toolbar with File Menu
pub fn toolbar(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.horizontal(|ui| {
        file_menu(ui);

        options_menu(ui, gui);

        workspace_menu(ui, player);

        help_menu(ui, gui);
    });
}

fn file_menu(ui: &mut Ui) {
    ui.menu_button("File", |ui| {
        if ui.button("Exit").clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    });
}

fn options_menu(ui: &mut Ui, gui: &mut GuiState) {
    ui.menu_button("Options", |ui| {
        if ui.ctx().theme() == Theme::Light {
            if ui.button("🌙 Toggle theme").clicked() {
                ui.ctx().set_theme(ThemePreference::Dark);
            }
        } else if ui.button("☀ Toggle theme").clicked() {
            ui.ctx().set_theme(ThemePreference::Light);
        }
        if ui
            .add(
                Button::new("Toggle soundfonts")
                    .shortcut_text(ui.ctx().format_shortcut(&GUI_SHOWFONTS)),
            )
            .clicked()
        {
            gui.show_soundfonts = !gui.show_soundfonts;
        }
    });
}

#[allow(clippy::too_many_lines)]
fn workspace_menu(ui: &mut Ui, player: &mut Player) {
    ui.menu_button("Workspace", |ui| {
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
        ui.menu_button("Soundfonts", |ui| {
            let mut list_mode = player.get_workspace().get_font_list_mode();
            ui.add_enabled_ui(list_mode == FileListMode::Manual, |ui| {
                if ui.button("Add soundfonts").clicked() {
                    if let Some(paths) = FileDialog::new()
                        .add_filter("Soundfonts", &["sf2"])
                        .pick_files()
                    {
                        for path in paths {
                            player.get_workspace_mut().add_font(path);
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
                player.get_workspace_mut().set_font_list_type(list_mode);
            }
        });
        ui.menu_button("Songs", |ui| {
            let mut list_mode = player.get_workspace().get_song_list_mode();
            ui.add_enabled_ui(list_mode == FileListMode::Manual, |ui| {
                if ui.button("Add songs").clicked() {
                    if let Some(paths) = FileDialog::new()
                        .add_filter("Midi files", &["mid"])
                        .pick_files()
                    {
                        for path in paths {
                            player.get_workspace_mut().add_song(path);
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

        if ui
            .add_enabled(
                player.get_workspace_idx() > 0,
                Button::new("Switch one left")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SWITCHLEFT)),
            )
            .clicked()
        {
            let _ = player.switch_workspace_left();
            ui.close_menu();
        }
        if ui
            .add_enabled(
                player.get_workspace_idx() < player.get_workspaces().len() - 1,
                Button::new("Switch one right")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SWITCHRIGHT)),
            )
            .clicked()
        {
            let _ = player.switch_workspace_right();
            ui.close_menu();
        }
        if ui
            .add_enabled(
                player.get_workspace_idx() > 0,
                Button::new("Move left")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_MOVELEFT)),
            )
            .clicked()
        {
            let _ = player.move_workspace_left();
            ui.close_menu();
        }
        if ui
            .add_enabled(
                player.get_workspace_idx() < player.get_workspaces().len() - 1,
                Button::new("Move right")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_MOVERIGHT)),
            )
            .clicked()
        {
            let _ = player.move_workspace_right();
            ui.close_menu();
        }
        if ui
            .add(
                Button::new("Create a new workspace")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_CREATE)),
            )
            .clicked()
        {
            let _ = player.remove_workspace(player.get_workspace_idx());
            ui.close_menu();
        }
        if ui
            .add(
                Button::new("Remove workspace")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_REMOVE)),
            )
            .clicked()
        {
            let _ = player.remove_workspace(player.get_workspace_idx());
            ui.close_menu();
        }
    });
}

fn help_menu(ui: &mut Ui, gui: &mut GuiState) {
    ui.menu_button("Help", |ui| {
        if ui.button("About").clicked() {
            gui.show_about_modal = true;
            ui.close_menu();
        }
        if ui.button("Keyboard shortcuts").clicked() {
            gui.show_shortcut_modal = true;
            ui.close_menu();
        }
    });
}
