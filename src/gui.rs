mod about;
pub(crate) mod conversions;
mod cooltoolbar;
mod hotkeys;
mod playback_controls;
mod workspace_select;

use std::time::Duration;

use crate::SfontPlayer;
use about::about_modal;
use conversions::format_duration;
use cooltoolbar::toolbar;
use eframe::egui::{Button, CentralPanel, Context, ScrollArea, TextWrapMode, TopBottomPanel, Ui};
use egui_extras::{Column, TableBuilder};
use hotkeys::{consume_shortcuts, shortcut_modal};
use playback_controls::playback_panel;
use rfd::FileDialog;
use workspace_select::workspace_tabs;

const TBL_ROW_H: f32 = 16.;

pub(crate) fn draw_gui(ctx: &Context, app: &mut SfontPlayer) {
    // Show modals
    about_modal(ctx, app);
    shortcut_modal(ctx, app);

    // Hotkeys
    consume_shortcuts(ctx, app);

    handle_dropped_files(ctx);
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
                disable_if_modal(ui, app);
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
                disable_if_modal(ui, app);
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
                disable_if_modal(ui, app);
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
        disable_if_modal(ui, app);
        song_table(ui, app);
    });
}

/// TODO: Drag files into the window to add them
/// https://github.com/sevonj/sfontplayer/issues/7
fn handle_dropped_files(ctx: &Context) {
    ctx.input(|i| {
        for file in i.raw.dropped_files.clone() {
            println!("{:?}", file)
        }
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
                            if ui
                                .add(
                                    Button::new(name)
                                        .frame(highlight)
                                        .wrap_mode(TextWrapMode::Truncate),
                                )
                                .clicked()
                            {
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
                            if ui
                                .add(
                                    Button::new(filename)
                                        .frame(is_selected)
                                        .wrap_mode(TextWrapMode::Truncate),
                                )
                                .clicked()
                            {
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

/// This will disable the UI if a modal window is open
fn disable_if_modal(ui: &mut Ui, app: &SfontPlayer) {
    if app.show_about_modal {
        ui.disable();
    }
    if app.show_shortcut_modal {
        ui.disable();
    }
}
