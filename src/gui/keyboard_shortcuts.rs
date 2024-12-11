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

pub const WORKSPACE_SWITCHLEFT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::ALT, Key::ArrowLeft);
pub const WORKSPACE_SWITCHRIGHT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::ALT, Key::ArrowRight);
pub const WORKSPACE_MOVELEFT: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::ArrowLeft);
pub const WORKSPACE_MOVERIGHT: KeyboardShortcut =
    KeyboardShortcut::new(CTRL_SHIFT, Key::ArrowRight);
pub const WORKSPACE_REMOVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::W);
pub const WORKSPACE_CREATE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::N);
pub const WORKSPACE_REFRESH: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::F5);
pub const WORKSPACE_OPEN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
pub const WORKSPACE_SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);
pub const WORKSPACE_SAVEAS: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::S);
pub const WORKSPACE_SAVEALL: KeyboardShortcut = KeyboardShortcut::new(CTRL_ALT, Key::S);
pub const WORKSPACE_DUPLICATE: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::D);
pub const WORKSPACE_REOPEN: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::T);

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
        if input.consume_shortcut(&WORKSPACE_MOVELEFT) {
            if let Err(e) = player.move_workspace_left() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&WORKSPACE_MOVERIGHT) {
            if let Err(e) = player.move_workspace_right() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&WORKSPACE_SAVEAS) {
            file_dialogs::save_workspace_as(player, player.get_workspace_idx(), gui);
        }
        if input.consume_shortcut(&WORKSPACE_SAVEALL) {
            if let Err(e) = player.save_all_portable_workspaces() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&WORKSPACE_DUPLICATE) {
            let _ = player.duplicate_workspace(player.get_workspace_idx());
        }
        if input.consume_shortcut(&WORKSPACE_REOPEN) {
            player.reopen_removed_workspace();
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

        if input.consume_shortcut(&WORKSPACE_SWITCHLEFT) {
            if let Err(e) = player.switch_workspace_left() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&WORKSPACE_SWITCHRIGHT) {
            if let Err(e) = player.switch_workspace_right() {
                gui.toast_error(e.to_string());
            }
        }
        if input.consume_shortcut(&WORKSPACE_CREATE) {
            player.new_workspace();
            let _ = player.switch_to_workspace(player.get_workspaces().len() - 1);
        }
        if input.consume_shortcut(&WORKSPACE_REMOVE) {
            let _ = player.remove_workspace(player.get_workspace_idx());
        }
        if input.consume_shortcut(&WORKSPACE_OPEN) {
            file_dialogs::open_workspace(player, gui);
        }
        if input.consume_shortcut(&WORKSPACE_SAVE) {
            if player.autosave {
                return;
            }
            if let Err(e) = player.save_portable_workspace(player.get_workspace_idx()) {
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
            player.skip();
        }
        if input.consume_shortcut(&PLAYBACK_SKIPBACK) {
            player.skip_back();
        }
        if input.consume_shortcut(&PLAYBACK_SHUFFLE) {
            player.toggle_shuffle();
        }
        if input.consume_shortcut(&PLAYBACK_REPEAT) {
            player.cycle_repeat();
        }
        if input.consume_shortcut(&WORKSPACE_REFRESH) {
            player.get_workspace_mut().refresh_font_list();
            player.get_workspace_mut().refresh_song_list();
        }
    });
}
