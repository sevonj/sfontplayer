use std::{
    env,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use eframe::egui::{mutex::Mutex, Context, ViewportBuilder, ViewportCommand};
use gui::{draw_gui, GuiState};
use player::{workspace::Workspace, Player};
use rodio::{OutputStream, Sink};

mod gui;
mod player;

fn main() {
    let args: Vec<String> = env::args().collect();

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
        Box::new(|cc| Ok(Box::new(SfontPlayer::new(cc, &args)))),
    );
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct SfontPlayer {
    #[serde(skip)]
    player: Arc<Mutex<Player>>,
    #[serde(skip)]
    stream: OutputStream,
    gui_state: GuiState,
}
impl Default for SfontPlayer {
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().expect("Could not create stream");
        let sink = Sink::try_new(&stream_handle).expect("Could not create sink");

        let mut player = Player::default();
        if let Err(e) = player.load_state() {
            println!("{e}");
        }
        let sfontplayer = Self {
            player: Arc::new(Mutex::new(player)),
            gui_state: GuiState::default(),
            stream,
        };
        sfontplayer.player.lock().set_sink(Some(sink));
        sfontplayer
    }
}

impl SfontPlayer {
    fn new(cc: &eframe::CreationContext<'_>, args: &[String]) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let mut sfontplayer = cc.storage.map_or_else(Self::default, |storage| {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        });
        sfontplayer.handle_launch_args(args);

        let player_clone = Arc::clone(&sfontplayer.player);
        update_thread(player_clone);

        sfontplayer
    }
    fn handle_launch_args(&mut self, args: &[String]) {
        let mut player = self.player.lock();

        let mut new_workspace = Workspace::default();
        new_workspace.name = "Opened files".into();

        for (i, arg) in args.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if arg.ends_with(".sfontspace") {
                if let Err(e) = player.open_portable_workspace(arg.into()) {
                    self.gui_state.toast_error(e.to_string());
                }
            } else if let Err(e) = new_workspace.add_file(arg.into()) {
                self.gui_state.toast_error(e.to_string());
            }
        }
        let has_fonts = !new_workspace.get_fonts().is_empty();
        let has_songs = !new_workspace.get_songs().is_empty();

        if has_fonts || has_songs {
            player.get_workspaces_mut().push(new_workspace);
            let index = player.get_workspaces().len() - 1;
            player.switch_to_workspace(index).expect("unreachable");
        }
        if has_songs {
            player.start();
        }
    }
}

impl eframe::App for SfontPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let player = self.player.lock();
        eframe::set_value(storage, eframe::APP_KEY, self);

        if let Err(e) = player.save_state() {
            println!("{e}");
            self.gui_state.toast_error("Saving app state failed.");
        }
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut player = self.player.lock();

        // Run app logic
        player.update();
        handle_events(&mut player, &mut self.gui_state, ctx);

        // Draw gui
        egui_extras::install_image_loaders(ctx);
        draw_gui(ctx, &mut player, &mut self.gui_state);
        self.gui_state.update_flags.clear();

        // Repaint continuously while playing
        if !player.is_paused() {
            ctx.request_repaint();
        }
    }
}

fn handle_events(player: &mut Player, gui: &mut GuiState, ctx: &Context) {
    let event_queue = player.get_event_queue();
    while !event_queue.is_empty() {
        match event_queue.remove(0) {
            player::PlayerEvent::Raise => {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(ViewportCommand::Focus);
            }
            player::PlayerEvent::Exit => ctx.send_viewport_cmd(ViewportCommand::Close),
            player::PlayerEvent::NotifyError(message) => gui.toast_error(message),
        }
    }
}

const THREAD_SLEEP: Duration = Duration::from_millis(200);
const FILELIST_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

fn update_thread(player: Arc<Mutex<Player>>) {
    thread::spawn(move || {
        let mut t_since_file_refresh = Duration::ZERO;
        let mut prev_update = Instant::now();

        loop {
            player.lock().update();

            let now = Instant::now();
            t_since_file_refresh += now - prev_update;
            if t_since_file_refresh >= FILELIST_REFRESH_INTERVAL {
                t_since_file_refresh -= FILELIST_REFRESH_INTERVAL;
                player.lock().get_workspace_mut().refresh_font_list();
                player.lock().get_workspace_mut().refresh_song_list();
            }

            prev_update = now;
            thread::sleep(THREAD_SLEEP);
        }
    });
}
