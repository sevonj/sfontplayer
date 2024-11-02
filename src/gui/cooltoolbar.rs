use eframe::egui::{Button, TextEdit, Theme, ThemePreference, Ui, ViewportCommand};
use rfd::FileDialog;

use crate::{
    player::{workspace::enums::FileListMode, Player},
    GuiState,
};

use super::{
    keyboard_shortcuts::{
        GUI_SHOWFONTS, WORKSPACE_CREATE, WORKSPACE_DUPLICATE, WORKSPACE_MOVELEFT,
        WORKSPACE_MOVERIGHT, WORKSPACE_REFRESH, WORKSPACE_REMOVE, WORKSPACE_SAVE, WORKSPACE_SAVEAS,
        WORKSPACE_SWITCHLEFT, WORKSPACE_SWITCHRIGHT,
    },
    modals::file_dialogs,
};

/// The topmost toolbar with File Menu
pub fn toolbar(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.horizontal(|ui| {
        file_menu(ui, player, gui);

        options_menu(ui, gui);

        workspace_menu(ui, player, gui);

        help_menu(ui, gui);
    });
}

fn file_menu(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.menu_button("File", |ui| {
        gui.disable_play_shortcut();

        if ui.button("Exit").clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
        if ui.button("Open Workspace file").clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter("Workspace file", &["sfontspace"])
                .pick_file()
            {
                if let Err(e) = player.open_portable_workspace(path) {
                    gui.toast_error(e.to_string());
                }
            }
            ui.close_menu();
        }
    });
}

fn options_menu(ui: &mut Ui, gui: &mut GuiState) {
    ui.menu_button("Options", |ui| {
        gui.disable_play_shortcut();

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
fn workspace_menu(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.menu_button("Workspace", |ui| {
        gui.disable_play_shortcut();

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
        let can_refresh = player.get_workspace().get_font_list_mode() != FileListMode::Manual
            || player.get_workspace().get_song_list_mode() != FileListMode::Manual;
        if ui
            .add_enabled(
                can_refresh,
                Button::new("Refresh directory content")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_REFRESH)),
            )
            .clicked()
        {
            player.get_workspace_mut().refresh_font_list();
            player.get_workspace_mut().refresh_song_list();
        }
        ui.add_enabled_ui(player.get_workspace().is_portable(), |ui| {
            let hover_text = if player.get_workspace().is_portable() {
                "Save unsaved changes."
            } else {
                "Current workspace is stored in app data. App data is saved automatically."
            };
            if ui
                .add(Button::new("Save").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SAVE)))
                .on_hover_text(hover_text)
                .on_disabled_hover_text(hover_text)
                .clicked()
            {
                let _ = player.get_workspace_mut().save_portable();
            }
        });
        if ui
            .add(Button::new("Save as").shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SAVEAS)))
            .on_hover_text("Save a copy to a new file")
            .clicked()
        {
            file_dialogs::save_workspace_as(player, player.get_workspace_idx(), gui);
        }
        if ui
            .add(
                Button::new("Duplicate")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_DUPLICATE)),
            )
            .on_hover_text("Create a copy of current workspace")
            .clicked()
        {
            let _ = player.duplicate_workspace(player.get_workspace_idx());
        }
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
        gui.disable_play_shortcut();

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
