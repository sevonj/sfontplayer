use eframe::egui::{Button, Label, Layout, RichText, Sense, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use size_format::SizeFormatterBinary;

use super::{
    actions,
    custom_controls::{circle_button, collapse_button, subheading},
    GuiState, TBL_ROW_H,
};
use crate::player::{
    playlist::{enums::FileListMode, font_meta::FontMeta},
    soundfont_list::FontSort,
    Player,
};

#[allow(clippy::too_many_lines)]
pub fn soundfont_table(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.horizontal(|ui| {
        ui.add(collapse_button(&mut gui.show_playlist_fonts));
        ui.add(subheading("Playlist soundfonts"))
            .on_hover_text("Soundfonts included with the playlist");
    });

    ui.add_space(4.);

    if !gui.show_playlist_fonts {
        return;
    }

    content_controls(ui, player);

    ui.separator();

    let is_active_playlist =
        !player.is_playing() || player.get_playlist_idx() == player.get_playing_playlist_idx();
    if !is_active_playlist {
        // Less intense gray highlight if not active
        ui.style_mut().visuals.selection.bg_fill = ui.style().visuals.widgets.active.bg_fill;
        ui.style_mut().visuals.selection.stroke = ui.style().visuals.widgets.active.fg_stroke;
    }
    let manual_files = player.get_playlist().get_font_list_mode() == FileListMode::Manual;

    let name_w = ui.available_width() - 64.;

    let tablebuilder = TableBuilder::new(ui)
        .striped(true)
        .sense(Sense::click())
        .column(Column::exact(if manual_files { 16. } else { 0. }))
        .column(Column::exact(name_w))
        .column(Column::remainder());

    let table = tablebuilder.header(20.0, |mut header| {
        let font_sort = player.get_playlist().get_font_sort();

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
                player.get_playlist_mut().set_font_sort(match font_sort {
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
            let widget = Button::new(title)
                .frame(false)
                .wrap_mode(TextWrapMode::Extend);
            if ui.add(widget).clicked() {
                player.get_playlist_mut().set_font_sort(match font_sort {
                    FontSort::SizeAsc => FontSort::SizeDesc,
                    _ => FontSort::SizeAsc,
                });
            }
        });
    });

    table.body(|body| {
        body.rows(
            TBL_ROW_H,
            player.get_playlist().get_fonts().len() + 1,
            |mut row| {
                if row.index() == 0 {
                    default_font_item(&mut row, player);
                    return;
                }
                let index = row.index() - 1;
                let fontref = &player.get_playlist().get_fonts()[index];
                let filename = fontref.get_name();
                let filepath = fontref.get_path();
                let filesize = fontref.get_size();
                let status = fontref.get_status();

                row.set_selected(Some(index) == player.get_playlist().get_font_idx());

                // Remove button
                row.col(|ui| {
                    if manual_files
                        && ui
                            .add(Button::new("❎").frame(false))
                            .on_hover_text("Remove")
                            .clicked()
                    {
                        let _ = player.get_playlist_mut().remove_font(index);
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
                    let _ = player.get_playlist_mut().set_font_idx(Some(index));
                }
                // Context menu
                row.response().context_menu(|ui| {
                    if ui.button("Refresh").clicked() {
                        player.get_playlist_mut().get_fonts_mut()[index].refresh();
                        ui.close_menu();
                    }
                    ui.add_enabled_ui(
                        player.get_playlist().get_font_list_mode() == FileListMode::Manual,
                        |ui| {
                            if ui.button("Remove").clicked() {
                                let _ = player.get_playlist_mut().remove_font(index);
                                ui.close_menu();
                            }
                        },
                    );
                    actions::open_file_dir(
                        ui,
                        &player.get_playlist().get_fonts()[index].get_path(),
                        gui,
                    );
                    ui.menu_button("Add to playlist", |ui| {
                        let filepath = player.get_playlist().get_fonts()[index].get_path();
                        if ui.button("➕ New playlist").clicked() {
                            player.new_playlist();
                            let playlist_index = player.get_playlists().len() - 1;
                            let _ = player.get_playlists_mut()[playlist_index]
                                .add_font(filepath.clone());
                        }
                        for i in 0..player.get_playlists().len() {
                            if i == player.get_playlist_idx() {
                                continue;
                            }
                            let playlist = &player.get_playlists_mut()[i];

                            let already_contains = playlist.contains_font(&filepath);
                            let dir_list = playlist.get_font_list_mode() != FileListMode::Manual;

                            let hovertext = if dir_list {
                                "Can't manually add files to directory list."
                            } else if already_contains {
                                "Playlist already contains this file."
                            } else {
                                ""
                            };

                            if ui
                                .add_enabled(
                                    !already_contains && !dir_list,
                                    Button::new(&playlist.name),
                                )
                                .on_disabled_hover_text(hovertext)
                                .clicked()
                            {
                                let _ = player.get_playlists_mut()[i].add_font(filepath.clone());
                                ui.close_menu();
                            }
                        }
                    });
                    if ui.button("Copy path").clicked() {
                        ui.output_mut(|o| o.copied_text = filepath.to_string_lossy().into());
                        ui.close_menu();
                        gui.toast_success("Copied");
                    }
                    if ui
                        .add_enabled(
                            !player.font_lib.contains_font(&filepath),
                            Button::new("Add to library"),
                        )
                        .on_disabled_hover_text("Already in library")
                        .clicked()
                    {
                        let _ = player
                            .font_lib
                            .add_path(player.get_playlist().get_fonts()[index].get_path());
                        ui.close_menu();
                    }
                });
            },
        );
    });
}

fn content_controls(ui: &mut Ui, player: &mut Player) {
    ui.horizontal(|ui| {
        let mut list_mode = player.get_playlist().get_font_list_mode();
        ui.add(actions::content_mode_selector(&mut list_mode));
        if list_mode != player.get_playlist().get_font_list_mode() {
            player.get_playlist_mut().set_font_list_mode(list_mode);
        }

        ui.with_layout(Layout::right_to_left(eframe::egui::Align::Center), |ui| {
            if player.get_playlist().get_font_list_mode() != FileListMode::Manual {
                if let Some(path) =
                    actions::pick_dir_button(player.get_playlist().get_font_dir(), ui)
                {
                    player.get_playlist_mut().set_font_dir(path);
                }
                if circle_button("🔃", ui)
                    .on_hover_text("Refresh content")
                    .clicked()
                {
                    player.get_playlist_mut().refresh_font_list();
                }
            } else if let Some(paths) = actions::pick_soundfonts_button(ui) {
                for path in paths {
                    player.get_playlist_mut().set_font_dir(path);
                }
            }
        });
    });
}

fn default_font_item(row: &mut egui_extras::TableRow<'_, '_>, player: &mut Player) {
    row.set_selected(player.get_playlist().get_font_idx().is_none());

    // Remove button
    row.col(|_| {});
    // Filename
    row.col(|ui| {
        ui.horizontal(|ui| {
            let font_ok;
            if player.get_default_soundfont().is_none() {
                ui.label(RichText::new("？"))
                    .on_hover_text("No default soundfont set.");
                font_ok = false;
            } else if let Err(e) = &player
                .get_default_soundfont()
                .map_or_else(|| Ok(()), FontMeta::get_status)
            {
                ui.label(RichText::new("？")).on_hover_text(e.to_string());
                font_ok = false;
            } else {
                font_ok = true;
            }
            let filename = player
                .get_default_soundfont()
                .map_or("None".into(), FontMeta::get_name);
            let text = format!("None (use default: {filename})");
            let filename_response = ui.add_enabled(
                font_ok,
                Label::new(RichText::new(text).weak())
                    .wrap_mode(TextWrapMode::Truncate)
                    .selectable(false),
            );
            if let Some(font) = &player.get_default_soundfont() {
                filename_response
                    .on_hover_text(font.get_path().to_string_lossy())
                    .on_disabled_hover_text(font.get_path().to_string_lossy());
            }
        });
    });
    // File size
    row.col(|_| {});

    // Select
    if row.response().clicked() {
        let _ = player.get_playlist_mut().set_font_idx(None);
    }
}
