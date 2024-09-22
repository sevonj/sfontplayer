use std::fmt::format;

use eframe::egui::{self, Color32};
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
        });

    egui::TopBottomPanel::bottom("playback_panel").show(ctx, |ui| {
        playback_panel(ui, app);
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        song_table(ui, app);
    });
}

fn soundfont_table(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .vscroll(false)
            .column(Column::exact(16.))
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
                            if ui.add(egui::Button::new("❎").frame(false)).clicked() {
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
        ui.horizontal(|ui| {
            if ui.button("⊞ Add soundfonts").clicked() {
                if let Some(mut paths) = FileDialog::new()
                    .add_filter("Soundfonts", &["sf2"])
                    .pick_files()
                {
                    app.soundfonts.append(&mut paths);
                }
            }
            if ui.button("Clear").clicked() {
                app.soundfonts.clear();
                app.selected_sf = None;
            }
        });
    });
}

fn song_table(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .vscroll(false)
            .column(Column::exact(16.))
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
                            if ui.add(egui::Button::new("❎").frame(false)).clicked() {
                                app.remove_midi(i)
                            }
                        });
                        row.col(|ui| {
                            let name = mid.file_name().unwrap().to_str().unwrap().to_owned();
                            let highlight = Some(i) == app.selected_midi;
                            if ui.add(egui::Button::new(name).frame(highlight)).clicked() {
                                app.selected_midi = Some(i);
                                app.start();
                            }
                        });
                    });
                }
            });
        ui.horizontal(|ui| {
            if ui.button("⊞ Add songs").clicked() {
                if let Some(mut paths) = FileDialog::new()
                    .add_filter("Midi files", &["mid"])
                    .pick_files()
                {
                    app.midis.append(&mut paths);
                }
            }
            if ui.button("Clear").clicked() {
                app.midis.clear();
                app.selected_midi = None;
            }
        });
    });
}

fn playback_panel(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    ui.horizontal(|ui| {
        ui.label("Add widgets");

        // Shuffle button
        if ui
            .add(egui::SelectableLabel::new(app.shuffle, "🔀"))
            .clicked()
        {
            app.shuffle = !app.shuffle;
            app.rebuild_queue();
        }

        let prev_enabled = if let Some(idx) = app.queue_idx {
            idx > 0
        } else {
            false
        };
        let next_enabled = if let Some(idx) = app.queue_idx {
            idx < app.queue.len() - 1
        } else {
            false
        };
        // Prev button
        if ui
            .add_enabled(prev_enabled, egui::Button::new("⏪"))
            .clicked()
        {
            app.queue_idx = Some(app.queue_idx.unwrap() - 1);
            app.load_song();
        }
        // PlayPause button
        if app.is_playing() {
            if ui.button("⏸").clicked() {
                app.pause();
            }
        } else {
            if ui.button("▶").clicked() {
                if app.is_empty() {
                    app.start();
                } else {
                    app.play();
                }
            }
        }
        // Next button
        if ui
            .add_enabled(next_enabled, egui::Button::new("⏩"))
            .clicked()
        {
            app.queue_idx = Some(app.queue_idx.unwrap() + 1);
            app.load_song();
        }
        // Stop button
        if ui
            .add_enabled(app.is_playing(), egui::Button::new("⏹"))
            .clicked()
        {
            app.stop()
        }
        // Slider
        let len = app.get_midi_length();
        let mut pos = app.get_midi_position();
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 300.0;
            ui.add(
                egui::Slider::new(&mut pos, 0.0..=len)
                    .show_value(false)
                    .trailing_fill(true),
            );
        });

        ui.label(format!("{:.0}/{:.0}", pos, len));
    });
}
