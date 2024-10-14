use eframe::egui::{scroll_area::ScrollBarVisibility, Button, Frame, ScrollArea, Stroke, Ui};

use crate::player::Player;

pub fn workspace_tabs(ui: &mut Ui, player: &mut Player) {
    ScrollArea::horizontal()
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .drag_to_scroll(true)
        .enable_scrolling(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                for i in 0..player.get_workspaces().len() {
                    workspace_tab(ui, player, i);
                }
                if ui.add(Button::new("➕").frame(false)).clicked() {
                    player.new_workspace();
                    let _ = player.switch_to_workspace(player.get_workspaces().len() - 1);
                }
            });
        });
}

fn workspace_tab(ui: &mut Ui, player: &mut Player, index: usize) {
    let mut workspace_title = player.get_workspaces()[index].name.clone();
    if !player.is_paused() && player.get_playing_workspace_idx() == index {
        workspace_title = "▶ ".to_owned() + &workspace_title;
    } else if !player.is_empty() && player.get_playing_workspace_idx() == index {
        workspace_title = "⏸ ".to_owned() + &workspace_title;
    }
    if player.get_workspaces()[index].is_portable() {
        workspace_title = "🖹 ".to_owned() + &workspace_title; // File icon
    }
    let current_tab = player.get_workspace_idx() == index;

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
                let _ = player.switch_to_workspace(index);
            }
            if ui.add(Button::new("❌").frame(false)).clicked() {
                let _ = player.remove_workspace(index);
            }
        });
}
