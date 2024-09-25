use eframe::egui::{
    vec2, Align2, Context, Key, KeyboardShortcut, Label, Modifiers, RichText, ScrollArea, Ui,
    Window,
};
use egui_extras::{Column, TableBuilder};

use crate::SfontPlayer;

const PLAYBACK_PLAYPAUSE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::Space);
const PLAYBACK_STARTSTOP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Space);
const PLAYBACK_SKIP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Period);
const PLAYBACK_SKIPBACK: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Comma);
const PLAYBACK_SHUFFLE: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::S);
const PLAYBACK_VOLUP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::ArrowUp);
const PLAYBACK_VOLDN: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::ArrowDown);

/// Modal window that shows Hotkeys
pub(crate) fn shortcut_modal(ctx: &Context, app: &mut SfontPlayer) {
    Window::new("Keyboard Shortcuts")
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
        .open(&mut app.show_shortcut_modal)
        .show(ctx, |ui| {
            ui.set_width(300.);
            ScrollArea::vertical().max_height(500.).show(ui, |ui| {
                let col_width = ui.available_width();

                TableBuilder::new(ui)
                    .vscroll(false)
                    .striped(true)
                    .column(Column::exact(col_width * 0.5))
                    .column(Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label("Name");
                        });
                        header.col(|ui| {
                            ui.label("Shortcut");
                        });
                    })
                    .body(|mut body| {
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
                                add_shortcut_title(ui, "Skip (next song)");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_SKIP));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Skip back (previous song))");
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
                    });
            });
        });
}

fn add_shortcut_title(ui: &mut Ui, text: &str) {
    // Slightly less intense color than Strong.
    let color = ui.visuals().widgets.open.text_color();
    ui.add(Label::new(RichText::new(text).color(color)));
}

/// Check and act on hotkeys
pub(crate) fn consume_shortcuts(ctx: &Context, app: &mut SfontPlayer) {
    if ctx.is_context_menu_open() {
        return;
    }

    ctx.input_mut(|input| {
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
            let _ = app.skip();
        }
        if input.consume_shortcut(&PLAYBACK_SKIPBACK) {
            let _ = app.skip_back();
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
    });
}
