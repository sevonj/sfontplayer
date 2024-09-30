use eframe::egui::{Button, TextEdit, Theme, ThemePreference, Ui, ViewportCommand};
use rfd::FileDialog;

use crate::{workspace::FileListMode, SfontPlayer};

use super::keyboard_shortcuts::{
    GUI_SHOWFONTS, WORKSPACE_CREATE, WORKSPACE_MOVELEFT, WORKSPACE_MOVERIGHT, WORKSPACE_REMOVE,
    WORKSPACE_SWITCHLEFT, WORKSPACE_SWITCHRIGHT,
};

/// The topmost toolbar with File Menu
pub(crate) fn toolbar(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.horizontal(|ui| {
        file_menu(ui);

        options_menu(ui, app);

        workspace_menu(ui, app);

        help_menu(ui, app);
    });
}

fn file_menu(ui: &mut Ui) {
    ui.menu_button("File", |ui| {
        if ui.button("Exit").clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    });
}

fn options_menu(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.menu_button("Options", |ui| {
        if ui.ctx().theme() == Theme::Light {
            if ui.button("🌙 Toggle theme").clicked() {
                ui.ctx().set_theme(ThemePreference::Dark)
            }
        } else if ui.button("☀ Toggle theme").clicked() {
            ui.ctx().set_theme(ThemePreference::Light)
        }
        if ui
            .add(
                Button::new("Toggle soundfonts")
                    .shortcut_text(ui.ctx().format_shortcut(&GUI_SHOWFONTS)),
            )
            .clicked()
        {
            app.show_soundfonts = !app.show_soundfonts
        }
    });
}

fn workspace_menu(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.menu_button("Workspace", |ui| {
        ui.menu_button("Rename Workspace", |ui| {
            if ui
                .add(TextEdit::singleline(&mut app.get_workspace_mut().name).desired_width(128.))
                .lost_focus()
            {
                ui.close_menu();
            }
            if ui.button("OK").clicked() {
                ui.close_menu();
            }
        });
        ui.menu_button("Soundfonts", |ui| {
            let mut list_mode = app.get_workspace().get_font_list_mode();
            ui.add_enabled_ui(list_mode == FileListMode::Manual, |ui| {
                if ui.button("Add soundfonts").clicked() {
                    if let Some(paths) = FileDialog::new()
                        .add_filter("Soundfonts", &["sf2"])
                        .pick_files()
                    {
                        for path in paths {
                            app.get_workspace_mut().add_font(path);
                        }
                        ui.close_menu();
                    }
                }
                if ui.button("Clear soundfonts").clicked() {
                    app.get_workspace_mut().clear_fonts();
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
                app.get_workspace_mut().set_font_list_type(list_mode);
            }
        });
        ui.menu_button("Songs", |ui| {
            let mut list_mode = app.get_workspace().get_song_list_mode();
            ui.add_enabled_ui(list_mode == FileListMode::Manual, |ui| {
                if ui.button("Add songs").clicked() {
                    if let Some(paths) = FileDialog::new()
                        .add_filter("Midi files", &["mid"])
                        .pick_files()
                    {
                        for path in paths {
                            app.get_workspace_mut().add_song(path);
                        }
                        ui.close_menu();
                    }
                }
                if ui.button("Clear songs").clicked() {
                    app.get_workspace_mut().clear_songs();
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
                app.get_workspace_mut().set_song_list_mode(list_mode);
            }
        });

        if ui
            .add_enabled(
                app.workspace_idx > 0,
                Button::new("Switch one left")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SWITCHLEFT)),
            )
            .clicked()
        {
            app.switch_workspace_left();
            ui.close_menu();
        }
        if ui
            .add_enabled(
                app.workspace_idx < app.workspaces.len() - 1,
                Button::new("Switch one right")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_SWITCHRIGHT)),
            )
            .clicked()
        {
            app.switch_workspace_right();
            ui.close_menu();
        }
        if ui
            .add_enabled(
                app.workspace_idx > 0,
                Button::new("Move left")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_MOVELEFT)),
            )
            .clicked()
        {
            app.move_workspace_left();
            ui.close_menu();
        }
        if ui
            .add_enabled(
                app.workspace_idx < app.workspaces.len() - 1,
                Button::new("Move right")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_MOVERIGHT)),
            )
            .clicked()
        {
            app.move_workspace_right();
            ui.close_menu();
        }
        if ui
            .add(
                Button::new("Create a new workspace")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_CREATE)),
            )
            .clicked()
        {
            app.remove_workspace(app.workspace_idx);
            ui.close_menu();
        }
        if ui
            .add(
                Button::new("Remove workspace")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_REMOVE)),
            )
            .clicked()
        {
            app.remove_workspace(app.workspace_idx);
            ui.close_menu();
        }
    });
}

fn help_menu(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.menu_button("Help", |ui| {
        if ui.button("About").clicked() {
            app.show_about_modal = true;
            ui.close_menu();
        }
        if ui.button("Keyboard shortcuts").clicked() {
            app.show_shortcut_modal = true;
            ui.close_menu();
        }
    });
}
