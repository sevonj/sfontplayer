use eframe::egui::{Button, Label, RichText, Sense, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use size_format::SizeFormatterBinary;

use super::{actions, TBL_ROW_H};
use crate::{
    player::{soundfont_library::FontLibrary, soundfont_list::FontSort, Player},
    GuiState,
};

pub fn soundfont_library(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.heading("Soundfont Library");
    ui.separator();

    if player.font_lib.get_fonts().is_empty() {
        empty_lib_placeholder(ui, gui);
    } else {
        soundfont_table(ui, &mut player.font_lib, gui);
    }
}

fn empty_lib_placeholder(ui: &mut Ui, gui: &mut GuiState) {
    ui.vertical_centered(|ui| {
        ui.add_space(24.);
        ui.label("No soundfonts in library.");
        ui.add_space(16.);
        if ui.button("Manage paths").clicked() {
            gui.show_settings_modal = true;
        }
    });
}

fn soundfont_table(ui: &mut Ui, font_lib: &mut FontLibrary, gui: &mut GuiState) {
    let name_w = ui.available_width() - 64.;
    let tablebuilder = TableBuilder::new(ui)
        .striped(true)
        .sense(Sense::click())
        .column(Column::exact(name_w))
        .column(Column::remainder());

    let table = tablebuilder.header(20.0, |mut header| {
        let font_sort = font_lib.get_sort();

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
                font_lib.set_sort(match font_sort {
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
                font_lib.set_sort(match font_sort {
                    FontSort::SizeAsc => FontSort::SizeDesc,
                    _ => FontSort::SizeAsc,
                });
            }
        });
    });

    table.body(|body| {
        body.rows(TBL_ROW_H, font_lib.get_fonts().len(), |mut row| {
            let index = row.index();
            let fontref = &font_lib.get_fonts()[index];
            let filename = fontref.get_name();
            let filepath = fontref.get_path();
            let filesize = fontref.get_size();
            let status = fontref.get_status();

            row.set_selected(Some(index) == font_lib.get_selected_index());

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
                let _ = font_lib.select(Some(index));
                println!("{:?}", font_lib.get_selected_index());
            }
            // Context menu
            row.response().context_menu(|ui| {
                //if ui.button("Refresh").clicked() {
                //    font_lib.get_workspace_mut().get_fonts_mut()[index].refresh();
                //    ui.close_menu();
                //}
                actions::open_file_dir(ui, &font_lib.get_fonts()[index].get_path(), gui);

                /*ui.menu_button("Add to workspace", |ui| {
                    let filepath = font_lib.get_fonts()[index].get_path();
                    if ui.button("➕ New workspace").clicked() {
                        font_lib.new_workspace();
                        let workspace_index = font_lib.get_workspaces().len() - 1;
                        let _ = font_lib.get_workspaces_mut()[workspace_index]
                            .add_font(filepath.clone());
                    }
                    for i in 0..font_lib.get_workspaces().len() {
                        if i == font_lib.get_workspace_idx() {
                            continue;
                        }
                        let workspace = &font_lib.get_workspaces_mut()[i];

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
                            let _ = font_lib.get_workspaces_mut()[i].add_font(filepath.clone());
                            ui.close_menu();
                        }
                    }
                });*/
                if ui.button("Copy path").clicked() {
                    ui.output_mut(|o| o.copied_text = filepath.to_string_lossy().into());
                    ui.close_menu();
                    gui.toast_success("Copied");
                }
            });
        });
    });
}
