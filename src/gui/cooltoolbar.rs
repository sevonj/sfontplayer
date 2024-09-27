use eframe::egui::{TextEdit, Ui};
use egui::Button;
use rfd::FileDialog;

use crate::SfontPlayer;

use super::hotkeys::{
    WORKSPACE_CREATE, WORKSPACE_MOVELEFT, WORKSPACE_MOVERIGHT, WORKSPACE_REMOVE,
    WORKSPACE_SWITCHLEFT, WORKSPACE_SWITCHRIGHT,
};

/// The topmost toolbar with File Menu
pub(crate) fn toolbar(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.horizontal(|ui| {
        file_menu(ui, app);

        workspace_menu(ui, app);

        help_menu(ui, app);

        soundfont_toggle(ui, app);
    });
}

fn file_menu(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.menu_button("File", |ui| {
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
        if ui.button("Add songs").clicked() {
            if let Some(paths) = FileDialog::new()
                .add_filter("Midi files", &["mid"])
                .pick_files()
            {
                for path in paths {
                    app.get_workspace_mut().add_midi(path);
                }
                ui.close_menu();
            }
        }
        if ui.button("Clear soundfonts").clicked() {
            app.get_workspace_mut().clear_fonts();
            ui.close_menu();
        }
        if ui.button("Clear songs").clicked() {
            app.get_workspace_mut().clear_midis();
            ui.close_menu();
        }
    });
}

fn workspace_menu(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.menu_button("Workspace", |ui| {
        ui.label("Rename Workspace");
        let rename_response =
            ui.add(TextEdit::singleline(&mut app.get_workspace_mut().name).desired_width(128.));
        if rename_response.lost_focus() {
            ui.close_menu();
        }
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
            .add_enabled(
                app.workspace_idx < app.workspaces.len() - 1,
                Button::new("Create a new workspace")
                    .shortcut_text(ui.ctx().format_shortcut(&WORKSPACE_CREATE)),
            )
            .clicked()
        {
            app.remove_workspace(app.workspace_idx);
            ui.close_menu();
        }
        if ui
            .add_enabled(
                app.workspace_idx < app.workspaces.len() - 1,
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
        if ui.button("Hotkeys").clicked() {
            app.show_shortcut_modal = true;
            ui.close_menu();
        }
    });
}

/// Special toggle for the soundfont list ui.
fn soundfont_toggle(ui: &mut Ui, app: &mut SfontPlayer) {
    if ui
        .selectable_label(app.show_soundfonts, "Soundfonts")
        .clicked()
    {
        app.show_soundfonts = !app.show_soundfonts
    }
}
