use eframe::egui::{Context, Key, KeyboardShortcut, Modifiers, ViewportCommand};

use super::{modals::file_dialogs, GuiState};
use crate::player::Player;

const CTRL_SHIFT: Modifiers = Modifiers::CTRL.plus(Modifiers::SHIFT);
const CTRL_ALT: Modifiers = Modifiers::CTRL.plus(Modifiers::ALT);

pub const PLAYBACK_PLAYPAUSE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Space);
pub const PLAYBACK_STARTSTOP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Space);
pub const PLAYBACK_SKIP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Period);
pub const PLAYBACK_SKIPBACK: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Comma);
pub const PLAYBACK_REPEAT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::R);
pub const PLAYBACK_SHUFFLE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::S);
pub const PLAYBACK_VOLUP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::ArrowUp);
pub const PLAYBACK_VOLDN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::ArrowDown);

pub const PLAYLIST_SWITCHLEFT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::ALT, Key::ArrowLeft);
pub const PLAYLIST_SWITCHRIGHT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::ALT, Key::ArrowRight);
pub const PLAYLIST_MOVELEFT: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::ArrowLeft);
pub const PLAYLIST_MOVERIGHT: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::ArrowRight);
pub const PLAYLIST_REMOVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::W);
pub const PLAYLIST_CREATE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::N);
pub const PLAYLIST: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F5);
pub const PLAYLIST_OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
pub const PLAYLIST_SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);
pub const PLAYLIST_SAVEAS: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::S);
pub const PLAYLIST_SAVEALL: KeyboardShortcut = KeyboardShortcut::new(CTRL_ALT, Key::S);
pub const PLAYLIST_DUPLICATE: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::D);
pub const PLAYLIST_REOPEN: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::T);

pub const GUI_QUIT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Q);
pub const GUI_SHOWFONTS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::ALT, Key::S);
pub const GUI_SETTINGS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Comma);
pub const GUI_SHORTCUTS: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::Questionmark);

/// Check and act on shortcuts
pub fn consume_shortcuts(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    if ctx.wants_keyboard_input() {
        return;
    }
    consume_2_modifiers(ctx, player, gui);
    consume_1_modifier(ctx, player, gui);
    consume_no_modifiers(ctx, player, gui);
}

fn consume_2_modifiers(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    ctx.input_mut(|input| {
        if input.consume_shortcut(&PLAYLIST_MOVELEFT) {
            if let Err(e) = player.move_playlist_left() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&PLAYLIST_MOVERIGHT) {
            if let Err(e) = player.move_playlist_right() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&PLAYLIST_SAVEAS) {
            file_dialogs::save_playlist_as(player, player.get_playlist_idx(), gui);
        }
        if input.consume_shortcut(&PLAYLIST_SAVEALL) {
            if let Err(e) = player.save_all_portable_playlists() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&PLAYLIST_DUPLICATE) {
            let _ = player.duplicate_playlist(player.get_playlist_idx());
        }
        if input.consume_shortcut(&PLAYLIST_REOPEN) {
            let _ = player.reopen_removed_playlist();
        }
    });
}

fn consume_1_modifier(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    let mut quit = false;

    ctx.input_mut(|input| {
        if input.consume_shortcut(&PLAYBACK_STARTSTOP) {
            if player.is_empty() {
                player.start();
            } else {
                player.stop();
            }
        }
        if input.consume_shortcut(&PLAYBACK_VOLUP) {
            let volume = player.get_volume();
            player.set_volume(volume + 5.);
        }
        if input.consume_shortcut(&PLAYBACK_VOLDN) {
            let volume = player.get_volume();
            player.set_volume(volume - 5.);
        }

        if input.consume_shortcut(&PLAYLIST_SWITCHLEFT) {
            if let Err(e) = player.switch_playlist_left() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&PLAYLIST_SWITCHRIGHT) {
            if let Err(e) = player.switch_playlist_right() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&PLAYLIST_CREATE) {
            let _ = player.new_playlist();
            let _ = player.switch_to_playlist(player.get_playlists().len() - 1);
        }
        if input.consume_shortcut(&PLAYLIST_REMOVE) {
            let _ = player.remove_playlist(player.get_playlist_idx());
        }
        if input.consume_shortcut(&PLAYLIST_OPEN) {
            file_dialogs::open_playlist(player, gui);
        }
        if input.consume_shortcut(&PLAYLIST_SAVE) {
            if player.autosave {
                return;
            }
            if let Err(e) = player.save_portable_playlist(player.get_playlist_idx()) {
                gui.toast_error(e.to_string());
            }
        }

        if input.consume_shortcut(&GUI_QUIT) {
            quit = true;
        }
        if input.consume_shortcut(&GUI_SHOWFONTS) {
            gui.show_font_library = !gui.show_font_library;
        }
        if input.consume_shortcut(&GUI_SETTINGS) {
            gui.show_settings_modal = true;
        }
        if input.consume_shortcut(&GUI_SHORTCUTS) {
            gui.show_shortcut_modal = true;
        }
    });

    // This is down here because sending the command from the input closure hangs the program.
    if quit {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}

fn consume_no_modifiers(ctx: &Context, player: &mut Player, _gui: &GuiState) {
    ctx.input_mut(|input| {
        if input.consume_shortcut(&PLAYBACK_PLAYPAUSE) {
            if !player.is_paused() {
                player.pause();
            } else if !player.is_empty() {
                player.play();
            }
        }
        if input.consume_shortcut(&PLAYBACK_SKIP) {
            let _ = player.skip();
        }
        if input.consume_shortcut(&PLAYBACK_SKIPBACK) {
            let _ = player.skip_back();
        }
        if input.consume_shortcut(&PLAYBACK_SHUFFLE) {
            player.toggle_shuffle();
        }
        if input.consume_shortcut(&PLAYBACK_REPEAT) {
            player.cycle_repeat();
        }
        if input.consume_shortcut(&PLAYLIST) {
            player.get_playlist_mut().recrawl_fonts();
            player.get_playlist_mut().refresh_song_list();
        }
    });
}
