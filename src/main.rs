use std::{default, path::PathBuf};

use audiosynth::AudioSynth;
use eframe::egui;
use gui::draw_gui;

mod audiosynth;
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
    synth: AudioSynth,
    pub(crate) soundfonts: Vec<PathBuf>,
    pub(crate) midis: Vec<PathBuf>,
    selected_sf: Option<usize>,
    #[serde(skip)]
    selected_midi: Option<usize>,
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
    fn play(&mut self) {
        if self.selected_midi.is_none() || self.selected_sf.is_none() {
            return;
        }
        if let Err(e) = self
            .synth
            .load_soundfont(&self.soundfonts[self.selected_sf.unwrap()])
        {
            println!("{}", e);
            return;
        }
        if let Err(e) = self
            .synth
            .play_midi(&self.midis[self.selected_midi.unwrap()])
        {
            println!("{}", e);
            return;
        }
    }
}

impl eframe::App for SfontPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        draw_gui(ctx, self);
    }
}
