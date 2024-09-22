use std::path::PathBuf;

use audio::AudioPlayer;
use eframe::egui;
use gui::draw_gui;

mod audio;
mod gui;
mod state;

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
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
    audioplayer: AudioPlayer,
    pub(crate) soundfonts: Vec<PathBuf>,
    pub(crate) midis: Vec<PathBuf>,
    selected_sf: Option<usize>,
    #[serde(skip)]
    selected_midi: Option<usize>,
    queue: Vec<usize>,
    queue_idx: usize,
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

    fn remove_sf(&mut self, index: usize) {
        self.soundfonts.remove(index);
        // We deleted currently selected
        if Some(index) == self.selected_sf {
            self.selected_sf = None;
        }
        // Deletion affected index
        else if Some(index) < self.selected_sf {
            self.selected_sf = Some(self.selected_sf.unwrap() - 1)
        }
    }

    fn remove_midi(&mut self, index: usize) {
        println!("delet: {}", index);
        self.midis.remove(index);
        // We deleted currently selected
        if Some(index) == self.selected_midi {
            self.selected_midi = None;
        }
        // Deletion affected index
        else if Some(index) < self.selected_midi {
            self.selected_midi = Some(self.selected_midi.unwrap() - 1)
        }
    }

    fn start(&mut self) {
        self.rebuild_queue();
        self.load_song();
    }
    fn load_song(&mut self) {
        self.audioplayer.stop_playback();
        if self.selected_midi.is_none() || self.selected_sf.is_none() {
            return;
        }
        let sf = &self.soundfonts[self.selected_sf.unwrap()];
        let mid = &self.midis[self.selected_midi.unwrap()];
        self.audioplayer.set_soundfont(sf.clone());
        self.audioplayer.set_midifile(mid.clone());

        if let Err(e) = self.audioplayer.start_playback() {
            println!("{}", e);
            return;
        }
    }
    fn stop(&mut self) {
        self.audioplayer.stop_playback();
        self.selected_midi = None;
    }
    fn play(&mut self) {
        self.audioplayer.play();
    }
    fn pause(&mut self) {
        self.audioplayer.pause()
    }
    fn is_playing(&self) -> bool {
        self.audioplayer.is_playing()
    }
    fn can_play(&self) -> bool {
        if self.selected_sf.is_none() || self.selected_midi.is_none() {
            return false;
        }
        self.audioplayer.can_play()
    }
    fn get_midi_length(&self) -> f64 {
        if let Some(len) = self.audioplayer.get_midi_length() {
            return len.as_secs_f64();
        }
        return 0.;
    }
    fn get_midi_position(&self) -> f64 {
        if let Some(len) = self.audioplayer.get_midi_position() {
            return len.as_secs_f64();
        }
        return 0.;
    }
    fn rebuild_queue(&mut self) {
        self.queue.clear();
        self.queue_idx = 0;

        // Sequential queue starting from currently selected song
        let start = self.selected_midi.unwrap_or(0);
        for i in start..self.midis.len() {
            self.queue.push(i);
        }
    }
}

impl eframe::App for SfontPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Advance queue or stop.
        if self.audioplayer.is_playing() && !self.audioplayer.can_play() {
            self.queue_idx += 1;
            if self.queue_idx < self.queue.len() {
                self.selected_midi = Some(self.queue[self.queue_idx]);
                self.start();
            } else {
                self.selected_midi = None;
                self.stop();
            }
        }

        draw_gui(ctx, self);
        if self.is_playing() {
            ctx.request_repaint();
        }
    }
}
