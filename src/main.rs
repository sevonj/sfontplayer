use eframe::egui::{mutex::Mutex, Context, ViewportBuilder, ViewportCommand};
use gui::{draw_gui, GuiState};
use midi_inspector::MidiInspector;
use player::{
    playlist::{MidiMeta, Playlist},
    Player, PlayerEvent,
};
use rodio::{OutputStream, Sink};
use std::{
    env,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

mod gui;
mod midi_inspector;
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
    midi_inspector: Option<MidiInspector>,
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
            midi_inspector: None,
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

        let mut new_playlist = Playlist::default();
        new_playlist.name = "Opened files".into();

        for (i, arg) in args.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if std::path::Path::new(arg)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("midpl"))
            {
                if let Err(e) = player.open_portable_playlist(arg.into()) {
                    self.gui_state.toast_error(e.to_string());
                }
            } else if let Err(e) = new_playlist.add_file(arg.into()) {
                self.gui_state.toast_error(e.to_string());
            }
        }
        let has_fonts = !new_playlist.get_fonts().is_empty();
        let has_songs = !new_playlist.get_songs().is_empty();

        if has_fonts || has_songs {
            player.get_playlists_mut().push(new_playlist);
            let index = player.get_playlists().len() - 1;
            player.switch_to_playlist(index).expect("unreachable");
        }
        if has_songs {
            player.start();
        }
    }

    /// Cancels app exit if needed
    fn quit_check(&mut self, ctx: &Context) {
        if !ctx.input(|i| i.viewport().close_requested()) {
            return;
        }
        let player = self.player.lock();
        if player.autosave {
            return;
        }
        if self.gui_state.force_quit {
            return;
        }

        for playlist in player.get_playlists() {
            if playlist.has_unsaved_changes() {
                self.gui_state.show_unsaved_quit_modal = true;
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            }
        }
    }
}

impl eframe::App for SfontPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let mut player = self.player.lock();
        if player.debug_block_saving {
            return;
        }
        eframe::set_value(storage, eframe::APP_KEY, self);

        if let Err(e) = player.save_state() {
            self.gui_state
                .toast_error(format!("Saving app state failed: {e}"));
        }
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // App logic
        {
            let mut player = self.player.lock();
            player.update();
            handle_events(&mut player, &mut self.gui_state, ctx);
            // Repaint continuously while playing
            if !player.is_paused() {
                ctx.request_repaint();
            }
        }

        // Draw
        egui_extras::install_image_loaders(ctx);
        draw_gui(ctx, self);

        if self.gui_state.update_flags.close_midi_inspector {
            self.midi_inspector = None;
            self.player.lock().clear_midi_override();
        } else if let Some(filepath) = &self.gui_state.update_flags.open_midi_inspector {
            if let Ok(insp) = MidiInspector::new(filepath) {
                self.midi_inspector = Some(insp);
                self.player
                    .lock()
                    .set_midi_override(MidiMeta::new(filepath.into()));
            }
        }

        self.gui_state.update_flags.clear();
        self.quit_check(ctx);
    }
}

fn handle_events(player: &mut Player, gui: &mut GuiState, ctx: &Context) {
    let event_queue = player.get_event_queue();
    while !event_queue.is_empty() {
        match event_queue.remove(0) {
            PlayerEvent::Raise => {
                ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(ViewportCommand::Focus);
            }
            PlayerEvent::Quit => ctx.send_viewport_cmd(ViewportCommand::Close),
            PlayerEvent::NotifyError(message) => gui.toast_error(message),
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
                player.lock().get_playlist_mut().recrawl_fonts();
                player.lock().get_playlist_mut().refresh_song_list();
            }

            prev_update = now;
            thread::sleep(THREAD_SLEEP);
        }
    });
}
