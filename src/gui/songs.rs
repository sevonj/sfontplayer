use super::{conversions::format_duration, GuiState, TBL_ROW_H};
use crate::player::{
    workspace::enums::{FileListMode, SongSort},
    Player,
};
use eframe::egui::{Align, Button, ComboBox, Label, Layout, RichText, Sense, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;
use size_format::SizeFormatterBinary;
use std::time::Duration;

pub fn song_titlebar(ui: &mut Ui, player: &mut Player) {
    ui.horizontal(|ui| {
        // Manually add files
        if player.get_workspace().get_song_list_mode() == FileListMode::Manual {
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
                        player.get_workspace_mut().add_song(path);
                    }
                }
            }
        }
        // Select directory
        else {
            let folder_text = if player.get_workspace().get_song_dir().is_some() {
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
                    player.get_workspace_mut().set_song_dir(path);
                }
            }
        }

        // Title
        ui.heading("Midi files");

        // Dir path
        if player.get_workspace().get_song_list_mode() != FileListMode::Manual {
            if ui
                .add(Button::new("🔃").frame(false))
                .on_hover_text("Refresh content")
                .clicked()
            {
                player.get_workspace_mut().refresh_song_list();
            }

            if let Some(dir) = &player.get_workspace().get_song_dir() {
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
            let mut list_mode = player.get_workspace().get_song_list_mode();
            ComboBox::from_id_salt("mode_select")
                .selected_text(format!("Content: {list_mode:?}"))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut list_mode, FileListMode::Manual, "Manual");
                    ui.selectable_value(&mut list_mode, FileListMode::Directory, "Directory");
                    ui.selectable_value(
                        &mut list_mode,
                        FileListMode::Subdirectories,
                        "Subdirectories",
                    );
                });
            if list_mode != player.get_workspace().get_song_list_mode() {
                player.get_workspace_mut().set_song_list_mode(list_mode);
            }
        });
    });
}

#[allow(clippy::too_many_lines)]
pub fn song_table(ui: &mut Ui, player: &mut Player, gui: &GuiState) {
    let is_active_workspace =
        !player.is_playing() || player.get_workspace_idx() == player.get_playing_workspace_idx();
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

    if gui.update_flags.scroll_to_song {
        if let Some(index) = player.get_workspace().get_song_idx() {
            tablebuilder = tablebuilder.scroll_to_row(index, Some(Align::Center));
        }
    }

    let table = tablebuilder.header(20.0, |mut header| {
        let song_sort = player.get_workspace().get_song_sort();

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
                player.get_workspace_mut().set_song_sort(match song_sort {
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
                player.get_workspace_mut().set_song_sort(match song_sort {
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
                player.get_workspace_mut().set_song_sort(match song_sort {
                    SongSort::SizeAsc => SongSort::SizeDesc,
                    _ => SongSort::SizeAsc,
                });
            }
        });
    });

    table.body(|body| {
        body.rows(
            TBL_ROW_H,
            player.get_workspace().get_songs().len(),
            |mut row| {
                let index = row.index();
                let midiref = &player.get_workspace().get_songs()[index];
                let filename = midiref.get_name();
                let filesize = midiref.get_size();
                let status = midiref.get_status();
                let manual_files =
                    player.get_workspace().get_song_list_mode() == FileListMode::Manual;

                let time = player.get_workspace().get_songs()[index]
                    .get_duration()
                    .unwrap_or(Duration::ZERO);

                row.set_selected(Some(index) == player.get_workspace().get_song_idx());

                // Remove button
                row.col(|ui| {
                    if manual_files
                        && ui
                            .add(Button::new("❎").frame(false))
                            .on_hover_text("Remove")
                            .clicked()
                    {
                        let _ = player.get_workspace_mut().remove_song(index);
                    }
                });
                // Filename
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        if let Err(e) = &status {
                            ui.label(RichText::new("？")).on_hover_text(e.to_string());
                        }
                        ui.add_enabled(
                            status.is_ok(),
                            Label::new(filename)
                                .wrap_mode(TextWrapMode::Truncate)
                                .selectable(false),
                        );
                    });
                });
                // Duration
                row.col(|ui| {
                    ui.add(
                        Label::new(format_duration(time))
                            .wrap_mode(TextWrapMode::Extend)
                            .selectable(false),
                    );
                });
                // File size
                row.col(|ui| {
                    let size_str = filesize.map_or_else(
                        || "??".into(),
                        |size| format!("{}B", SizeFormatterBinary::new(size)),
                    );
                    ui.add(
                        Label::new(size_str)
                            .wrap_mode(TextWrapMode::Extend)
                            .selectable(false),
                    );
                });

                // Select
                if row.response().clicked() {
                    let _ = player.get_workspace_mut().set_song_idx(Some(index));
                    player.start();
                }

                // Context menu
                row.response().context_menu(|ui| {
                    if ui.button("Refresh").clicked() {
                        player.get_workspace_mut().get_songs_mut()[index].refresh();
                        ui.close_menu();
                    }
                    if ui.button("Remove").clicked() {
                        let _ = player.get_workspace_mut().remove_song(index);
                        ui.close_menu();
                    }
                    if ui.button("Go to directory").clicked() {
                        let filepath = player.get_workspace().get_songs()[index].get_path();
                        let _ = open::that(filepath.parent().expect("Can't open parent"));
                        ui.close_menu();
                    }
                    ui.menu_button("Add to workspace", |ui| {
                        let filepath = player.get_workspace().get_songs()[index].get_path();
                        if ui.button("➕ New workspace").clicked() {
                            player.new_workspace();
                            let workspace_index = player.get_workspaces().len() - 1;
                            player.get_workspaces_mut()[workspace_index].add_song(filepath.clone());
                        }
                        for i in 0..player.get_workspaces().len() {
                            if i == player.get_workspace_idx() {
                                continue;
                            }
                            let workspace = &player.get_workspaces_mut()[i];

                            let already_contains = workspace.contains_song(&filepath);
                            let dir_list = workspace.get_song_list_mode() != FileListMode::Manual;

                            let hovertext = if dir_list {
                                "Can't manually add files to directory list."
                            } else if already_contains {
                                "Workspace already contains this file."
                            } else {
                                ""
                            };

                            if ui
                                .add_enabled(
                                    !already_contains && !dir_list,
                                    Button::new(&workspace.name),
                                )
                                .on_disabled_hover_text(hovertext)
                                .clicked()
                            {
                                player.get_workspaces_mut()[i].add_song(filepath.clone());
                                ui.close_menu();
                            }
                        }
                    });
                });
            },
        );
    });
}
