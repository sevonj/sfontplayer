use eframe::egui::{
    lerp, pos2, vec2, Align, Align2, Context, Layout, RichText, ScrollArea, Sense, Stroke, Ui,
    Widget, WidgetInfo, WidgetType, Window,
};

use crate::{player::Player, GuiState};

pub fn settings_modal(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    Window::new("Settings")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
        .open(&mut gui.show_settings_modal)
        .show(ctx, |ui| {
            ui.set_height(400.);
            ScrollArea::vertical().show(ui, |ui| {
                ui.set_width(320.);
                ui.horizontal(|ui| {
                    ui.add_space(8.);
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() - 8.);

                        ui.add_space(8.);
                        ui.heading(RichText::new("General Settings").strong());
                        ui.separator();
                        ui.add_space(8.);

                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width() - 32.);
                                ui.heading("Autosave");
                                ui.label(
                                    "Disable manual saving and use autosave for all workspaces.",
                                );
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add(toggle(&mut player.autosave));
                            });
                        });
                        ui.add_space(8.);

                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width() - 32.);
                                ui.heading("Show developer settings");
                                ui.label("These are not useful to normal users.");
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add(toggle(&mut gui.show_developer_options));
                            });
                        });
                        ui.add_space(8.);
                        if !gui.show_developer_options {
                            return;
                        }

                        ui.heading(RichText::new("Developer / Debug Settings").strong());
                        ui.separator();
                        ui.add_space(8.);

                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width() - 32.);
                                ui.heading("Block saving");
                                ui.label("Turning this on will prevent anything being saved.");
                            });
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.add(toggle(&mut player.debug_block_saving));
                            });
                        });
                        ui.add_space(8.);
                    });
                });
            });
        });
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
