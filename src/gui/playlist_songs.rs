use eframe::egui::{Align, Button, Label, Layout, RichText, Sense, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use size_format::SizeFormatterBinary;
use std::time::Duration;

use super::{
    actions,
    conversions::format_duration,
    custom_controls::{circle_button, subheading},
    GuiState, TBL_ROW_H,
};
use crate::player::{
    workspace::enums::{FileListMode, SongSort},
    Player,
};

#[allow(clippy::too_many_lines)]
pub fn playlist_song_panel(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.horizontal(|ui| {
        ui.add(subheading("Playlist"));
        content_controls(ui, player);
    });

    ui.separator();

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
        .column(Column::initial(width).resizable(true))
        .column(Column::initial(96.).resizable(true))
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
                let filepath = midiref.get_path();
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
                        )
                        .on_hover_text(filepath.to_string_lossy())
                        .on_disabled_hover_text(filepath.to_string_lossy());
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
                    ui.add_enabled_ui(
                        player.get_workspace().get_song_list_mode() == FileListMode::Manual,
                        |ui| {
                            if ui.button("Remove").clicked() {
                                let _ = player.get_workspace_mut().remove_song(index);
                                ui.close_menu();
                            }
                        },
                    );
                    actions::open_file_dir(
                        ui,
                        &player.get_workspace().get_songs()[index].get_path(),
                        gui,
                    );
                    ui.menu_button("Add to workspace", |ui| {
                        let filepath = player.get_workspace().get_songs()[index].get_path();
                        if ui.button("➕ New workspace").clicked() {
                            player.new_workspace();
                            let workspace_index = player.get_workspaces().len() - 1;
                            let _ = player.get_workspaces_mut()[workspace_index]
                                .add_song(filepath.clone());
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
                                let _ = player.get_workspaces_mut()[i].add_song(filepath.clone());
                                ui.close_menu();
                            }
                        }
                    });
                    if ui.button("Copy path").clicked() {
                        ui.output_mut(|o| o.copied_text = filepath.to_string_lossy().into());
                        ui.close_menu();
                        gui.toast_success("Copied");
                    }
                });
            },
        );
    });
}

fn content_controls(ui: &mut Ui, player: &mut Player) {
    ui.horizontal(|ui| {
        let mut list_mode = player.get_workspace().get_song_list_mode();
        ui.add(actions::content_mode_selector(&mut list_mode));
        if list_mode != player.get_workspace().get_song_list_mode() {
            player.get_workspace_mut().set_song_list_mode(list_mode);
        }

        ui.with_layout(Layout::right_to_left(eframe::egui::Align::Center), |ui| {
            if player.get_workspace().get_song_list_mode() != FileListMode::Manual {
                if let Some(path) =
                    actions::pick_dir_button(player.get_workspace().get_song_dir(), ui)
                {
                    player.get_workspace_mut().set_song_dir(path);
                }
                if circle_button("🔃", ui)
                    .on_hover_text("Refresh content")
                    .clicked()
                {
                    player.get_workspace_mut().refresh_song_list();
                }
            } else if let Some(paths) = actions::pick_midifiles_button(ui) {
                for path in paths {
                    player.get_workspace_mut().set_song_dir(path);
                }
            }
        });
    });
}