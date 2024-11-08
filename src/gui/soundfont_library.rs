use eframe::egui::{Button, Label, RichText, Sense, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use size_format::SizeFormatterBinary;

use super::{actions, TBL_ROW_H};
use crate::{
    player::{
        workspace::{enums::FileListMode, font_meta::FontSort},
        Player,
    },
    GuiState,
};

pub fn soundfont_library(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.heading("Soundfont Library");
    soundfont_table(ui, player, gui);
}

fn soundfont_table(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    let is_active_workspace =
        !player.is_playing() || player.get_workspace_idx() == player.get_playing_workspace_idx();
    if !is_active_workspace {
        // Less intense gray highlight if not active
        ui.style_mut().visuals.selection.bg_fill = ui.style().visuals.widgets.active.bg_fill;
        ui.style_mut().visuals.selection.stroke = ui.style().visuals.widgets.active.fg_stroke;
    }

    let tablebuilder = TableBuilder::new(ui)
        .striped(true)
        .sense(Sense::click())
        .column(Column::exact(192.))
        .column(Column::exact(48.));

    let table = tablebuilder.header(20.0, |mut header| {
        let font_sort = player.get_workspace().get_font_sort();

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
                player.get_workspace_mut().set_font_sort(match font_sort {
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
                player.get_workspace_mut().set_font_sort(match font_sort {
                    FontSort::SizeAsc => FontSort::SizeDesc,
                    _ => FontSort::SizeAsc,
                });
            }
        });
    });

    table.body(|body| {
        body.rows(
            TBL_ROW_H,
            player.get_workspace().get_fonts().len(),
            |mut row| {
                let index = row.index();
                let fontref = &player.get_workspace().get_fonts()[index];
                let filename = fontref.get_name();
                let filepath = fontref.get_path();
                let filesize = fontref.get_size();
                let status = fontref.get_status();

                row.set_selected(Some(index) == player.get_workspace().get_font_idx());

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
                    let _ = player.get_workspace_mut().set_font_idx(Some(index));
                }
                // Context menu
                row.response().context_menu(|ui| {
                    if ui.button("Refresh").clicked() {
                        player.get_workspace_mut().get_fonts_mut()[index].refresh();
                        ui.close_menu();
                    }
                    ui.add_enabled_ui(
                        player.get_workspace().get_font_list_mode() == FileListMode::Manual,
                        |ui| {
                            if ui.button("Remove").clicked() {
                                let _ = player.get_workspace_mut().remove_font(index);
                                ui.close_menu();
                            }
                        },
                    );
                    actions::open_file_dir(
                        ui,
                        &player.get_workspace().get_fonts()[index].get_path(),
                        gui,
                    );
                    ui.menu_button("Add to workspace", |ui| {
                        let filepath = player.get_workspace().get_fonts()[index].get_path();
                        if ui.button("➕ New workspace").clicked() {
                            player.new_workspace();
                            let workspace_index = player.get_workspaces().len() - 1;
                            let _ = player.get_workspaces_mut()[workspace_index]
                                .add_font(filepath.clone());
                        }
                        for i in 0..player.get_workspaces().len() {
                            if i == player.get_workspace_idx() {
                                continue;
                            }
                            let workspace = &player.get_workspaces_mut()[i];

                            let already_contains = workspace.contains_font(&filepath);
                            let dir_list = workspace.get_font_list_mode() != FileListMode::Manual;

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
                                let _ = player.get_workspaces_mut()[i].add_font(filepath.clone());
                                ui.close_menu();
                            }
                        }
                    });
                    if ui.button("Copy path").clicked() {
                        ui.output_mut(|o| o.copied_text = filepath.to_string_lossy().into());
                        ui.close_menu();
                        gui.toast_success("Copied");
                    }
                    if ui.button("Make default").clicked() {
                        player.set_default_soundfont(Some(
                            player.get_workspace().get_fonts()[index].clone(),
                        ));
                        ui.close_menu();
                    }
                });
            },
        );
    });
}
