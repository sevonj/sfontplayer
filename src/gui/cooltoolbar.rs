use eframe::egui;
use rfd::FileDialog;

use crate::SfontPlayer;

pub(crate) fn toolbar(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    ui.menu_button("File", |ui| {
        ui.label("Workspace");
        if ui.button("Add soundfonts").clicked() {
            if let Some(paths) = FileDialog::new()
                .add_filter("Soundfonts", &["sf2"])
                .pick_files()
            {
                for path in paths {
                    app.add_sf(path);
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
            app.clear_sfs();
            ui.close_menu();
        }
        if ui.button("Clear songs").clicked() {
            app.clear_midis();
            ui.close_menu();
        }
    });
}
