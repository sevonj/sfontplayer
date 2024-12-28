pub mod actions;
pub mod conversions;
mod cooltoolbar;
pub mod custom_controls;
pub mod keyboard_shortcuts;
mod midi_inspector;
pub mod modals;
mod playback_controls;
mod playlist_fonts;
mod playlist_songs;
pub mod soundfont_library;
mod tabs;

use crate::midi_inspector::MidiInspector;
use crate::player::Player;
use crate::SfontPlayer;
use cooltoolbar::toolbar;
use eframe::egui::{vec2, CentralPanel, Context, Frame, SidePanel, TopBottomPanel, Ui};
use egui_notify::Toasts;
use keyboard_shortcuts::consume_shortcuts;
use midi_inspector::midi_inspector;
use modals::{about_modal::about_modal, settings::settings_modal, shortcuts::shortcut_modal};
use modals::{unsaved_close_dialog, unsaved_quit_dialog};
use playback_controls::playback_panel;
use playlist_fonts::soundfont_table;
use playlist_songs::playlist_song_panel;
use soundfont_library::soundfont_library;
use std::path::PathBuf;
use tabs::playlist_tabs;

const TBL_ROW_H: f32 = 16.;

/// For gui stuff that doesn't count as app logic.
#[derive(Default, serde::Deserialize, serde::Serialize)]
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
            .show_progress_bar(false)
            .closable(true);
    }
    pub fn toast_success<S: AsRef<str>>(&mut self, caption: S) {
        self.toasts
            .success(caption.as_ref())
            .show_progress_bar(false)
            .closable(true);
    }
}

#[derive(Default)]
pub struct UpdateFlags {
    pub scroll_to_song: bool,
    pub open_midi_inspector: Option<PathBuf>,
    pub close_midi_inspector: bool,
}
impl UpdateFlags {
    pub fn clear(&mut self) {
        self.scroll_to_song = false;
        self.open_midi_inspector = None;
        self.close_midi_inspector = false;
    }
}

#[allow(clippy::too_many_lines, clippy::significant_drop_tightening)]
pub fn draw_gui(ctx: &Context, app: &mut SfontPlayer) {
    let player = &mut app.player.lock();
    let gui = &mut app.gui_state;

    about_modal(ctx, gui);
    settings_modal(ctx, player, gui);
    shortcut_modal(ctx, gui);
    unsaved_close_dialog(ctx, player);
    unsaved_quit_dialog(ctx, player, gui);

    TopBottomPanel::top("top_bar")
        .resizable(false)
        .show_separator_line(false)
        .frame(
            Frame::default()
                .inner_margin(vec2(8., 2.))
                .fill(ctx.style().visuals.widgets.open.weak_bg_fill),
        )
        .show(ctx, |ui| {
            toolbar(ui, player, gui);
        });

    TopBottomPanel::bottom("playback_panel").show(ctx, |ui| {
        playback_panel(ui, player, gui);
    });

    if gui.show_font_library {
        SidePanel::right("soundfont_library")
            .exact_width(256.)
            .resizable(false)
            .frame(Frame::default())
            .show(ctx, |ui| {
                disable_if_modal(ui, gui);

                TopBottomPanel::bottom("playlist_fonts")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        ui.add_space(6.);

                        soundfont_table(ui, player, gui);
                    });

                CentralPanel::default().show_inside(ui, |ui| {
                    soundfont_library(ui, player, gui);
                });
            });
    }

    if let Some(inspector) = &mut app.midi_inspector {
        midi_inspector_panel(ctx, inspector, gui);
    } else {
        playlist_panel(ctx, player, gui);
    }
    gui.toasts.show(ctx);
    consume_shortcuts(ctx, player, gui);
    handle_dropped_files(ctx);
}

fn midi_inspector_panel(ctx: &Context, inspector: &mut MidiInspector, gui: &mut GuiState) {
    CentralPanel::default()
        .frame(Frame::central_panel(&ctx.style()).inner_margin(vec2(8., 2.)))
        .show(ctx, |ui| {
            disable_if_modal(ui, gui);

            midi_inspector(ui, inspector, gui);
        });
}

fn playlist_panel(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    TopBottomPanel::top("tab_bar")
        .resizable(false)
        .show_separator_line(false)
        .show(ctx, |ui| {
            playlist_tabs(ui, player, gui);
        });

    CentralPanel::default()
        .frame(Frame::central_panel(&ctx.style()).inner_margin(vec2(8., 2.)))
        .show(ctx, |ui| {
            disable_if_modal(ui, gui);

            playlist_song_panel(ui, player, gui);
        });
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
