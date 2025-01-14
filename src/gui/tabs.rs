use super::{actions, GuiState};
use crate::player::Player;
use eframe::egui::{
    scroll_area::ScrollBarVisibility, vec2, Button, Color32, Frame, Label, RichText, ScrollArea,
    Sense, Shadow, Stroke, Ui, UiBuilder,
};

pub fn playlist_tabs(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ScrollArea::horizontal()
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
        .drag_to_scroll(true)
        .enable_scrolling(true)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.style_mut().spacing.item_spacing = vec2(0.0, 0.0);
                ui.horizontal(|ui| {
                    ui.allocate_space(vec2(0.0, 26.0));
                    for i in 0..player.get_playlists().len() {
                        playlist_tab(ui, player, i, gui);
                    }
                    ui.add_space(6.0);
                    if ui
                        .add(Button::new("➕").frame(false))
                        .on_hover_text("Create new playlist")
                        .clicked()
                    {
                        let _ = player.new_playlist();
                        let _ = player.switch_to_playlist(player.get_playlists().len() - 1);
                    }
                });
                ui.add_space(1.0);
            });
        });
}

fn playlist_tab(ui: &mut Ui, player: &mut Player, index: usize, gui: &mut GuiState) {
    let mut playlist_title = player.get_playlists()[index].name.clone();
    if !player.is_paused() && player.get_playing_playlist_idx() == index {
        playlist_title = "▶ ".to_owned() + &playlist_title;
    } else if !player.is_empty() && player.get_playing_playlist_idx() == index {
        playlist_title = "⏸ ".to_owned() + &playlist_title;
    }
    if player.get_playlists()[index].is_portable() {
        playlist_title = "🖹 ".to_owned() + &playlist_title; // File icon
    }
    let current_tab = player.get_playlist_idx() == index;

    ui.style_mut().spacing.item_spacing.x = 1.0;
    let id = format!("playlist_tab_{index}");
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
                        RichText::new(playlist_title).color(style.interact(&response).text_color()),
                    )
                    .selectable(false),
                );

                ui.add_space(6.0);

                let unsaved = player.get_playlists()[index].has_unsaved_changes();
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
                    .on_hover_text("Close this playlist")
                    .clicked()
                {
                    let _ = player.remove_playlist(index);
                }
                ui.add_space(2.0);
            });

        if response.clicked() {
            let _ = player.switch_to_playlist(index);
        }

        response.context_menu(|ui| {
            actions::rename_playlist(ui, player, index);
            actions::refresh_playlist(player, index, ui);
            if let Some(filepath) = player.get_playlists()[index].get_portable_path() {
                actions::open_file_dir(ui, &filepath, gui);
            }

            ui.separator();

            actions::save_playlist(ui, player, index, gui);
            actions::save_playlist_as(ui, player, index, gui);
            actions::duplicate_playlist(ui, player, index);
            actions::close_playlist(ui, player, index);

            ui.separator();

            actions::move_playlist_left(ui, player, index);
            actions::move_playlist_right(ui, player, index);
        });
    });
}
