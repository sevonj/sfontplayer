use crate::player::{
    workspace::{FileListMode, FontSort},
    Player,
};
use eframe::egui::{Align, Button, ComboBox, Label, Layout, RichText, Sense, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use rfd::FileDialog;
use size_format::SizeFormatterBinary;

use super::TBL_ROW_H;

pub fn font_titlebar(ui: &mut Ui, player: &mut Player) {
    ui.horizontal(|ui| {
        // Manually add files
        if player.get_workspace().get_font_list_mode() == FileListMode::Manual {
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
                        player.get_workspace_mut().add_font(path);
                    }
                }
            }
        }
        // Select directory
        else {
            let folder_text = if player.get_workspace().get_font_dir().is_some() {
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
                    player.get_workspace_mut().set_font_dir(path);
                }
            }
        }

        // Title
        ui.heading("Soundfonts");

        // Dir path
        if player.get_workspace().get_font_list_mode() != FileListMode::Manual {
            if ui
                .add(Button::new("🔃").frame(false))
                .on_hover_text("Refresh content")
                .clicked()
            {
                player.get_workspace_mut().refresh_font_list();
            }

            if let Some(dir) = &player.get_workspace().get_font_dir() {
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
            let mut list_mode = player.get_workspace().get_font_list_mode();
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
            if list_mode != player.get_workspace().get_font_list_mode() {
                player.get_workspace_mut().set_font_list_type(list_mode);
            }
        });
    });
}

#[allow(clippy::too_many_lines)]
pub fn soundfont_table(ui: &mut Ui, player: &mut Player) {
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
        .column(Column::exact(16.))
        .column(Column::auto().resizable(true))
        .column(Column::remainder());

    let table = tablebuilder.header(20.0, |mut header| {
        let font_sort = player.get_workspace().get_font_sort();

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
                let filesize = fontref.get_size();
                let status = fontref.get_status();
                let manual_files =
                    player.get_workspace().get_font_list_mode() == FileListMode::Manual;

                row.set_selected(Some(index) == player.get_workspace().get_font_idx());

                // Remove button
                row.col(|ui| {
                    if manual_files
                        && ui
                            .add(Button::new("❎").frame(false))
                            .on_hover_text("Remove")
                            .clicked()
                    {
                        let _ = player.get_workspace_mut().remove_font(index);
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
                    if ui.button("Remove").clicked() {
                        let _ = player.get_workspace_mut().remove_font(index);
                        ui.close_menu();
                    }
                    if ui.button("Go to directory").clicked() {
                        let filepath = player.get_workspace().get_fonts()[index].get_path();
                        let _ = open::that(filepath.parent().expect("Can't open parent"));
                        ui.close_menu();
                    }
                });
            },
        );
    });
}
