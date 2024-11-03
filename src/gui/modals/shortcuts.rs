use eframe::egui::{vec2, Align2, Context, Label, RichText, ScrollArea, TextWrapMode, Ui, Window};
use egui_extras::{Column, TableBuilder};

use crate::{
    gui::keyboard_shortcuts::{
        GUI_SETTINGS, GUI_SHOWFONTS, PLAYBACK_PLAYPAUSE, PLAYBACK_SHUFFLE, PLAYBACK_SKIP,
        PLAYBACK_SKIPBACK, PLAYBACK_STARTSTOP, PLAYBACK_VOLDN, PLAYBACK_VOLUP, WORKSPACE_CREATE,
        WORKSPACE_DUPLICATE, WORKSPACE_MOVELEFT, WORKSPACE_MOVERIGHT, WORKSPACE_REFRESH,
        WORKSPACE_REMOVE, WORKSPACE_SAVE, WORKSPACE_SAVEALL, WORKSPACE_SAVEAS,
        WORKSPACE_SWITCHLEFT, WORKSPACE_SWITCHRIGHT,
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
                                add_shortcut_title(ui, "Save all loose workspaces");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&WORKSPACE_SAVEALL));
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
                        body.row(16., |mut row| {
                            row.col(|ui| {
                                add_shortcut_title(ui, "Open settings");
                            });
                            row.col(|ui| {
                                ui.label(ctx.format_shortcut(&GUI_SETTINGS));
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
