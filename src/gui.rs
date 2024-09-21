use eframe::egui;
use rfd::FileDialog;

use crate::SfontPlayer;
use egui_extras::{Column, TableBuilder};

const TBL_ROW_H: f32 = 16.;

pub(crate) fn draw_gui(ctx: &egui::Context, app: &mut SfontPlayer) {
    ctx.input(|i| {
        for file in i.raw.dropped_files.clone() {
            println!("{:?}", file)
        }
    });

    egui::TopBottomPanel::top("sf_table")
        .resizable(true)
        .show(ctx, |ui| {
            soundfont_table(ui, app);
            if ui.button("⊞ Add soundfonts").clicked() {
                if let Some(mut paths) = FileDialog::new()
                    .add_filter("Soundfonts", &["sf2"])
                    .pick_files()
                {
                    app.soundfonts.append(&mut paths);
                }
            }
        });
    egui::TopBottomPanel::bottom("playback_panel").show(ctx, |ui| {});
    egui::CentralPanel::default().show(ctx, |ui| {
        song_table(ui, app);
        if ui.button("⊞ Add songs").clicked() {
            if let Some(mut paths) = FileDialog::new()
                .add_filter("Midi files", &["mid"])
                .pick_files()
            {
                app.midis.append(&mut paths);
            }
        }
    });
}

fn soundfont_table(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(32.))
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|_| {});
            header.col(|ui| {
                ui.heading("Soundfont");
            });
        })
        .body(|mut body| {
            for (i, sf) in app.soundfonts.clone().iter().enumerate() {
                body.row(TBL_ROW_H, |mut row| {
                    row.col(|ui| {
                        if ui.button("❎").clicked() {
                            app.remove_sf(i)
                        }
                    });
                    row.col(|ui| {
                        let name = sf.file_name().unwrap().to_str().unwrap().to_owned();
                        let highlight = Some(i) == app.selected_sf;
                        if ui.add(egui::Button::new(name).frame(highlight)).clicked() {
                            app.selected_sf = Some(i);
                        }
                    });
                });
            }
        });
}

fn song_table(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(32.))
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|_| {});
            header.col(|ui| {
                ui.heading("Midis");
            });
        })
        .body(|mut body| {
            for (i, mid) in app.midis.clone().iter().enumerate() {
                body.row(TBL_ROW_H, |mut row| {
                    row.col(|ui| {
                        if ui.button("❎").clicked() {
                            app.remove_midi(i)
                        }
                    });
                    row.col(|ui| {
                        let name = mid.file_name().unwrap().to_str().unwrap().to_owned();
                        let highlight = Some(i) == app.selected_midi;
                        if ui.add(egui::Button::new(name).frame(highlight)).clicked() {
                            app.selected_midi = Some(i);
                            app.play();
                        }
                    });
                });
            }
        });
}
