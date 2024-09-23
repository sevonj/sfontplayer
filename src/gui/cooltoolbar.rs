use eframe::egui;
use rfd::FileDialog;

use crate::SfontPlayer;

pub(crate) fn toolbar(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    ui.horizontal(|ui| {
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

        ui.menu_button("Workspace", |ui| {
            ui.label("Rename Workspace");
            let rename_response = ui.add(
                egui::TextEdit::singleline(&mut app.get_workspace_mut().name).desired_width(128.),
            );
            if rename_response.lost_focus() {
                ui.close_menu();
            }
        });
    });
}
