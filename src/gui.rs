pub mod conversions;
mod cooltoolbar;
mod workspace_select;

use std::time::Duration;

use crate::SfontPlayer;
use conversions::format_duration;
use cooltoolbar::toolbar;
use eframe::egui::{
    Button, CentralPanel, Context, ScrollArea, SelectableLabel, Slider, TopBottomPanel, Ui,
};
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;
use workspace_select::{workspace_options, workspace_tabs};

const TBL_ROW_H: f32 = 16.;

pub(crate) fn draw_gui(ctx: &Context, app: &mut SfontPlayer) {
    ctx.input(|i| {
        for file in i.raw.dropped_files.clone() {
            println!("{:?}", file)
        }
    });
    TopBottomPanel::top("top_bar")
        .resizable(false)
        .show(ctx, |ui| {
            toolbar(ui, app);
            workspace_tabs(ui, app);
        });

    TopBottomPanel::top("workspace_toolbar")
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                workspace_options(ui, app);
                soundfont_controls(ui, app);
            });
        });

    if app.show_soundfonts {
        TopBottomPanel::top("sf_toolbar")
            .show_separator_line(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(Button::new("➕").frame(false))
                        .on_hover_text("Add")
                        .clicked()
                    {
                        if let Some(paths) = FileDialog::new()
                            .add_filter("Soundfonts", &["sf2"])
                            .pick_files()
                        {
                            for path in paths {
                                app.add_sf(path);
                            }
                        }
                    }
                    ui.heading("Soundfonts");
                });
            });
        TopBottomPanel::top("sf_table")
            .resizable(true)
            .show(ctx, |ui| {
                soundfont_table(ui, app);
            });
    }

    TopBottomPanel::bottom("playback_panel").show(ctx, |ui| {
        playback_panel(ui, app);
    });

    TopBottomPanel::top("song_toolbar")
        .show_separator_line(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .add(Button::new("➕").frame(false))
                    .on_hover_text("Add")
                    .clicked()
                {
                    if let Some(paths) = FileDialog::new()
                        .add_filter("Midi files", &["mid"])
                        .pick_files()
                    {
                        for path in paths {
                            app.add_midi(path);
                        }
                    }
                }
                ui.heading("Midi files");
            });
        });
    CentralPanel::default().show(ctx, |ui| {
        song_table(ui, app);
    });
}

fn soundfont_controls(ui: &mut Ui, app: &mut SfontPlayer) {
    if ui
        .selectable_label(app.show_soundfonts, "Soundfonts")
        .clicked()
    {
        app.show_soundfonts = !app.show_soundfonts
    }
}

fn soundfont_table(ui: &mut Ui, app: &mut SfontPlayer) {
    ScrollArea::vertical().show(ui, |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .vscroll(false)
            .column(Column::exact(16.))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|_| {});
                header.col(|ui| {
                    ui.label("Name");
                });
            })
            .body(|mut body| {
                for (i, sf) in app.get_soundfonts().iter().enumerate() {
                    body.row(TBL_ROW_H, |mut row| {
                        row.col(|ui| {
                            if ui
                                .add(Button::new("❎").frame(false))
                                .on_hover_text("Remove")
                                .clicked()
                            {
                                app.remove_sf(i)
                            }
                        });
                        row.col(|ui| {
                            let name = sf.file_name().unwrap().to_str().unwrap().to_owned();
                            let highlight = Some(i) == app.get_sf_idx();
                            if ui.add(Button::new(name).frame(highlight)).clicked() {
                                app.set_sf_idx(i);
                            }
                        });
                    });
                }
            });
    });
}

fn song_table(ui: &mut Ui, app: &mut SfontPlayer) {
    ScrollArea::vertical().show(ui, |ui| {
        TableBuilder::new(ui)
            .striped(true)
            .vscroll(false)
            .column(Column::exact(16.))
            .column(Column::auto().resizable(true))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|_| {});
                header.col(|ui| {
                    ui.label("Name");
                });
                header.col(|ui| {
                    ui.label("Time");
                });
            })
            .body(|mut body| {
                for i in 0..app.get_midis().len() {
                    let is_selected = Some(i) == app.get_midi_idx();
                    let filename = app.get_midis()[i]
                        .get_path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_owned();
                    let time = app.get_midis()[i].get_duration().unwrap_or(Duration::ZERO);

                    body.row(TBL_ROW_H, |mut row| {
                        // Remove button
                        row.col(|ui| {
                            if ui
                                .add(Button::new("❎").frame(false))
                                .on_hover_text("Remove")
                                .clicked()
                            {
                                app.remove_midi(i)
                            }
                        });
                        // Filename
                        row.col(|ui| {
                            if ui.add(Button::new(filename).frame(is_selected)).clicked() {
                                app.set_midi_idx(i);
                                app.start();
                            }
                        });
                        // Duration
                        row.col(|ui| {
                            ui.label(format_duration(time));
                        });
                    });
                }
            });
    });
}

fn playback_panel(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.horizontal(|ui| {
        ui.label("🎵");

        // Shuffle button
        if ui.add(SelectableLabel::new(app.shuffle, "🔀")).clicked() {
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
        if ui.add_enabled(prev_enabled, Button::new("⏪")).clicked() {
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
        if ui.add_enabled(next_enabled, Button::new("⏩")).clicked() {
            app.set_queue_idx(Some(app.get_queue_idx().unwrap() + 1));
            app.load_song();
        }
        // Stop button
        if ui.add_enabled(app.is_playing(), Button::new("⏹")).clicked() {
            app.stop()
        }
        // Slider
        let len = app.get_midi_length();
        let pos = app.get_midi_position();
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = 300.0;
            ui.add(
                Slider::new(&mut pos.as_secs_f64(), 0.0..=len.as_secs_f64())
                    .show_value(false)
                    .trailing_fill(true),
            );
        });

        ui.label(format!("{}/{}", format_duration(pos), format_duration(len)));
    });
}
