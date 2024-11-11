use eframe::egui::{
    lerp, pos2, vec2, Align, Align2, Button, CollapsingHeader, Context, Label, Layout, RichText,
    ScrollArea, Sense, Stroke, TextWrapMode, Ui, Vec2, Widget, WidgetInfo, WidgetType, Window,
};
use egui_extras::{Column, TableBuilder};

use crate::{
    player::{soundfont_library::FontLibrary, Player},
    GuiState,
};

use super::file_dialogs;

pub fn settings_modal(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    let window_size = ctx.input(|i| i.screen_rect()).size() - Vec2 { x: 32., y: 64. };
    let modal_size = window_size.min(Vec2 { x: 600., y: 800. });

    Window::new("Settings")
        .collapsible(false)
        .fixed_size(modal_size)
        .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
        .open(&mut gui.show_settings_modal)
        .show(ctx, |ui| {
            //ui.set_height(window_rect.height());
            ScrollArea::vertical().show(ui, |ui| {
                //ui.set_width(window_rect.width());
                ui.horizontal(|ui| {
                    ui.add_space(8.);
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() - 8.);
                        ui.add_space(8.);

                        category_heading(ui, "General Settings");

                        theme_control(ui);
                        ui.add(toggle_row(
                            "Autosave",
                            "Disable manual saving and use autosave for all workspaces",
                            &mut player.autosave,
                        ));
                        ui.add(toggle_row(
                            "Show developer settings",
                            "These are not useful to normal users",
                            &mut gui.show_developer_options,
                        ));

                        category_heading(ui, "Soundfont library");

                        font_lib_paths(ui, &mut player.font_lib);

                        if ui
                            .add(toggle_row(
                                "Search subdirectories",
                                "Look for soundfonts in subdirectories",
                                &mut player.font_lib.crawl_subdirs,
                            ))
                            .changed()
                        {
                            player.font_lib.refresh_files();
                        };

                        if !gui.show_developer_options {
                            return;
                        }
                        category_heading(ui, "Developer / Debug Settings");

                        ui.add(toggle_row(
                            "Block saving",
                            "Turning this on will prevent anything being saved",
                            &mut player.debug_block_saving,
                        ));
                    });
                });
            });
        });
}

fn category_heading<S>(ui: &mut Ui, title: S)
where
    String: From<S>,
{
    ui.heading(RichText::new(title).strong());
    ui.separator();
    ui.add_space(8.);
}

fn toggle_row<S>(title: S, subtitle: S, on: &mut bool) -> impl Widget + '_
where
    String: From<S>,
{
    let title: String = title.into();
    let subtitle: String = subtitle.into();

    move |ui: &mut Ui| {
        let response = ui
            .with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width() - 32.);
                    ui.heading(title);
                    ui.label(subtitle);
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add(toggle(on))
                })
                .inner
            })
            .inner;

        ui.add_space(8.);

        response
    }
}

pub fn toggle(on: &mut bool) -> impl Widget + '_ {
    move |ui: &mut Ui| {
        let desired_size = ui.spacing().interact_size.y * vec2(2.0, 1.0);
        let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }
        response
            .widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, ui.is_enabled(), *on, ""));

        if ui.is_rect_visible(rect) {
            let anim_t = ui.ctx().animate_bool_responsive(response.id, *on);
            let visuals = ui.style().interact_selectable(&response, *on);
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height();
            ui.painter()
                .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
            let circle_x = lerp((rect.left() + radius)..=(rect.right() - radius), anim_t);
            ui.painter().circle(
                pos2(circle_x, rect.center().y),
                0.75 * radius,
                visuals.fg_stroke.color,
                Stroke::NONE,
            );
        }

        response
    }
}

fn theme_control(ui: &mut Ui) {
    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.vertical(|ui| {
            ui.set_width(ui.available_width() - 32.);
            ui.heading("Theme");
            ui.label("Change theme");
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            eframe::egui::widgets::global_theme_preference_buttons(ui);
            // if ui.ctx().theme() == Theme::Light {
            //     if ui.button("🌙 Toggle theme").clicked() {
            //         ui.ctx().set_theme(ThemePreference::Dark);
            //     }
            // } else if ui.button("☀ Toggle theme").clicked() {
            //     ui.ctx().set_theme(ThemePreference::Light);
            // }
        });
    });
    ui.add_space(8.);
}

fn font_lib_paths(ui: &mut Ui, font_lib: &mut FontLibrary) {
    let title = "Paths";
    let subtitle = "Paths to search soundfonts from";

    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.vertical(|ui| {
            ui.set_width(ui.available_width() - 32.);
            ui.heading(title);
            ui.label(subtitle);
        });
    });

    CollapsingHeader::new("Manage paths").show(ui, |ui| {
        if font_lib.get_paths().is_empty() {
            ui.label("No paths added.");
        } else {
            font_lib_table(ui, font_lib);
        }
        ui.add_space(8.);

        ui.horizontal(|ui| {
            if ui.button("Add files").clicked() {
                file_dialogs::add_font_lib_files(font_lib);
            }
            if ui.button("Add directories").clicked() {
                file_dialogs::add_font_lib_dirs(font_lib);
            }
            if ui.button("Clear all").clicked() {
                font_lib.clear();
            }
        });
    });

    ui.add_space(8.);
}

fn font_lib_table(ui: &mut Ui, font_lib: &mut FontLibrary) {
    let tablebuilder = TableBuilder::new(ui)
        .striped(true)
        .column(Column::exact(16.))
        .column(Column::remainder())
        .sense(Sense::click());

    let table = tablebuilder.header(0., |mut header| {
        header.col(|_| {});
        header.col(|_| {});
    });

    table.body(|body| {
        body.rows(16., font_lib.get_paths().len(), |mut row| {
            let index = row.index();
            let path = font_lib.get_paths()[index].clone();

            // Remove button
            row.col(|ui| {
                if ui
                    .add(Button::new("❎").frame(false))
                    .on_hover_text("Remove path")
                    .clicked()
                {
                    let _ = font_lib.remove_path(index);
                }
            });
            // Filename
            row.col(|ui| {
                ui.horizontal(|ui| {
                    //if let Err(e) = &status {
                    //    ui.label(RichText::new("？")).on_hover_text(e.to_string());
                    //}
                    ui.add(
                        Label::new(path.to_string_lossy())
                            .wrap_mode(TextWrapMode::Truncate)
                            .selectable(false),
                    )
                    .on_hover_text(path.to_string_lossy())
                    .on_disabled_hover_text(path.to_string_lossy());
                });
            });

            // Context menu
            row.response().context_menu(|ui| {
                //ui.add_enabled_ui(
                //    player.get_workspace().get_song_list_mode() == FileListMode::Manual,
                //    |ui| {
                //        if ui.button("Remove").clicked() {
                //            let _ = player.get_workspace_mut().remove_song(index);
                //            ui.close_menu();
                //        }
                //    },
                //);
                //actions::open_file_dir(
                //    ui,
                //    &player.get_workspace().get_songs()[index].get_path(),
                //    gui,
                //);
                if ui.button("Copy path").clicked() {
                    ui.output_mut(|o| o.copied_text = path.to_string_lossy().into());
                    ui.close_menu();
                    //gui.toast_success("Copied");
                }
            });
        });
    });
}
