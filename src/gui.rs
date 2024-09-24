mod about;
pub(crate) mod conversions;
mod cooltoolbar;
mod workspace_select;

use std::time::Duration;

use crate::SfontPlayer;
use about::about_window;
use conversions::format_duration;
use cooltoolbar::toolbar;
use eframe::egui::{
    Button, CentralPanel, Context, ScrollArea, SelectableLabel, Slider, TopBottomPanel, Ui,
};
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;
use workspace_select::workspace_tabs;

const TBL_ROW_H: f32 = 16.;

pub(crate) fn draw_gui(ctx: &Context, app: &mut SfontPlayer) {
    about_window(ctx, app);

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

    if app.show_soundfonts {
        TopBottomPanel::top("font_titlebar")
            .show_separator_line(false)
            .resizable(false)
            .show(ctx, |ui| {
                check_disabled(ui, app);
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
                                app.add_font(path);
                            }
                        }
                    }
                    ui.heading("Soundfonts");
                });
            });
        TopBottomPanel::top("font_table")
            .resizable(true)
            .show(ctx, |ui| {
                check_disabled(ui, app);
                soundfont_table(ui, app);
            });
    }

    TopBottomPanel::bottom("playback_panel").show(ctx, |ui| {
        playback_panel(ui, app);
    });

    TopBottomPanel::top("midi_titlebar")
        .show_separator_line(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                check_disabled(ui, app);
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
        check_disabled(ui, app);
        song_table(ui, app);
    });
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
                for (i, sf) in app.get_fonts().clone().iter().enumerate() {
                    body.row(TBL_ROW_H, |mut row| {
                        row.col(|ui| {
                            if ui
                                .add(Button::new("❎").frame(false))
                                .on_hover_text("Remove")
                                .clicked()
                            {
                                app.remove_font(i)
                            }
                        });
                        row.col(|ui| {
                            let name = sf.file_name().unwrap().to_str().unwrap().to_owned();
                            let highlight = Some(i) == app.get_font_idx();
                            if ui.add(Button::new(name).frame(highlight)).clicked() {
                                app.set_font_idx(i);
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
            app.play_selected_song();
        }
        // PlayPause button
        if app.is_paused() {
            if ui.button("▶").clicked() {
                if app.is_empty() {
                    app.start();
                } else {
                    app.play();
                }
            }
        } else {
            if ui.button("⏸").clicked() {
                app.pause();
            }
        }
        // Next button
        if ui.add_enabled(next_enabled, Button::new("⏩")).clicked() {
            app.set_queue_idx(Some(app.get_queue_idx().unwrap() + 1));
            app.play_selected_song();
        }
        // Stop button
        if ui.add_enabled(!app.is_paused(), Button::new("⏹")).clicked() {
            app.stop()
        }
        // Slider
        let len = app.get_midi_length();
        // This stops the slider from showing halfway if len is zero.
        let slider_len = if len.is_zero() { 1. } else { len.as_secs_f64() };
        let pos = app.get_midi_position();
        ui.horizontal(|ui| {
            ui.spacing_mut().slider_width = f32::max(ui.available_width() - 128., 64.);
            ui.add_enabled(
                !len.is_zero(),
                Slider::new(&mut pos.as_secs_f64(), 0.0..=slider_len)
                    .show_value(false)
                    .trailing_fill(true),
            );
        });

        ui.label(format!("{}/{}", format_duration(pos), format_duration(len)));

        volume_control(ui, app);
    });
}

fn volume_control(ui: &mut Ui, app: &mut SfontPlayer) {
    let speaker_icon_str = match app.volume {
        x if x == 0.0 => "🔇",
        x if (0.0..33.0).contains(&x) => "🔈",
        x if (33.0..66.0).contains(&x) => "🔉",
        _ => "🔊",
    };

    ui.menu_button(speaker_icon_str, |ui| {
        if ui
            .add(
                Slider::new(&mut app.volume, 0.0..=100.)
                    .vertical()
                    .show_value(false)
                    .trailing_fill(true),
            )
            .changed()
        {
            app.update_volume();
        }
    });

    ui.label(format!("{:00}", app.volume));
}

/// This will disable the UI if modals are open
fn check_disabled(ui: &mut Ui, app: &SfontPlayer) {
    if app.show_about_window {
        ui.disable();
    }
}
