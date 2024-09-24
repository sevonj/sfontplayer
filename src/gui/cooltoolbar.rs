use eframe::egui::{TextEdit, Ui};
use rfd::FileDialog;

use super::about::about_window;
use crate::SfontPlayer;

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
                    app.add_font(path);
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
                    app.add_midi(path);
                }
                ui.close_menu();
            }
        }
        if ui.button("Clear soundfonts").clicked() {
            app.clear_fonts();
            ui.close_menu();
        }
        if ui.button("Clear songs").clicked() {
            app.clear_midis();
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
    });
}

fn help_menu(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.menu_button("Help", |ui| {
        if ui.button("About").clicked() {
            app.show_about_window = true;
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
