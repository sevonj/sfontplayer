mod about;
pub mod conversions;
mod cooltoolbar;
mod fonts;
mod keyboard_shortcuts;
mod playback_controls;
mod songs;
mod workspace_select;

use crate::player::Player;
use about::about_modal;
use cooltoolbar::toolbar;
use eframe::egui::{CentralPanel, Context, TopBottomPanel, Ui};
use egui_notify::Toasts;
use fonts::{font_titlebar, soundfont_table};
use keyboard_shortcuts::{consume_shortcuts, shortcut_modal};
use playback_controls::playback_panel;
use songs::{song_table, song_titlebar};
use workspace_select::workspace_tabs;

const TBL_ROW_H: f32 = 16.;

/// For gui stuff that doesn't count as app logic.
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct GuiState {
    pub show_soundfonts: bool,
    #[serde(skip)]
    pub show_about_modal: bool,
    #[serde(skip)]
    pub show_shortcut_modal: bool,
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

    /// Keyboard shortcut won't be consumed at the end of this frame.
    /// To avoid involuntary pausing by spacebar while typing in a text box.
    pub fn disable_play_shortcut(&mut self) {
        self.update_flags.disable_play_shortcut = true;
    }
}

#[derive(Default)]
pub struct UpdateFlags {
    pub scroll_to_song: bool,
    pub disable_play_shortcut: bool,
}
impl UpdateFlags {
    pub fn clear(&mut self) {
        self.scroll_to_song = false;
        self.disable_play_shortcut = false;
    }
}

#[allow(clippy::too_many_lines)]
pub fn draw_gui(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    about_modal(ctx, gui);
    shortcut_modal(ctx, gui);

    TopBottomPanel::top("top_bar")
        .resizable(false)
        .show(ctx, |ui| {
            toolbar(ui, player, gui);
            workspace_tabs(ui, player, gui);
        });

    TopBottomPanel::bottom("playback_panel").show(ctx, |ui| {
        playback_panel(ui, player, gui);
    });

    if gui.show_soundfonts {
        TopBottomPanel::top("font_titlebar")
            .show_separator_line(false)
            .resizable(false)
            .show(ctx, |ui| {
                disable_if_modal(ui, gui);
                font_titlebar(ui, player);
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
            song_titlebar(ui, player);
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
    if gui.show_about_modal {
        ui.disable();
    }
    if gui.show_shortcut_modal {
        ui.disable();
    }
}
