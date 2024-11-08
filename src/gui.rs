pub mod actions;
pub mod conversions;
mod cooltoolbar;
pub mod keyboard_shortcuts;
pub mod modals;
mod playback_controls;
mod playlist_fonts;
mod playlist_songs;
pub mod soundfont_library;
mod workspace_select;

use crate::player::Player;
use cooltoolbar::toolbar;
use eframe::egui::{CentralPanel, Context, SidePanel, TopBottomPanel, Ui};
use egui_notify::Toasts;
use keyboard_shortcuts::consume_shortcuts;
use modals::{about_modal::about_modal, settings::settings_modal, shortcuts::shortcut_modal};
use modals::{unsaved_close_dialog, unsaved_quit_dialog};
use playback_controls::playback_panel;
use playlist_fonts::{font_titlebar, soundfont_table};
use playlist_songs::{song_table, song_titlebar};
use soundfont_library::soundfont_library;
use workspace_select::workspace_tabs;

const TBL_ROW_H: f32 = 16.;

/// For gui stuff that doesn't count as app logic.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct GuiState {
    pub show_playlist_fonts: bool,
    pub show_font_library: bool,
    #[serde(skip)]
    pub show_about_modal: bool,
    #[serde(skip)]
    pub show_settings_modal: bool,
    #[serde(skip)]
    pub show_shortcut_modal: bool,
    #[serde(skip)]
    pub show_unsaved_quit_modal: bool,
    pub show_developer_options: bool,
    /// Bypass unsaved files check on close.
    #[serde(skip)]
    pub force_quit: bool,
    /// Frame update flags. Acted on and cleared at the end of frame update.
    #[serde(skip)]
    pub update_flags: UpdateFlags,
    #[serde(skip)]
    pub toasts: Toasts,
}

impl GuiState {
    pub fn toast_error<S: AsRef<str>>(&mut self, caption: S) {
        self.toasts
            .error(caption.as_ref())
            .set_show_progress_bar(false)
            .set_closable(true);
    }
    pub fn toast_success<S: AsRef<str>>(&mut self, caption: S) {
        self.toasts
            .success(caption.as_ref())
            .set_show_progress_bar(false)
            .set_closable(true);
    }
}

#[derive(Default)]
pub struct UpdateFlags {
    pub scroll_to_song: bool,
}
impl UpdateFlags {
    pub fn clear(&mut self) {
        self.scroll_to_song = false;
    }
}

#[allow(clippy::too_many_lines)]
pub fn draw_gui(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    about_modal(ctx, gui);
    settings_modal(ctx, player, gui);
    shortcut_modal(ctx, gui);
    unsaved_close_dialog(ctx, player);
    unsaved_quit_dialog(ctx, player, gui);

    TopBottomPanel::top("top_bar")
        .resizable(false)
        .show(ctx, |ui| {
            toolbar(ui, player, gui);
            workspace_tabs(ui, player, gui);
        });

    TopBottomPanel::bottom("playback_panel").show(ctx, |ui| {
        playback_panel(ui, player, gui);
    });

    SidePanel::right("soundfont_library").show(ctx, |ui| soundfont_library(ui, player, gui));

    if gui.show_playlist_fonts {
        TopBottomPanel::top("font_titlebar")
            .show_separator_line(false)
            .resizable(false)
            .show(ctx, |ui| {
                disable_if_modal(ui, gui);
                font_titlebar(ui, player, gui);
            });
        TopBottomPanel::top("font_table")
            .resizable(true)
            .show(ctx, |ui| {
                disable_if_modal(ui, gui);
                soundfont_table(ui, player, gui);
            });
    }

    TopBottomPanel::top("song_titlebar")
        .show_separator_line(false)
        .resizable(false)
        .show(ctx, |ui| {
            disable_if_modal(ui, gui);
            song_titlebar(ui, player, gui);
        });
    CentralPanel::default().show(ctx, |ui| {
        disable_if_modal(ui, gui);
        song_table(ui, player, gui);
    });

    gui.toasts.show(ctx);
    consume_shortcuts(ctx, player, gui);
    handle_dropped_files(ctx);
}

/// TODO: Drag files into the window to add them
/// <https://github.com/sevonj/sfontplayer/issues/7>
fn handle_dropped_files(ctx: &Context) {
    ctx.input(|i| {
        for file in i.raw.dropped_files.clone() {
            println!("{file:?}");
        }
    });
}

/// This will disable the UI if a modal window is open
fn disable_if_modal(ui: &mut Ui, gui: &GuiState) {
    if gui.show_about_modal
        || gui.show_settings_modal
        || gui.show_shortcut_modal
        || gui.show_unsaved_quit_modal
    {
        ui.disable();
    }
}
