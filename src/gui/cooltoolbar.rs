use eframe::egui::{Button, Ui, ViewportCommand};

use super::{
    actions,
    keyboard_shortcuts::{GUI_QUIT, GUI_SETTINGS, GUI_SHORTCUTS, GUI_SHOWFONTS},
};
use crate::{player::Player, GuiState};

/// The topmost toolbar with File Menu
pub fn toolbar(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.horizontal(|ui| {
        file_menu(ui, player, gui);

        options_menu(ui, gui);

        help_menu(ui, gui);
    });
}

fn file_menu(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.menu_button("File", |ui| {
        gui.disable_play_shortcut();

        actions::new_workspace(ui, player);
        actions::open_workspace(ui, player, gui);
        actions::save_current_workspace(ui, player, gui);
        actions::save_current_workspace_as(ui, player, gui);
        actions::duplicate_current_workspace(ui, player);
        actions::close_current_workspace(ui, player);
        actions::reopen_workspace(ui, player);

        ui.separator();

        actions::rename_current_workspace(ui, player);
        actions::refresh_current_workspace(player, ui);
        actions::current_workspace_fonts_action(ui, player);
        actions::current_workspace_songs_action(ui, player);

        ui.separator();

        actions::switch_workspace_left(ui, player);
        actions::switch_workspace_right(ui, player);
        actions::move_current_workspace_left(ui, player);
        actions::move_current_workspace_right(ui, player);

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
        gui.disable_play_shortcut();

        if ui
            .add(
                Button::new("Toggle soundfonts")
                    .shortcut_text(ui.ctx().format_shortcut(&GUI_SHOWFONTS)),
            )
            .clicked()
        {
            gui.show_soundfonts = !gui.show_soundfonts;
        }
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
        gui.disable_play_shortcut();

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
