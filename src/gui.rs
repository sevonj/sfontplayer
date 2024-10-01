mod about;
pub mod conversions;
mod cooltoolbar;
mod keyboard_shortcuts;
mod playback_controls;
mod workspace_select;

use crate::{
    workspace::{FileListMode, FontSort, SongSort},
    SfontPlayer,
};
use about::about_modal;
use conversions::format_duration;
use cooltoolbar::toolbar;
use eframe::egui::{
    Align, Button, CentralPanel, ComboBox, Context, Label, Layout, RichText, Sense, TextWrapMode,
    TopBottomPanel, Ui,
};
use egui_extras::{Column, TableBuilder};
use keyboard_shortcuts::{consume_shortcuts, shortcut_modal};
use playback_controls::playback_panel;
use rfd::FileDialog;
use size_format::SizeFormatterBinary;
use std::time::Duration;
use workspace_select::workspace_tabs;

const TBL_ROW_H: f32 = 16.;

#[allow(clippy::too_many_lines)]
pub fn draw_gui(ctx: &Context, app: &mut SfontPlayer) {
    // Show modals
    about_modal(ctx, app);
    shortcut_modal(ctx, app);

    // Keyboard shortcuts
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
                            ui.label(dir.to_string_lossy()).context_menu(|ui| {
                                if ui.button("Go to directory").clicked() {
                                    let _ = open::that(dir);
                                    ui.close_menu();
                                }
                            });
                        } else {
                            ui.label("No directory.");
                        }
                    }

                    // Content mode select
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let mut list_mode = app.get_workspace().get_font_list_mode();
                        ComboBox::from_id_salt("mode_select")
                            .selected_text(format!("Content: {list_mode:?}"))
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
                if app.get_workspace().get_song_list_mode() == FileListMode::Manual {
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
                                app.get_workspace_mut().add_song(path);
                            }
                        }
                    }
                }
                // Select directory
                else {
                    let folder_text = if app.get_workspace().get_song_dir().is_some() {
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
                            app.get_workspace_mut().set_song_dir(path);
                        }
                    }
                }

                // Title
                ui.heading("Midi files");

                // Dir path
                if app.get_workspace().get_song_list_mode() != FileListMode::Manual {
                    if ui
                        .add(Button::new("🔃").frame(false))
                        .on_hover_text("Refresh content")
                        .clicked()
                    {
                        app.get_workspace_mut().refresh_song_list();
                    }

                    if let Some(dir) = &app.get_workspace().get_song_dir() {
                        ui.label(dir.to_string_lossy()).context_menu(|ui| {
                            if ui.button("Go to directory").clicked() {
                                let _ = open::that(dir);
                                ui.close_menu();
                            }
                        });
                    } else {
                        ui.label("No directory.");
                    }
                }

                // Content mode select
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut list_mode = app.get_workspace().get_song_list_mode();
                    ComboBox::from_id_salt("mode_select")
                        .selected_text(format!("Content: {list_mode:?}"))
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
                    if list_mode != app.get_workspace().get_song_list_mode() {
                        app.get_workspace_mut().set_song_list_mode(list_mode);
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
/// <https://github.com/sevonj/sfontplayer/issues/7>
fn handle_dropped_files(ctx: &Context) {
    ctx.input(|i| {
        for file in i.raw.dropped_files.clone() {
            println!("{file:?}");
        }
    });
}

#[allow(clippy::too_many_lines)]
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
        .column(Column::auto().resizable(true))
        .column(Column::remainder());

    let table = tablebuilder.header(20.0, |mut header| {
        let font_sort = app.get_workspace().get_font_sort();

        header.col(|_| {});
        header.col(|ui| {
            let title = match font_sort {
                FontSort::NameAsc => "Name ⏶",
                FontSort::NameDesc => "Name ⏷",
                _ => "Name",
            };
            if ui
                .add(
                    Button::new(title)
                        .frame(false)
                        .wrap_mode(TextWrapMode::Extend),
                )
                .clicked()
            {
                app.get_workspace_mut().set_font_sort(match font_sort {
                    FontSort::NameAsc => FontSort::NameDesc,
                    _ => FontSort::NameAsc,
                });
            }
        });
        header.col(|ui| {
            let title = match font_sort {
                FontSort::SizeAsc => "Size ⏶",
                FontSort::SizeDesc => "Size ⏷",
                _ => "Size",
            };
            if ui
                .add(
                    Button::new(title)
                        .frame(false)
                        .wrap_mode(TextWrapMode::Extend),
                )
                .clicked()
            {
                app.get_workspace_mut().set_font_sort(match font_sort {
                    FontSort::SizeAsc => FontSort::SizeDesc,
                    _ => FontSort::SizeAsc,
                });
            }
        });
    });

    table.body(|body| {
        body.rows(
            TBL_ROW_H,
            app.get_workspace().get_fonts().len(),
            |mut row| {
                let index = row.index();
                let fontref = &app.get_workspace().get_fonts()[index];
                let filename = fontref.get_name();
                let filesize = fontref.get_size();
                let error = fontref.get_error();
                let manual_files = app.get_workspace().get_font_list_mode() == FileListMode::Manual;

                row.set_selected(Some(index) == app.get_workspace().get_font_idx());

                // Remove button
                row.col(|ui| {
                    if manual_files
                        && ui
                            .add(Button::new("❎").frame(false))
                            .on_hover_text("Remove")
                            .clicked()
                    {
                        let _ = app.get_workspace_mut().remove_font(index);
                    }
                });
                // Filename
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        if let Some(e) = &error {
                            ui.label(RichText::new("？")).on_hover_text(e.to_string());
                        }
                        if ui
                            .add_enabled(
                                error.is_none(),
                                Button::new(filename)
                                    .frame(false)
                                    .wrap_mode(TextWrapMode::Truncate),
                            )
                            .clicked()
                        {
                            let _ = app.get_workspace_mut().set_font_idx(Some(index));
                        }
                    });
                });

                // File size
                row.col(|ui| {
                    let size_str = filesize.map_or_else(
                        || "??".into(),
                        |size| format!("{}B", SizeFormatterBinary::new(size)),
                    );
                    ui.add(Label::new(size_str).wrap_mode(TextWrapMode::Extend));
                });

                // TODO: Find out why this doesn't work
                if row.response().clicked() {
                    println!("CLICK");
                    let _ = app.get_workspace_mut().set_font_idx(Some(index));
                }

                // Context menu
                row.response().context_menu(|ui| {
                    if ui.button("Refresh").clicked() {
                        app.get_workspace_mut().get_fonts_mut()[index].refresh();
                        ui.close_menu();
                    }
                    if ui.button("Remove").clicked() {
                        let _ = app.get_workspace_mut().remove_font(index);
                        ui.close_menu();
                    }
                    if ui.button("Go to directory").clicked() {
                        let filepath = app.get_workspace().get_fonts()[index].get_path();
                        let _ = open::that(filepath.parent().expect("Can't open parent"));
                        ui.close_menu();
                    }
                });
            },
        );
    });
}

#[allow(clippy::too_many_lines)]
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
        .column(Column::auto_with_initial_suggestion(96.).resizable(true))
        .column(Column::remainder())
        .sense(Sense::click());

    if app.update_flags.scroll_to_song {
        if let Some(index) = app.get_workspace().get_song_idx() {
            tablebuilder = tablebuilder.scroll_to_row(index, Some(Align::Center));
        }
    }

    let table = tablebuilder.header(20.0, |mut header| {
        let song_sort = app.get_workspace().get_song_sort();

        header.col(|_| {});
        header.col(|ui| {
            let title = match song_sort {
                SongSort::NameAsc => "Name ⏶",
                SongSort::NameDesc => "Name ⏷",
                _ => "Name",
            };
            if ui
                .add(
                    Button::new(title)
                        .frame(false)
                        .wrap_mode(TextWrapMode::Extend),
                )
                .clicked()
            {
                app.get_workspace_mut().set_song_sort(match song_sort {
                    SongSort::NameAsc => SongSort::NameDesc,
                    _ => SongSort::NameAsc,
                });
            }
        });
        header.col(|ui| {
            let title = match song_sort {
                SongSort::TimeAsc => "Time ⏶",
                SongSort::TimeDesc => "Time ⏷",
                _ => "Time",
            };
            if ui
                .add(
                    Button::new(title)
                        .frame(false)
                        .wrap_mode(TextWrapMode::Extend),
                )
                .clicked()
            {
                app.get_workspace_mut().set_song_sort(match song_sort {
                    SongSort::TimeAsc => SongSort::TimeDesc,
                    _ => SongSort::TimeAsc,
                });
            }
        });
        header.col(|ui| {
            let title = match song_sort {
                SongSort::SizeAsc => "Size ⏶",
                SongSort::SizeDesc => "Size ⏷",
                _ => "Size",
            };
            if ui
                .add(
                    Button::new(title)
                        .frame(false)
                        .wrap_mode(TextWrapMode::Extend),
                )
                .clicked()
            {
                app.get_workspace_mut().set_song_sort(match song_sort {
                    SongSort::SizeAsc => SongSort::SizeDesc,
                    _ => SongSort::SizeAsc,
                });
            }
        });
    });

    table.body(|body| {
        body.rows(
            TBL_ROW_H,
            app.get_workspace().get_songs().len(),
            |mut row| {
                let index = row.index();
                let midiref = &app.get_workspace().get_songs()[index];
                let filename = midiref.get_name();
                let filesize = midiref.get_size();
                let error = midiref.get_error();
                let manual_files = app.get_workspace().get_song_list_mode() == FileListMode::Manual;

                let time = app.get_workspace().get_songs()[index]
                    .get_duration()
                    .unwrap_or(Duration::ZERO);

                row.set_selected(Some(index) == app.get_workspace().get_song_idx());

                // Remove button
                row.col(|ui| {
                    if manual_files
                        && ui
                            .add(Button::new("❎").frame(false))
                            .on_hover_text("Remove")
                            .clicked()
                    {
                        let _ = app.get_workspace_mut().remove_song(index);
                    }
                });
                // Filename
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        if let Some(e) = &error {
                            ui.label(RichText::new("？")).on_hover_text(e.to_string());
                        }
                        if ui
                            .add_enabled(
                                error.is_none(),
                                Button::new(filename)
                                    .frame(false)
                                    .wrap_mode(TextWrapMode::Truncate),
                            )
                            .clicked()
                        {
                            let _ = app.get_workspace_mut().set_song_idx(Some(index));
                            app.start();
                        }
                    });
                });
                // Duration
                row.col(|ui| {
                    ui.add(Label::new(format_duration(time)).wrap_mode(TextWrapMode::Extend));
                });
                // File size
                row.col(|ui| {
                    let size_str = filesize.map_or_else(
                        || "??".into(),
                        |size| format!("{}B", SizeFormatterBinary::new(size)),
                    );
                    ui.add(Label::new(size_str).wrap_mode(TextWrapMode::Extend));
                });

                // TODO: Find out why this doesn't work
                if row.response().clicked() {
                    println!("CLICK");
                    let _ = app.get_workspace_mut().set_song_idx(Some(index));
                    app.start();
                }

                // Context menu
                row.response().context_menu(|ui| {
                    if ui.button("Refresh").clicked() {
                        app.get_workspace_mut().get_songs_mut()[index].refresh();
                        ui.close_menu();
                    }
                    if ui.button("Remove").clicked() {
                        let _ = app.get_workspace_mut().remove_song(index);
                        ui.close_menu();
                    }
                    if ui.button("Go to directory").clicked() {
                        let filepath = app.get_workspace().get_songs()[index].get_path();
                        let _ = open::that(filepath.parent().expect("Can't open parent"));
                        ui.close_menu();
                    }
                });
            },
        );
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
