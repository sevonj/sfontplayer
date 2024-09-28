mod about;
pub(crate) mod conversions;
mod cooltoolbar;
mod hotkeys;
mod playback_controls;
mod workspace_select;

use std::time::Duration;

use crate::{data::FileListMode, SfontPlayer};
use about::about_modal;
use conversions::format_duration;
use cooltoolbar::toolbar;
use eframe::egui::{Button, CentralPanel, Context, Sense, TextWrapMode, TopBottomPanel, Ui};
use egui::{Layout, RichText};
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
                    // Manually add files
                    if app.get_workspace().get_font_list_mode() == FileListMode::Manual {
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
                                    app.get_workspace_mut().add_font(path);
                                }
                            }
                        }
                    }
                    // Select directory
                    else {
                        let folder_text = if app.get_workspace().get_font_dir().is_some() {
                            "🗁"
                        } else {
                            "🗀"
                        };
                        if ui
                            .add(Button::new(folder_text).frame(false))
                            .on_hover_text("Select directory")
                            .clicked()
                        {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                app.get_workspace_mut().set_font_dir(path);
                                app.get_workspace_mut().refresh_font_list();
                            }
                        }
                    }

                    // Title
                    ui.heading("Soundfonts");

                    // Dir path
                    if app.get_workspace().get_font_list_mode() != FileListMode::Manual {
                        if ui
                            .add(Button::new("🔃").frame(false))
                            .on_hover_text("Refresh content")
                            .clicked()
                        {
                            app.get_workspace_mut().refresh_font_list();
                        }

                        if let Some(dir) = &app.get_workspace().get_font_dir() {
                            ui.label(dir.to_string_lossy());
                        } else {
                            ui.label("No directory.");
                        }
                    }

                    // Content mode select
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut list_mode = app.get_workspace().get_font_list_mode().clone();
                        egui::ComboBox::from_id_salt("mode_select")
                            .selected_text(format!("Content: {:?}", list_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut list_mode, FileListMode::Manual, "Manual");
                                ui.selectable_value(
                                    &mut list_mode,
                                    FileListMode::Directory,
                                    "Directory",
                                );
                                ui.selectable_value(
                                    &mut list_mode,
                                    FileListMode::Subdirectories,
                                    "Subdirectories",
                                );
                            });
                        if list_mode != app.get_workspace().get_font_list_mode() {
                            app.get_workspace_mut().set_font_list_type(list_mode);
                        }
                    });
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
                // Manually add files
                if app.get_workspace().get_midi_list_mode() == FileListMode::Manual {
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
                                app.get_workspace_mut().add_midi(path);
                            }
                        }
                    }
                }
                // Select directory
                else {
                    let folder_text = if app.get_workspace().get_midi_dir().is_some() {
                        "🗁"
                    } else {
                        "🗀"
                    };
                    if ui
                        .add(Button::new(folder_text).frame(false))
                        .on_hover_text("Select directory")
                        .clicked()
                    {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            app.get_workspace_mut().set_midi_dir(path);
                        }
                    }
                }

                // Title
                ui.heading("Midi files");

                // Dir path
                if app.get_workspace().get_midi_list_mode() != FileListMode::Manual {
                    if ui
                        .add(Button::new("🔃").frame(false))
                        .on_hover_text("Refresh content")
                        .clicked()
                    {
                        app.get_workspace_mut().refresh_midi_list();
                    }

                    if let Some(dir) = &app.get_workspace().get_midi_dir() {
                        ui.label(dir.to_string_lossy());
                    } else {
                        ui.label("No directory.");
                    }
                }

                // Content mode select
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut list_mode = app.get_workspace().get_midi_list_mode().clone();
                    egui::ComboBox::from_id_salt("mode_select")
                        .selected_text(format!("Content: {:?}", list_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut list_mode, FileListMode::Manual, "Manual");
                            ui.selectable_value(
                                &mut list_mode,
                                FileListMode::Directory,
                                "Directory",
                            );
                            ui.selectable_value(
                                &mut list_mode,
                                FileListMode::Subdirectories,
                                "Subdirectories",
                            );
                        });
                    if list_mode != app.get_workspace().get_midi_list_mode() {
                        app.get_workspace_mut().set_midi_list_mode(list_mode);
                    }
                });
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
    let is_active_workspace = !app.is_playing || app.workspace_idx == app.playing_workspace_idx;
    if !is_active_workspace {
        // Less intense gray highlight if not active
        ui.style_mut().visuals.selection.bg_fill = ui.style().visuals.widgets.active.bg_fill;
        ui.style_mut().visuals.selection.stroke = ui.style().visuals.widgets.active.fg_stroke;
    }

    let tablebuilder = TableBuilder::new(ui)
        .striped(true)
        .sense(Sense::click())
        .column(Column::exact(16.))
        .column(Column::remainder());

    let table = tablebuilder.header(20.0, |mut header| {
        header.col(|_| {});
        header.col(|ui| {
            ui.label("Name");
        });
    });

    table.body(|body| {
        body.rows(TBL_ROW_H, app.get_workspace().fonts.len(), |mut row| {
            let index = row.index();
            let fontref = &app.get_workspace().fonts[index];
            let filename = fontref.get_name();
            let is_error = fontref.is_error();
            let manual_files = app.get_workspace().get_font_list_mode() == FileListMode::Manual;

            row.set_selected(Some(index) == app.get_workspace().font_idx);

            // Remove button
            row.col(|ui| {
                if manual_files {
                    if ui
                        .add(Button::new("❎").frame(false))
                        .on_hover_text("Remove")
                        .clicked()
                    {
                        app.get_workspace_mut().remove_font(index)
                    }
                }
            });
            // Filename
            row.col(|ui| {
                let mut filename_richtext = RichText::new(filename);
                if is_error {
                    filename_richtext = filename_richtext.color(ui.visuals().error_fg_color);
                }
                if ui
                    .add(
                        Button::new(filename_richtext)
                            .frame(false)
                            .wrap_mode(TextWrapMode::Truncate),
                    )
                    .clicked()
                {
                    app.get_workspace_mut().font_idx = Some(index);
                }
            });

            // TODO: Find out why this doesn't work
            if row.response().clicked() {
                println!("CLICK");
                app.get_workspace_mut().font_idx = Some(index);
            }
        });
    });
}

fn song_table(ui: &mut Ui, app: &mut SfontPlayer) {
    let is_active_workspace = !app.is_playing || app.workspace_idx == app.playing_workspace_idx;
    if !is_active_workspace {
        // Less intense gray highlight if not active
        ui.style_mut().visuals.selection.bg_fill = ui.style().visuals.widgets.active.bg_fill;
        ui.style_mut().visuals.selection.stroke = ui.style().visuals.widgets.active.fg_stroke;
    }

    let width = ui.available_width() - 192.;

    let mut tablebuilder = TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(16.))
        .column(Column::auto_with_initial_suggestion(width).resizable(true))
        .column(Column::remainder())
        .sense(Sense::click());

    if app.update_flags.scroll_to_song {
        if let Some(index) = app.get_workspace().midi_idx {
            tablebuilder = tablebuilder.scroll_to_row(index, Some(egui::Align::Center))
        }
    }

    let table = tablebuilder.header(20.0, |mut header| {
        header.col(|_| {});
        header.col(|ui| {
            ui.label("Name");
        });
        header.col(|ui| {
            ui.label("Time");
        });
    });

    table.body(|body| {
        body.rows(TBL_ROW_H, app.get_workspace().midis.len(), |mut row| {
            let index = row.index();
            let midiref = &app.get_workspace().midis[index];
            let filename = midiref.get_name();
            let is_error = midiref.is_error();
            let manual_files = app.get_workspace().get_midi_list_mode() == FileListMode::Manual;

            let time = app.get_workspace().midis[index]
                .get_duration()
                .unwrap_or(Duration::ZERO);

            row.set_selected(Some(index) == app.get_workspace().midi_idx);

            // Remove button
            row.col(|ui| {
                if manual_files {
                    if ui
                        .add(Button::new("❎").frame(false))
                        .on_hover_text("Remove")
                        .clicked()
                    {
                        app.get_workspace_mut().remove_midi(index)
                    }
                }
            });
            // Filename
            row.col(|ui| {
                let mut filename_richtext = RichText::new(filename);
                if is_error {
                    filename_richtext = filename_richtext.color(ui.visuals().error_fg_color);
                }
                if ui
                    .add(
                        Button::new(filename_richtext)
                            .frame(false)
                            .wrap_mode(TextWrapMode::Truncate),
                    )
                    .clicked()
                {
                    app.get_workspace_mut().midi_idx = Some(index);
                    app.start();
                }
            });
            // Duration
            row.col(|ui| {
                ui.label(format_duration(time));
            });

            // TODO: Find out why this doesn't work
            if row.response().clicked() {
                println!("CLICK");
                app.get_workspace_mut().midi_idx = Some(index);
                app.start();
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
