use super::GuiState;
use crate::player::Player;
use eframe::egui::{
    vec2, Align, Align2, Button, Color32, Context, Layout, Response, RichText, Ui, ViewportCommand,
    WidgetText, Window,
};

pub mod about_modal;
pub mod file_dialogs;
pub mod settings;
pub mod shortcuts;

enum DialogButtonStyle {
    None,
    Suggested,
    Destructive,
}

/// Workspace close confirm with unsaved changes
pub fn unsaved_close_dialog(ctx: &Context, player: &mut Player) {
    let Some(index) = player.get_workspace_waiting_for_discard() else {
        return;
    };
    let name = player.get_workspaces()[index].name.clone();

    Window::new("Unsaved changes")
        .collapsible(false)
        .title_bar(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
        .show(ctx, |ui| {
            ui.set_width(420.);

            ui.add_space(12.);

            ui.horizontal(|ui| {
                ui.add_space(16.);
                ui.label(RichText::new("🎵").size(60.0));
                ui.vertical(|ui| {
                    ui.add_space(10.);
                    ui.heading("Unsaved changes");
                    ui.label("You have unsaved changes. Close this workspace?");
                    ui.label(format!("Workspace: {name}"));
                });
                ui.add_space(16.);
            });

            ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                ui.add_space(12.);

                if add_dialog_button(ui, "Discard", &DialogButtonStyle::Destructive).clicked() {
                    let _ = player.force_remove_workspace(index);
                };

                ui.add_enabled_ui(!player.debug_block_saving, |ui| {
                    if add_dialog_button(ui, "Save", &DialogButtonStyle::Suggested).clicked() {
                        let _ = player.save_portable_workspace(index);
                    };
                });

                if add_dialog_button(ui, "Cancel", &DialogButtonStyle::None).clicked() {
                    let _ = player.cancel_remove_workspace(index);
                };
            });
            ui.add_space(4.);
        });
}

/// App quit confirm with unsaved changes
pub fn unsaved_quit_dialog(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    if gui.show_unsaved_quit_modal {
        Window::new("Unsaved changes")
            .collapsible(false)
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, vec2(0., 0.))
            .show(ctx, |ui| {
                ui.set_width(420.);

                ui.add_space(12.);

                ui.horizontal(|ui| {
                    ui.add_space(16.);
                    ui.label(RichText::new("🎵").size(60.0));
                    ui.vertical(|ui| {
                        ui.add_space(10.);
                        ui.heading("Unsaved changes");
                        ui.label("You have unsaved changes. Are you sure you want to quit?");
                    });
                    ui.add_space(16.);
                });

                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    ui.add_space(12.);
                    if add_dialog_button(ui, "Discard and quit", &DialogButtonStyle::Destructive)
                        .clicked()
                    {
                        gui.force_quit = true;
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    };
                    ui.add_enabled_ui(!player.debug_block_saving, |ui| {
                        if add_dialog_button(ui, "Save all and quit", &DialogButtonStyle::Suggested)
                            .clicked()
                        {
                            let _ = player.save_all_portable_workspaces();
                            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                        };
                    });
                    if add_dialog_button(ui, "Cancel", &DialogButtonStyle::None).clicked() {
                        gui.show_unsaved_quit_modal = false;
                    };
                });
                ui.add_space(4.);
            });
    }
}

fn add_dialog_button<S>(ui: &mut Ui, text: S, style: &DialogButtonStyle) -> Response
where
    WidgetText: From<S>,
{
    let fill = match style {
        DialogButtonStyle::None => ui.style().visuals.widgets.active.bg_fill,
        DialogButtonStyle::Suggested => ui.style().visuals.selection.bg_fill,
        DialogButtonStyle::Destructive => Color32::from_rgba_unmultiplied(0x80, 0, 0, 0x80),
    };

    ui.add(Button::new(text).fill(fill))
}
