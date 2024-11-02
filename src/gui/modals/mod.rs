use super::GuiState;
use crate::player::Player;
use eframe::egui::{
    vec2, Align, Align2, Button, Color32, Context, Layout, Response, RichText, Ui, ViewportCommand,
    WidgetText, Window,
};

pub mod about_modal;
pub mod file_dialogs;
pub mod shortcuts;

enum DialogButtonStyle {
    None,
    Suggested,
    Destructive,
}

/// Modal window that shows "About"
pub fn unsaved_exit_dialog(ctx: &Context, player: &mut Player, gui: &mut GuiState) {
    if gui.show_unsaved_exit_modal {
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
                        ui.label("You have unsaved changes. Are you sure you want to exit?");
                    });
                    ui.add_space(16.);
                });

                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    ui.add_space(12.);
                    if add_dialog_button(ui, "Discard and exit", &DialogButtonStyle::Destructive)
                        .clicked()
                    {
                        gui.force_exit = true;
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    };
                    if add_dialog_button(ui, "Save all and exit", &DialogButtonStyle::Suggested)
                        .clicked()
                    {
                        player.save_all_portable_workspaces();
                        ui.ctx().send_viewport_cmd(ViewportCommand::Close);
                    };
                    if add_dialog_button(ui, "Cancel", &DialogButtonStyle::None).clicked() {
                        gui.show_unsaved_exit_modal = false;
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
