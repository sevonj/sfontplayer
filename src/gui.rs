mod cooltoolbar;
mod workspace_select;

use crate::SfontPlayer;
use cooltoolbar::toolbar;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;
use workspace_select::{workspace_options, workspace_tabs};

const TBL_ROW_H: f32 = 16.;

pub(crate) fn draw_gui(ctx: &egui::Context, app: &mut SfontPlayer) {
    ctx.input(|i| {
        for file in i.raw.dropped_files.clone() {
            println!("{:?}", file)
        }
    });
    egui::TopBottomPanel::top("top_bar")
        .resizable(false)
        .show(ctx, |ui| {
            toolbar(ui, app);
            workspace_tabs(ui, app);
        });

    egui::TopBottomPanel::top("workspace_toolbar")
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                workspace_options(ui, app);
                soundfont_controls(ui, app);
            });
        });

    if app.show_soundfonts {
        egui::TopBottomPanel::top("sf_table")
            .resizable(true)
            .show(ctx, |ui| {
                soundfont_table(ui, app);
            });
    }

    egui::TopBottomPanel::bottom("playback_panel").show(ctx, |ui| {
        playback_panel(ui, app);
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        song_table(ui, app);
    });
}

fn soundfont_controls(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    if ui
        .selectable_label(app.show_soundfonts, "Soundfonts")
        .clicked()
    {
        app.show_soundfonts = !app.show_soundfonts
    }
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
                for (i, sf) in app.get_soundfonts().iter().enumerate() {
                    body.row(TBL_ROW_H, |mut row| {
                        row.col(|ui| {
                            if ui.add(egui::Button::new("❎").frame(false)).clicked() {
                                app.remove_sf(i)
                            }
                        });
                        row.col(|ui| {
                            let name = sf.file_name().unwrap().to_str().unwrap().to_owned();
                            let highlight = Some(i) == app.get_sf_idx();
                            if ui.add(egui::Button::new(name).frame(highlight)).clicked() {
                                app.set_sf_idx(i);
                            }
                        });
                    });
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
                for (i, mid) in app.get_midis().iter().enumerate() {
                    body.row(TBL_ROW_H, |mut row| {
                        row.col(|ui| {
                            if ui.add(egui::Button::new("❎").frame(false)).clicked() {
                                app.remove_midi(i)
                            }
                        });
                        row.col(|ui| {
                            let name = mid.file_name().unwrap().to_str().unwrap().to_owned();
                            let highlight = Some(i) == app.get_midi_idx();
                            if ui.add(egui::Button::new(name).frame(highlight)).clicked() {
                                app.set_midi_idx(i);
                                app.start();
                            }
                        });
                    });
                }
            });
    });
}

fn playback_panel(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    ui.horizontal(|ui| {
        ui.label("🎵");

        // Shuffle button
        if ui
            .add(egui::SelectableLabel::new(app.shuffle, "🔀"))
            .clicked()
        {
            app.shuffle = !app.shuffle;
            app.rebuild_queue();
        }

        let prev_enabled = if let Some(idx) = app.get_queue_idx() {
            idx > 0
        } else {
            false
        };
        let next_enabled = if let Some(idx) = app.get_queue_idx() {
            idx < app.get_queue().len() - 1
        } else {
            false
        };
        // Prev button
        if ui
            .add_enabled(prev_enabled, egui::Button::new("⏪"))
            .clicked()
        {
            app.set_queue_idx(Some(app.get_queue_idx().unwrap() - 1));
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
            app.set_queue_idx(Some(app.get_queue_idx().unwrap() + 1));
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
        let pos = app.get_midi_position();
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 300.0;
            ui.add(
                egui::Slider::new(&mut pos.as_secs_f64(), 0.0..=len.as_secs_f64())
                    .show_value(false)
                    .trailing_fill(true),
            );
        });

        ui.label(format!(
            "{:02}:{:02}/{:02}:{:02}",
            pos.as_secs() / 60,
            pos.as_secs() % 60,
            len.as_secs() / 60,
            len.as_secs() % 60,
        ));
    });
}
