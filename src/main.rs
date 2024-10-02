use eframe::egui::{Context, ViewportBuilder};
use gui::{draw_gui, GuiState};
use player::Player;

mod gui;
mod player;

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_app_id("jyls_sfontplayer")
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "SfontPlayer",
        native_options,
        Box::new(|cc| Ok(Box::new(SfontPlayer::new(cc)))),
    );
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct SfontPlayer {
    player: Player,
    gui_state: GuiState,
}

impl SfontPlayer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self::default()
    }
}

impl eframe::App for SfontPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Make sure image loader exists!
        egui_extras::install_image_loaders(ctx);

        self.player.update();

        draw_gui(ctx, &mut self.player, &mut self.gui_state);

        let notifications = self.player.get_notification_queue_mut();
        while !notifications.is_empty() {
            self.gui_state.toast_error(notifications.remove(0));
        }
        self.gui_state.toasts.show(ctx);

        if !self.player.is_paused() {
            ctx.request_repaint();
        }

        self.gui_state.update_flags.clear();
    }
}
