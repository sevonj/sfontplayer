use super::{actions, GuiState};
use crate::player::Player;
use eframe::egui::{
    scroll_area::ScrollBarVisibility, vec2, Button, Color32, Frame, Label, RichText, ScrollArea,
    Sense, Shadow, Stroke, Ui, UiBuilder,
};

pub fn workspace_tabs(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ScrollArea::horizontal()
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .drag_to_scroll(true)
        .enable_scrolling(true)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.style_mut().spacing.item_spacing = vec2(0.0, 0.0);
                ui.horizontal(|ui| {
                    ui.allocate_space(vec2(0.0, 26.0));
                    for i in 0..player.get_workspaces().len() {
                        workspace_tab(ui, player, i, gui);
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
                ui.add_space(1.0);
            });
        });
}

fn workspace_tab(ui: &mut Ui, player: &mut Player, index: usize, gui: &mut GuiState) {
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
        let shadow = Shadow {
            offset: [0.0, 2.0].into(),
            color: if current_tab {
                style.visuals.selection.bg_fill
            } else {
                fill
            },
            ..Default::default()
        };
        Frame::group(&style)
            .inner_margin(4.)
            .outer_margin(0.)
            .rounding(0.)
            .stroke(Stroke::NONE)
            .fill(fill)
            .shadow(shadow)
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

                let unsaved = player.get_workspaces()[index].has_unsaved_changes();
                if !(response.hovered() || current_tab || unsaved) {
                    ui.style_mut().visuals.widgets.inactive.fg_stroke.color = Color32::TRANSPARENT;
                }
                let close_symbol = if unsaved && !player.autosave {
                    "⊗"
                } else {
                    "❌"
                };
                if ui
                    .add(Button::new(RichText::new(close_symbol).size(14.0)).frame(false))
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

        response.context_menu(|ui| {
            actions::rename_workspace(ui, player, index);
            actions::refresh_workspace(player, index, ui);
            if let Some(filepath) = player.get_workspaces()[index].get_portable_path() {
                actions::open_file_dir(ui, &filepath, gui);
            }

            ui.separator();

            actions::save_workspace(ui, player, index, gui);
            actions::save_workspace_as(ui, player, index, gui);
            actions::duplicate_workspace(ui, player, index);
            actions::close_workspace(ui, player, index);

            ui.separator();

            actions::move_workspace_left(ui, player, index);
            actions::move_workspace_right(ui, player, index);
        });
    });
}
