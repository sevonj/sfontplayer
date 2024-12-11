use eframe::egui::{Align, Button, Layout, RichText, Ui, ViewportCommand};

use super::{
    actions,
    keyboard_shortcuts::{GUI_QUIT, GUI_SETTINGS, GUI_SHORTCUTS},
};
use crate::{player::Player, GuiState};

/// The topmost toolbar with File Menu
pub fn toolbar(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.horizontal(|ui| {
        file_menu(ui, player, gui);

        options_menu(ui, gui);

        help_menu(ui, gui);

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            sidebar_toggle(ui, gui);
        });
    });
}

fn file_menu(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.menu_button("File", |ui| {
        actions::new_playlist(ui, player);
        actions::open_playlist(ui, player, gui);
        actions::save_current_playlist(ui, player, gui);
        actions::save_current_playlist_as(ui, player, gui);
        actions::duplicate_current_playlist(ui, player);
        actions::close_current_playlist(ui, player);
        actions::reopen_playlist(ui, player);

        ui.separator();

        actions::rename_current_playlist(ui, player);
        actions::refresh_current_playlist(player, ui);
        actions::current_playlist_fonts_action(ui, player);
        actions::current_playlist_songs_action(ui, player);

        ui.separator();

        actions::switch_playlist_left(ui, player);
        actions::switch_playlist_right(ui, player);
        actions::move_current_playlist_left(ui, player);
        actions::move_current_playlist_right(ui, player);

        ui.separator();

        if ui
            .add(Button::new("Quit").shortcut_text(ui.ctx().format_shortcut(&GUI_QUIT)))
            .clicked()
        {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
    });
}

fn options_menu(ui: &mut Ui, gui: &mut GuiState) {
    ui.menu_button("Options", |ui| {
        if ui
            .add(Button::new("Settings").shortcut_text(ui.ctx().format_shortcut(&GUI_SETTINGS)))
            .clicked()
        {
            gui.show_settings_modal = true;
            ui.close_menu();
        }
    });
}

fn help_menu(ui: &mut Ui, gui: &mut GuiState) {
    ui.menu_button("Help", |ui| {
        if ui.button("About").clicked() {
            gui.show_about_modal = true;
            ui.close_menu();
        }
        if ui
            .add(
                Button::new("Keyboard shortcuts")
                    .shortcut_text(ui.ctx().format_shortcut(&GUI_SHORTCUTS)),
            )
            .clicked()
        {
            gui.show_shortcut_modal = true;
            ui.close_menu();
        }
    });
}

fn sidebar_toggle(ui: &mut Ui, gui: &mut GuiState) {
    if ui
        .add(
            Button::new(
                RichText::new(if gui.show_font_library {
                    "⏵"
                } else {
                    "⏴ Soundfonts"
                })
                .size(16.),
            )
            .frame(false),
        )
        .clicked()
    {
        gui.show_font_library = !gui.show_font_library;
    };
}
