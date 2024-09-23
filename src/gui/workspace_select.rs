use eframe::egui::{self, Button, Stroke};

use crate::SfontPlayer;

pub(crate) fn workspace_tabs(ui: &mut egui::Ui, app: &mut SfontPlayer) {
    ui.horizontal(|ui| {
        for i in 0..app.workspaces.len() {
            workspace_tab(ui, app, i);
        }
        if ui.add(Button::new("➕").frame(false)).clicked() {
            app.new_workspace();
            app.workspace_idx = app.workspaces.len() - 1;
        }
    });
}

fn workspace_tab(ui: &mut egui::Ui, app: &mut SfontPlayer, index: usize) {
    let mut workspace_title = app.workspaces[index].name.clone();
    if app.is_playing() && app.playing_workspace_idx == index {
        workspace_title = "▶ ".to_owned() + &workspace_title;
    }
    let current_tab = app.workspace_idx == index;

    let style = (*ui.ctx().style()).clone();
    let fill = if current_tab {
        style.visuals.code_bg_color
    } else {
        style.visuals.faint_bg_color
    };

    egui::Frame::group(&style)
        .inner_margin(4.)
        .outer_margin(0.)
        .rounding(0.)
        .stroke(Stroke::NONE)
        .fill(fill)
        .show(ui, |ui| {
            if ui.add(Button::new(workspace_title).frame(false)).clicked() {
                app.workspace_idx = index;
            }
            if ui.add(Button::new("❌").frame(false)).clicked() {
                app.remove_workspace(index);
            }
        });
}
