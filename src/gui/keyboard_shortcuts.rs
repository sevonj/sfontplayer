use eframe::egui::{
    vec2, Align2, Context, Key, KeyboardShortcut, Label, Modifiers, RichText, ScrollArea,
    TextWrapMode, Ui, Window,
};
use egui_extras::{Column, TableBuilder};

use crate::SfontPlayer;

const CTRL_SHIFT: Modifiers = Modifiers::CTRL.plus(Modifiers::SHIFT);

pub const PLAYBACK_PLAYPAUSE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Space);
pub const PLAYBACK_STARTSTOP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Space);
pub const PLAYBACK_SKIP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Period);
pub const PLAYBACK_SKIPBACK: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Comma);
pub const PLAYBACK_SHUFFLE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);
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

pub const GUI_SHOWFONTS: KeyboardShortcut = KeyboardShortcut::new(Modifiers::ALT, Key::S);

/// Modal window that shows keyboard shortcuts
#[allow(clippy::too_many_lines)]
pub fn shortcut_modal(ctx: &Context, app: &mut SfontPlayer) {
    Window::new("Keyboard Shortcuts")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
        .open(&mut app.show_shortcut_modal)
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
pub fn consume_shortcuts(ctx: &Context, app: &mut SfontPlayer) {
    if ctx.is_context_menu_open() {
        return;
    }

    ctx.input_mut(|input| {
        // --- Playback

        if input.consume_shortcut(&PLAYBACK_PLAYPAUSE) {
            if !app.is_paused() {
                app.pause();
            } else if !app.is_empty() {
                app.play();
            }
        }
        if input.consume_shortcut(&PLAYBACK_STARTSTOP) {
            if app.is_empty() {
                app.start();
            } else {
                app.stop();
            }
        }
        if input.consume_shortcut(&PLAYBACK_SKIP) {
            app.skip();
        }
        if input.consume_shortcut(&PLAYBACK_SKIPBACK) {
            app.skip_back();
        }
        if input.consume_shortcut(&PLAYBACK_SHUFFLE) {
            app.toggle_shuffle();
        }
        if input.consume_shortcut(&PLAYBACK_VOLUP) {
            app.set_volume(app.volume + 5.);
        }
        if input.consume_shortcut(&PLAYBACK_VOLDN) {
            app.set_volume(app.volume - 5.);
        }

        // --- Workspaces

        if input.consume_shortcut(&WORKSPACE_SWITCHLEFT) {
            app.switch_workspace_left();
        }
        if input.consume_shortcut(&WORKSPACE_SWITCHRIGHT) {
            app.switch_workspace_right();
        }
        if input.consume_shortcut(&WORKSPACE_MOVELEFT) {
            app.move_workspace_left();
        }
        if input.consume_shortcut(&WORKSPACE_MOVERIGHT) {
            app.move_workspace_right();
        }
        if input.consume_shortcut(&WORKSPACE_CREATE) {
            app.new_workspace();
        }
        if input.consume_shortcut(&WORKSPACE_REMOVE) {
            app.remove_workspace(app.workspace_idx);
        }

        // --- GUI

        if input.consume_shortcut(&GUI_SHOWFONTS) {
            app.show_soundfonts = !app.show_soundfonts;
        }
    });
}
