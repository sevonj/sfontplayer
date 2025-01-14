//! Common actions for context menus and such
//!

use std::path::{Path, PathBuf};

use eframe::egui::{Button, ComboBox, Label, TextEdit, Ui, Widget};
use rfd::FileDialog;

use super::{
    custom_controls::circle_button,
    keyboard_shortcuts::{
        PLAYLIST, PLAYLIST_CREATE, PLAYLIST_DUPLICATE, PLAYLIST_MOVELEFT, PLAYLIST_MOVERIGHT,
        PLAYLIST_OPEN, PLAYLIST_REMOVE, PLAYLIST_REOPEN, PLAYLIST_SAVE, PLAYLIST_SAVEAS,
        PLAYLIST_SWITCHLEFT, PLAYLIST_SWITCHRIGHT,
    },
    modals::file_dialogs,
    GuiState,
};
use crate::player::{playlist::enums::FileListMode, Player};

// --- Common File Actions --- //

pub fn open_file_dir(ui: &mut Ui, filepath: &Path, gui: &mut GuiState) {
    if ui.button("Go to directory").clicked() {
        let Some(dir) = filepath.parent() else {
            gui.toast_error("Failed to get file parent.");
            return;
        };
        if let Err(e) = open::that(dir) {
            gui.toast_error(e.to_string());
        }
        ui.close_menu();
    }
}

pub fn pick_dir_button(dir: Option<&PathBuf>, ui: &mut Ui) -> Option<PathBuf> {
    let folder_text = if dir.is_some() { "🗁" } else { "🗀" };
    if circle_button(folder_text, ui)
        .on_hover_text("Select directory")
        .clicked()
    {
        return FileDialog::new().pick_folder();
    }
    None
}

pub fn pick_soundfonts_button(ui: &mut Ui) -> Option<Vec<PathBuf>> {
    if circle_button("➕", ui).on_hover_text("Add").clicked() {
        return FileDialog::new()
            .add_filter("Soundfonts", &["sf2"])
            .pick_files();
    }
    None
}

pub fn pick_midifiles_button(ui: &mut Ui) -> Option<Vec<PathBuf>> {
    if circle_button("➕", ui).on_hover_text("Add").clicked() {
        return FileDialog::new()
            .add_filter("Midi files", &["mid"])
            .pick_files();
    }
    None
}

// --- Playlist File Actions --- //

pub fn new_playlist(ui: &mut Ui, player: &mut Player) {
    if ui
        .add(Button::new("New").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_CREATE)))
        .on_hover_text("Create a new playlist")
        .clicked()
    {
        let _ = player.new_playlist();
        ui.close_menu();
    }
}

pub fn open_playlist(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    if ui
        .add(Button::new("Open").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_OPEN)))
        .on_hover_text("Load a playlist file")
        .clicked()
    {
        file_dialogs::open_playlist(player, gui);
        ui.close_menu();
    }
}

pub fn save_playlist(ui: &mut Ui, player: &mut Player, index: usize, gui: &mut GuiState) {
    ui.add_enabled_ui(
        player.get_playlists()[index].is_portable() && !player.autosave,
        |ui| {
            let hover_text = get_save_playlist_tooltip(player, index);
            if ui
                .add(Button::new("Save"))
                .on_hover_text(hover_text)
                .on_disabled_hover_text(hover_text)
                .clicked()
            {
                if let Err(e) = player.save_portable_playlist(index) {
                    gui.toast_error(e.to_string());
                }
                ui.close_menu();
            }
        },
    );
}

pub fn save_current_playlist(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.add_enabled_ui(
        player.get_playlist().is_portable() && !player.autosave,
        |ui| {
            let hover_text = get_save_playlist_tooltip(player, player.get_playlist_idx());
            if ui
                .add(Button::new("Save").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_SAVE)))
                .on_hover_text(hover_text)
                .on_disabled_hover_text(hover_text)
                .clicked()
            {
                if let Err(e) = player.save_portable_playlist(player.get_playlist_idx()) {
                    gui.toast_error(e.to_string());
                }
                ui.close_menu();
            }
        },
    );
}

fn get_save_playlist_tooltip(player: &Player, index: usize) -> &str {
    if !player.get_playlists()[index].is_portable() {
        "Playlists in app memory are saved automatically."
    } else if player.autosave {
        "Autosave is enabled."
    } else {
        "Save unsaved changes."
    }
}

pub fn save_playlist_as(ui: &mut Ui, player: &mut Player, index: usize, gui: &mut GuiState) {
    if ui
        .add(Button::new("Save as"))
        .on_hover_text("Save a copy to a new file")
        .clicked()
    {
        file_dialogs::save_playlist_as(player, index, gui);
        ui.close_menu();
    }
}

pub fn save_current_playlist_as(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    if ui
        .add(Button::new("Save as").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_SAVEAS)))
        .on_hover_text("Save a copy to a new file")
        .clicked()
    {
        file_dialogs::save_playlist_as(player, player.get_playlist_idx(), gui);
        ui.close_menu();
    }
}

pub fn duplicate_playlist(ui: &mut Ui, player: &mut Player, index: usize) {
    if ui
        .add(Button::new("Duplicate"))
        .on_hover_text("Create a copy of this playlist")
        .clicked()
    {
        let _ = player.duplicate_playlist(index);
        ui.close_menu();
    }
}

pub fn duplicate_current_playlist(ui: &mut Ui, player: &mut Player) {
    if ui
        .add(Button::new("Duplicate").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_DUPLICATE)))
        .on_hover_text("Create a copy of current playlist")
        .clicked()
    {
        let _ = player.duplicate_playlist(player.get_playlist_idx());
        ui.close_menu();
    }
}

pub fn close_playlist(ui: &mut Ui, player: &mut Player, index: usize) {
    if ui
        .add(Button::new("Close"))
        .on_hover_text("Close playlist")
        .clicked()
    {
        let _ = player.remove_playlist(index);
        ui.close_menu();
    }
}

pub fn close_current_playlist(ui: &mut Ui, player: &mut Player) {
    if ui
        .add(Button::new("Close").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_REMOVE)))
        .on_hover_text("Close playlist")
        .clicked()
    {
        let _ = player.remove_playlist(player.get_playlist_idx());
        ui.close_menu();
    }
}

pub fn reopen_playlist(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.has_removed_playlist(),
            Button::new("Reopen closed").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_REOPEN)),
        )
        .on_hover_text("Reopen last closed playlist")
        .on_disabled_hover_text("Reopen last closed playlist")
        .clicked()
    {
        let _ = player.reopen_removed_playlist();
        ui.close_menu();
    }
}

// --- Playlist Content Actions --- //

pub fn rename_playlist(ui: &mut Ui, player: &mut Player, index: usize) {
    ui.add(Label::new("Name:").selectable(false));
    ui.add(TextEdit::singleline(&mut player.get_playlists_mut()[index].name).desired_width(128.));
}

pub fn rename_current_playlist(ui: &mut Ui, player: &mut Player) {
    ui.menu_button("Rename playlist", |ui| {
        if ui
            .add(TextEdit::singleline(&mut player.get_playlist_mut().name).desired_width(128.))
            .lost_focus()
        {
            ui.close_menu();
        }
        if ui.button("OK").clicked() {
            ui.close_menu();
        }
    });
}

pub fn refresh_playlist(player: &mut Player, index: usize, ui: &mut Ui) {
    let playlist = &mut player.get_playlists_mut()[index];
    let can_refresh = playlist.get_font_list_mode() != FileListMode::Manual
        || playlist.get_song_list_mode() != FileListMode::Manual;
    ui.add_enabled_ui(can_refresh, |ui| {
        if ui
            .button("Refresh content")
            .on_hover_text("Refresh directory contents")
            .on_disabled_hover_text("This playlist uses manual listing.")
            .clicked()
        {
            playlist.refresh_font_list();
            playlist.refresh_song_list();
            ui.close_menu();
        }
    });
}

pub fn refresh_current_playlist(player: &mut Player, ui: &mut Ui) {
    let can_refresh = player.get_playlist().get_font_list_mode() != FileListMode::Manual
        || player.get_playlist().get_song_list_mode() != FileListMode::Manual;
    if ui
        .add_enabled(
            can_refresh,
            Button::new("Refresh content").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST)),
        )
        .on_hover_text("Refresh directory contents")
        .on_disabled_hover_text("This playlist uses manual listing.")
        .clicked()
    {
        player.get_playlist_mut().refresh_font_list();
        player.get_playlist_mut().refresh_song_list();
        ui.close_menu();
    }
}

pub fn current_playlist_fonts_action(ui: &mut Ui, player: &mut Player) {
    ui.menu_button("Soundfonts", |ui| {
        let mut list_mode = player.get_playlist().get_font_list_mode();
        ui.add_enabled_ui(list_mode == FileListMode::Manual, |ui| {
            if ui.button("Add soundfonts").clicked() {
                if let Some(paths) = FileDialog::new()
                    .add_filter("Soundfonts", &["sf2"])
                    .pick_files()
                {
                    for path in paths {
                        let _ = player.get_playlist_mut().add_font(path);
                    }
                    ui.close_menu();
                }
            }
            if ui.button("Clear soundfonts").clicked() {
                player.get_playlist_mut().clear_fonts();
                ui.close_menu();
            }
        });
        ui.label("Content mode");
        let response1 = ui.radio_value(&mut list_mode, FileListMode::Manual, "Manual");
        let response2 = ui.radio_value(&mut list_mode, FileListMode::Directory, "Directory");
        let response3 = ui.radio_value(
            &mut list_mode,
            FileListMode::Subdirectories,
            "Subdirectories",
        );
        if response1.clicked() || response2.clicked() || response3.clicked() {
            player.get_playlist_mut().set_font_list_mode(list_mode);
        }
    });
}
pub fn current_playlist_songs_action(ui: &mut Ui, player: &mut Player) {
    ui.menu_button("Songs", |ui| {
        let mut list_mode = player.get_playlist().get_song_list_mode();
        ui.add_enabled_ui(list_mode == FileListMode::Manual, |ui| {
            if ui.button("Add songs").clicked() {
                if let Some(paths) = FileDialog::new()
                    .add_filter("Midi files", &["mid"])
                    .pick_files()
                {
                    for path in paths {
                        let _ = player.get_playlist_mut().add_song(path);
                    }
                    ui.close_menu();
                }
            }
            if ui.button("Clear songs").clicked() {
                player.get_playlist_mut().clear_songs();
                ui.close_menu();
            }
        });
        ui.label("Content mode");
        let response1 = ui.radio_value(&mut list_mode, FileListMode::Manual, "Manual");
        let response2 = ui.radio_value(&mut list_mode, FileListMode::Directory, "Directory");
        let response3 = ui.radio_value(
            &mut list_mode,
            FileListMode::Subdirectories,
            "Subdirectories",
        );
        if response1.clicked() || response2.clicked() || response3.clicked() {
            player.get_playlist_mut().set_song_list_mode(list_mode);
        }
    });
}

pub fn content_mode_selector(mode: &mut FileListMode) -> impl Widget + '_ {
    move |ui: &mut Ui| {
        ComboBox::from_id_salt("mode_select")
            .selected_text(format!("{mode}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(mode, FileListMode::Manual, FileListMode::Manual.to_string());
                ui.selectable_value(
                    mode,
                    FileListMode::Directory,
                    FileListMode::Directory.to_string(),
                );
                ui.selectable_value(
                    mode,
                    FileListMode::Subdirectories,
                    FileListMode::Subdirectories.to_string(),
                );
            })
            .response
    }
}

// --- Playlist Navigation --- //

pub fn switch_playlist_left(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.get_playlist_idx() > 0,
            Button::new("Switch one left")
                .shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_SWITCHLEFT)),
        )
        .on_hover_text("Switch to previous playlist")
        .on_disabled_hover_text("Switch to previous playlist")
        .clicked()
    {
        let _ = player.switch_playlist_left();
        ui.close_menu();
    }
}

pub fn switch_playlist_right(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.get_playlist_idx() < player.get_playlists().len() - 1,
            Button::new("Switch one right")
                .shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_SWITCHRIGHT)),
        )
        .on_hover_text("Switch to next playlist")
        .on_disabled_hover_text("Switch to next playlist")
        .clicked()
    {
        let _ = player.switch_playlist_right();
        ui.close_menu();
    }
}

pub fn move_playlist_left(ui: &mut Ui, player: &mut Player, index: usize) {
    if ui
        .add_enabled(index > 0, Button::new("Move left"))
        .on_hover_text("Move playlist left")
        .on_disabled_hover_text("Move playlist left")
        .clicked()
    {
        let _ = player.move_playlist(index, index - 1);
        ui.close_menu();
    }
}

pub fn move_current_playlist_left(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.get_playlist_idx() > 0,
            Button::new("Move left").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_MOVELEFT)),
        )
        .on_hover_text("Move playlist left")
        .on_disabled_hover_text("Move playlist left")
        .clicked()
    {
        let _ = player.move_playlist_left();
        ui.close_menu();
    }
}

pub fn move_playlist_right(ui: &mut Ui, player: &mut Player, index: usize) {
    if ui
        .add_enabled(
            index < player.get_playlists().len() - 1,
            Button::new("Move right"),
        )
        .on_hover_text("Move playlist right")
        .on_disabled_hover_text("Move playlist right")
        .clicked()
    {
        let _ = player.move_playlist(index, index + 1);
        ui.close_menu();
    }
}

pub fn move_current_playlist_right(ui: &mut Ui, player: &mut Player) {
    if ui
        .add_enabled(
            player.get_playlist_idx() < player.get_playlists().len() - 1,
            Button::new("Move right").shortcut_text(ui.ctx().format_shortcut(&PLAYLIST_MOVERIGHT)),
        )
        .on_hover_text("Move playlist right")
        .on_disabled_hover_text("Move playlist right")
        .clicked()
    {
        let _ = player.move_playlist_right();
        ui.close_menu();
    }
}
