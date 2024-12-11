use eframe::egui::{vec2, Align2, Context, Label, RichText, ScrollArea, TextWrapMode, Ui, Window};
use egui_extras::{Column, TableBuilder};

use crate::{
    gui::keyboard_shortcuts::{
        GUI_QUIT, GUI_SETTINGS, GUI_SHORTCUTS, GUI_SHOWFONTS, PLAYBACK_PLAYPAUSE, PLAYBACK_REPEAT,
        PLAYBACK_SHUFFLE, PLAYBACK_SKIP, PLAYBACK_SKIPBACK, PLAYBACK_STARTSTOP, PLAYBACK_VOLDN,
        PLAYBACK_VOLUP, PLAYLIST, PLAYLIST_CREATE, PLAYLIST_DUPLICATE, PLAYLIST_MOVELEFT,
        PLAYLIST_MOVERIGHT, PLAYLIST_OPEN, PLAYLIST_REMOVE, PLAYLIST_REOPEN, PLAYLIST_SAVE,
        PLAYLIST_SAVEALL, PLAYLIST_SAVEAS, PLAYLIST_SWITCHLEFT, PLAYLIST_SWITCHRIGHT,
    },
    GuiState,
};

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
                                add_shortcut_title(ui, "Toggle shuffle");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_SHUFFLE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Cycle repeat");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYBACK_REPEAT));
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

                        // --- Playlists

                        body.row(16., |mut row| {
                            row.col(|ui| {
                                ui.label("Playlists");
                            });
                            row.col(|_| {});
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Switch to previous playlist (left)");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_SWITCHLEFT));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Switch to next playlist (right)");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_SWITCHRIGHT));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Move current playlist left");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_MOVELEFT));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Move current playlist right");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_MOVERIGHT));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Create new playlist");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_CREATE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Remove current playlist");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_REMOVE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Refresh playlist content");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Open playlist");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_OPEN));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Save playlist");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_SAVE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Save all playlists");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_SAVEALL));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Save playlist to a new file");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_SAVEAS));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Duplicate current playlist");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_DUPLICATE));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Reopen last closed playlist");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&PLAYLIST_REOPEN));
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
                                add_shortcut_title(ui, "Toggle font library sidebar");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&GUI_SHOWFONTS));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Open settings");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&GUI_SETTINGS));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Show shortcut list");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&GUI_SHORTCUTS));
                            });
                        });
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Quit the app");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&GUI_QUIT));
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
