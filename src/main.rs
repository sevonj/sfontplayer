use std::{thread, time::Duration};

use eframe::egui::{Context, ViewportBuilder, ViewportCommand};
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
    #[serde(skip)]
    player: Player,
    gui_state: GuiState,
}

impl SfontPlayer {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        update_thread(cc.egui_ctx.clone());

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self::default()
    }

    fn handle_events(&mut self, ctx: &Context) {
        let event_queue = self.player.get_event_queue();
        while !event_queue.is_empty() {
            match event_queue.remove(0) {
                player::PlayerEvent::Raise => {
                    ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                    ctx.send_viewport_cmd(ViewportCommand::Focus);
                }
                player::PlayerEvent::Exit => ctx.send_viewport_cmd(ViewportCommand::Close),
                player::PlayerEvent::NotifyError(message) => self.gui_state.toast_error(message),
            }
        }
    }
}

impl eframe::App for SfontPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);

        if let Err(e) = self.player.save_state() {
            println!("{e}");
            self.gui_state.toast_error("Saving app state failed.");
        }
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Run app logic
        self.player.update();
        self.handle_events(ctx);

        // Draw gui
        egui_extras::install_image_loaders(ctx);
        draw_gui(ctx, &mut self.player, &mut self.gui_state);
        self.gui_state.update_flags.clear();

        // Repaint continuously while playing
        if !self.player.is_paused() {
            ctx.request_repaint();
        }
    }
}

const THREAD_SLEEP: Duration = Duration::from_millis(200);

/// Request repaint periodically because app logic needs to run.
fn update_thread(ctx: Context) {
    thread::spawn(move || loop {
        ctx.request_repaint();
        thread::sleep(THREAD_SLEEP);
    });
}
