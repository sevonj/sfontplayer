use eframe::egui::{scroll_area::ScrollBarVisibility, Button, Frame, ScrollArea, Stroke, Ui};

use crate::SfontPlayer;

pub fn workspace_tabs(ui: &mut Ui, app: &mut SfontPlayer) {
    ScrollArea::horizontal()
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .drag_to_scroll(true)
        .enable_scrolling(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for i in 0..app.get_workspaces().len() {
                    workspace_tab(ui, app, i);
                }
                if ui.add(Button::new("➕").frame(false)).clicked() {
                    app.new_workspace();
                    app.switch_workspace(app.get_workspaces().len() - 1);
                }
            });
        });
}

fn workspace_tab(ui: &mut Ui, app: &mut SfontPlayer, index: usize) {
    let mut workspace_title = app.get_workspaces()[index].name.clone();
    if !app.is_paused() && app.playing_workspace_idx == index {
        workspace_title = "▶ ".to_owned() + &workspace_title;
    } else if !app.is_empty() && app.playing_workspace_idx == index {
        workspace_title = "⏸ ".to_owned() + &workspace_title;
    }
    let current_tab = app.workspace_idx == index;

    let style = (*ui.ctx().style()).clone();
    let fill = if current_tab {
        style.visuals.code_bg_color
    } else {
        style.visuals.faint_bg_color
    };

    Frame::group(&style)
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
