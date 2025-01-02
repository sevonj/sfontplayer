use build_info::{build_info, CrateInfo};
use eframe::egui::{
    include_image, vec2, Align2, CollapsingHeader, Context, Frame, Image, Label, Link, OpenUrl,
    Response, RichText, ScrollArea, Sense, Stroke, Ui, Vec2, Window,
};

use crate::GuiState;

build_info!(fn build_info);

/// Modal window that shows "About"
pub fn about_modal(ctx: &Context, gui: &mut GuiState) {
    Window::new("About")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
        .open(&mut gui.show_about_modal)
        .show(ctx, |ui| {
            ui.set_height(400.);
            ScrollArea::vertical().show(ui, |ui| {
                ui.set_width(320.);
                ui.horizontal(|ui| {
                    ui.add_space(8.);
                    ui.vertical(|ui| {
                        ui.set_width(ui.available_width() - 8.);

                        info_self(ui);

                        ui.add_space(8.);
                        ui.separator();
                        ui.add_space(8.);

                        info_dependencies(ui);
                    });
                });
            });
        });
}

/// Cool info page for this app
fn info_self(ui: &mut Ui) {
    ui.add_space(32.);

    ui.horizontal(|ui| {
        ui.add_space(62.);
        ui.add(
            Image::new(include_image!("../../assets/icon_sfontplayer.svg"))
                .fit_to_exact_size(vec2(64., 64.)),
        );
        ui.vertical(|ui| {
            ui.add_space(4.);
            ui.heading(RichText::new("SfontPlayer").strong());
            ui.label("by Sevonj & Co.");
            ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
        });
    });

    ui.add_space(32.);
    if desc_button(ui, "Contributors", "View full list of contributors.").clicked() {
        ui.ctx().open_url(OpenUrl::new_tab(
            "https://github.com/sevonj/sfontplayer/contributors",
        ));
    }
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

    license_collapse(ui, "Read license", include_str!("../../../LICENSE.txt"));
}

/// List of all dependencies
fn info_dependencies(ui: &mut Ui) {
    ui.heading(RichText::new("Dependencies").strong());
    ui.label("(direct dependencies)");
    ui.add_space(2.);
    ui.label("This app relies on a number of other open-source projects:");
    ui.add_space(8.);

    for crate_info in &build_info().crate_info.dependencies {
        dependency_item(ui, crate_info);
    }
}

/// Dependency crateinfo component
fn dependency_item(ui: &mut Ui, crate_info: &CrateInfo) {
    CollapsingHeader::new(&crate_info.name).show_unindented(ui, |ui| {
        ui.add_space(8.);

        ui.label(RichText::new("Authors:").strong());
        for author in &crate_info.authors {
            // Remove email
            ui.label(author.split('<').take(1).collect::<Vec<_>>()[0]);
        }
        if crate_info.authors.is_empty() {
            ui.label("(this crate did not specify its authors!)");
        }

        ui.add_space(2.);

        ui.label(RichText::new("License:").strong());
        ui.label(
            crate_info
                .license
                .clone()
                .unwrap_or_else(|| "(this crate did not specify its license!)".into()),
        );
    });

    ui.add_space(8.);
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
            ui.vertical(|ui| {
                link_response = ui.add(Link::new(RichText::new(title).underline().strong()));
                ui.add(Label::new(RichText::new(desc)).selectable(false));
            });
        });
    link_response
}

/// License component
fn license_collapse(ui: &mut Ui, name: &str, text: &str) {
    Frame::group(ui.style())
        .inner_margin(4.)
        .outer_margin(0.)
        .rounding(0.)
        .stroke(Stroke::NONE)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new("License information").strong());
                ui.collapsing(name, |ui| ui.label(text));
            });
        });
}
