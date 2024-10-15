use crate::player::{
    workspace::enums::{FileListMode, FontSort},
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
                        let _ = player.get_workspace_mut().add_font(path);
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
            player.get_workspace().get_fonts().len() + 1,
            |mut row| {
                if row.index() == 0 {
                    default_font_item(&mut row, player);
                    return;
                }
                let index = row.index() - 1;
                let fontref = &player.get_workspace().get_fonts()[index];
                let filename = fontref.get_name();
                let filepath = fontref.get_path();
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
                    if ui.button("Go to directory").clicked() {
                        let filepath = player.get_workspace().get_fonts()[index].get_path();
                        let _ = open::that(filepath.parent().expect("Can't open parent"));
                        ui.close_menu();
                    }
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

fn default_font_item(row: &mut egui_extras::TableRow<'_, '_>, player: &mut Player) {
    row.set_selected(player.get_workspace().get_font_idx().is_none());

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
                .map_or_else(|| Ok(()), |font| font.get_status())
            {
                ui.label(RichText::new("？")).on_hover_text(e.to_string());
                font_ok = false;
            } else {
                font_ok = true;
            }
            let filename = player
                .get_default_soundfont()
                .map_or("None".into(), |font| font.get_name());
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
        let _ = player.get_workspace_mut().set_font_idx(None);
    }
}
