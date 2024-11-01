use eframe::egui::{
    vec2, Align2, Context, Key, KeyboardShortcut, Label, Modifiers, RichText, ScrollArea,
    TextWrapMode, Ui, Window,
};
use egui_extras::{Column, TableBuilder};

use crate::{gui::file_dialogs, player::Player, GuiState};

const CTRL_SHIFT: Modifiers = Modifiers::CTRL.plus(Modifiers::SHIFT);

pub const PLAYBACK_PLAYPAUSE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Space);
pub const PLAYBACK_STARTSTOP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Space);
pub const PLAYBACK_SKIP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Period);
pub const PLAYBACK_SKIPBACK: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Comma);
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
pub const WORKSPACE_SAVE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);
pub const WORKSPACE_SAVEAS: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::S);
pub const WORKSPACE_DUPLICATE: KeyboardShortcut = KeyboardShortcut::new(CTRL_SHIFT, Key::D);

pub const GUI_SHOWFONTS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::ALT, Key::S);

/// Modal window that shows keyboard shortcuts
#[allow(clippy::too_many_lines)]
pub fn shortcut_modal(ctx: &Context, gui: &mut GuiState) {
    Window::new("Keyboard Shortcuts")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
        .open(&mut gui.show_shortcut_modal)
        .show(ctx, |ui| {
            ui.set_width(300.);
            ScrollArea::vertical().max_height(500.).show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                TableBuilder::new(ui)
                    .vscroll(false)
                    .column(Column::auto())
                    .column(Column::remainder())
                    .body(|mut body| {
                        // --- Playback

                        body.row(16., |mut row| {
                            row.col(|ui| {
                                ui.label("Playback control");
                            });
                            row.col(|_| {});
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Play / Pause");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_PLAYPAUSE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Start / Stop playback");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_STARTSTOP));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Skip");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_SKIP));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Skip back");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_SKIPBACK));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Toggle Shuffle");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_SHUFFLE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Increase volume");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_VOLUP));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Decrease volume");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_VOLDN));
                            });
                        });

                        // --- Workspaces

                        body.row(16., |mut row| {
                            row.col(|ui| {
                                ui.label("Workspaces");
                            });
                            row.col(|_| {});
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Switch to previous workspace (left)");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_SWITCHLEFT));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Switch to next workspace (right)");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_SWITCHRIGHT));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Move current workspace left");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_MOVELEFT));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Move current workspace right");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_MOVERIGHT));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Create new workspace");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_CREATE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Remove current workspace");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_REMOVE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Refresh workspace content");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_REFRESH));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Save workspace (loose file only)");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_SAVE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Save workspace to a new file.");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_SAVEAS));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Duplicate current workspace.");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_DUPLICATE));
                            });
                        });

                        // --- GUI

                        body.row(16., |mut row| {
                            row.col(|ui| {
                                ui.label("Interface");
                            });
                            row.col(|_| {});
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Toggle soundfont table");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&GUI_SHOWFONTS));
                            });
                        });
                    });
            });
        });
}

fn add_shortcut_title(ui: &mut Ui, text: &str) {
    // Slightly less intense color than Strong.
    let color = ui.visuals().widgets.open.text_color();
    ui.add(Label::new(RichText::new(text).color(color)));
}

/// Check and act on shortcuts
pub fn consume_shortcuts(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    if ctx.is_context_menu_open() {
        return;
    }

    #[allow(clippy::cognitive_complexity)]
    ctx.input_mut(|input| {
        // --- Playback

        if !gui.update_flags.disable_play_shortcut && input.consume_shortcut(&PLAYBACK_PLAYPAUSE) {
            if !player.is_paused() {
                player.pause();
            } else if !player.is_empty() {
                player.play();
            }
        }
        if input.consume_shortcut(&PLAYBACK_STARTSTOP) {
            if player.is_empty() {
                player.start();
            } else {
                player.stop();
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
        if input.consume_shortcut(&PLAYBACK_VOLUP) {
            let volume = player.get_volume();
            player.set_volume(volume + 5.);
        }
        if input.consume_shortcut(&PLAYBACK_VOLDN) {
            let volume = player.get_volume();
            player.set_volume(volume - 5.);
        }

        // --- Workspaces

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
        if input.consume_shortcut(&WORKSPACE_CREATE) {
            player.new_workspace();
            let _ = player.switch_to_workspace(player.get_workspaces().len() - 1);
        }
        if input.consume_shortcut(&WORKSPACE_REMOVE) {
            let _ = player.remove_workspace(player.get_workspace_idx());
        }
        if input.consume_shortcut(&WORKSPACE_REFRESH) {
            player.get_workspace_mut().refresh_font_list();
            player.get_workspace_mut().refresh_song_list();
        }
        if input.consume_shortcut(&WORKSPACE_SAVEAS) {
            file_dialogs::save_workspace_as(player, player.get_workspace_idx(), gui);
        }
        if input.consume_shortcut(&WORKSPACE_SAVE) {
            let _ = player.get_workspace().save_portable();
        }
        if input.consume_shortcut(&WORKSPACE_DUPLICATE) {
            let _ = player.duplicate_workspace(player.get_workspace_idx());
        }

        // --- GUI

        if input.consume_shortcut(&GUI_SHOWFONTS) {
            gui.show_soundfonts = !gui.show_soundfonts;
        }
    });
}
