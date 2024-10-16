use eframe::egui::{
    scroll_area::ScrollBarVisibility, vec2, Button, Color32, Frame, Label, RichText, ScrollArea,
    Sense, Stroke, Ui, UiBuilder,
};

use crate::player::Player;

pub fn workspace_tabs(ui: &mut Ui, player: &mut Player) {
    ScrollArea::horizontal()
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .drag_to_scroll(true)
        .enable_scrolling(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing.x = 0.0;
                ui.allocate_space(vec2(0.0, 26.0));
                for i in 0..player.get_workspaces().len() {
                    workspace_tab(ui, player, i);
                }
                ui.add_space(6.0);
                if ui
                    .add(Button::new("➕").frame(false))
                    .on_hover_text("Create new workspace")
                    .clicked()
                {
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

    ui.style_mut().spacing.item_spacing.x = 1.0;
    let id = format!("workspace_tab_{index}");
    let sense = Sense::union(Sense::click(), Sense::hover());

    ui.scope_builder(UiBuilder::new().id_salt(id).sense(sense), |ui| {
        let style = (*ui.ctx().style()).clone();
        let response = ui.response();
        let fill = if current_tab {
            style.interact(&response).bg_fill
        //} else if response.hovered() {
        //    style.interact(&response).weak_bg_fill
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
                ui.style_mut().spacing.item_spacing.x = 0.0;
                ui.add_space(4.0);
                ui.add(
                    Label::new(
                        RichText::new(workspace_title)
                            .color(style.interact(&response).text_color()),
                    )
                    .selectable(false),
                );

                ui.add_space(6.0);

                if !(response.hovered() || current_tab) {
                    ui.style_mut().visuals.widgets.inactive.fg_stroke.color = Color32::TRANSPARENT;
                }
                if ui
                    .add(Button::new(RichText::new("❌").size(14.0)).frame(false))
                    .on_hover_text("Close this workspace")
                    .clicked()
                {
                    let _ = player.remove_workspace(index);
                }
                ui.add_space(2.0);
            });

        if response.clicked() {
            let _ = player.switch_to_workspace(index);
        }
    });
}
