use std::time::Duration;

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
        // Run app logic
        self.player.update();

        // Draw gui
        egui_extras::install_image_loaders(ctx);
        draw_gui(ctx, &mut self.player, &mut self.gui_state);
        self.gui_state.update_flags.clear();

        // Repaint continuously while playing
        if !self.player.is_paused() {
            ctx.request_repaint();
        }

        // Repaint periodically because app logic needs to run.
        if !ctx.has_requested_repaint() {
            ctx.request_repaint_after(Duration::from_millis(500));
        };
    }
}
