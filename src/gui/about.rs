use eframe::egui::{
    vec2, Align2, Context, Frame, Label, Link, OpenUrl, Response, RichText, ScrollArea, Sense,
    Stroke, Ui, Vec2, Window,
};

use crate::SfontPlayer;
pub(crate) fn about_window(ctx: &Context, app: &mut SfontPlayer) {
    Window::new("About")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
        .open(&mut app.show_about_window)
        .show(ctx, |ui| {
            ui.set_width(300.);
            ScrollArea::vertical().max_height(500.).show(ui, |ui| {
                info_self(ui);

                ui.add_space(8.);
                ui.separator();
                ui.add_space(8.);

                info_dependencies(ui);
            });
        });
}

/// Cool info page for this app
fn info_self(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.add_space(60.);
        ui.label(RichText::new("🎵").size(60.0));
        ui.vertical(|ui| {
            ui.add_space(12.);
            ui.heading("SfontPlayer");
            ui.label("by Sevonj");
            ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
        });
    });

    ui.add_space(16.);

    // Repo
    if desc_button(
        ui,
        "View project repository",
        "Bug reports, feature requests, source code",
    )
    .clicked()
    {
        ui.ctx()
            .open_url(OpenUrl::new_tab(env!("CARGO_PKG_REPOSITORY")));
    }

    // License
    if desc_button(ui, "License information", "Not yet licensed.").clicked() {
        ui.ctx()
            .open_url(OpenUrl::new_tab(env!("CARGO_PKG_REPOSITORY")));
    }
}

/// List of all dependencies
fn info_dependencies(ui: &mut Ui) {
    ui.label("This program relies on a number of other open-source projects:");
    desc_button(ui, "License information", "Under construction");
    desc_button(ui, "License information", "Under construction");
    desc_button(ui, "License information", "Under construction");
    desc_button(ui, "License information", "Under construction");
}

/// Custom "button" with a description
fn desc_button(ui: &mut Ui, title: &str, desc: &str) -> Response {
    // Allocate dummy response.
    let mut link_response = ui.allocate_response(Vec2::ZERO, Sense::click());

    Frame::group(ui.style())
        .inner_margin(4.)
        .outer_margin(0.)
        .rounding(0.)
        .stroke(Stroke::NONE)
        .show(ui, |ui| {
            ui.style_mut().visuals.hyperlink_color = ui.style().visuals.strong_text_color();
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(8.);
                ui.vertical(|ui| {
                    link_response = ui.add(Link::new(RichText::new(title).strong()));
                    ui.add(Label::new(RichText::new(desc)).selectable(false));
                });
            });
        });
    link_response
}