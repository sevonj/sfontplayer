use std::{path::PathBuf, time::Duration};
extern crate rand;

use audio::AudioPlayer;
use data::{MidiMeta, Workspace};
use eframe::egui;
use gui::draw_gui;
use rand::seq::SliceRandom;

mod audio;
mod data;
mod gui;

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
    // -- Audio
    #[serde(skip)]
    audioplayer: AudioPlayer,

    // -- Data
    workspaces: Vec<Workspace>,
    workspace_idx: usize,
    /// Queued, because deletion will be requested in a loop.
    workspace_delet_queue: Vec<usize>,
    /// Which workspace was last playing music
    playing_workspace_idx: usize,

    // -- Settings
    shuffle: bool,
    show_soundfonts: bool,
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
    fn set_font_idx(&mut self, index: usize) {
        self.get_workspace_mut().font_idx = Some(index);
        self.load_song();
    }
    fn get_fonts(&mut self) -> Vec<PathBuf> {
        self.get_workspace().fonts.clone()
    }
    fn get_font_idx(&mut self) -> Option<usize> {
        self.get_workspace().font_idx
    }
    fn add_font(&mut self, path: PathBuf) {
        let workspace = self.get_workspace_mut();
        if !workspace.fonts.contains(&path) {
            workspace.fonts.push(path);
        }
    }
    fn remove_font(&mut self, index: usize) {
        let workspace = self.get_workspace_mut();
        workspace.remove_font(index);
        if Some(index) == workspace.font_idx {
            self.stop();
        }
    }
    fn clear_fonts(&mut self) {
        let workspace = self.get_workspace_mut();
        workspace.fonts.clear();
        workspace.font_idx = None;
        self.stop();
    }
    fn get_midis(&self) -> &Vec<MidiMeta> {
        &self.get_workspace().midis
    }
    fn get_midi_idx(&mut self) -> Option<usize> {
        self.get_workspace().midi_idx
    }
    fn set_midi_idx(&mut self, index: usize) {
        self.get_workspace_mut().midi_idx = Some(index);
    }
    fn add_midi(&mut self, filepath: PathBuf) {
        if self.get_workspace().contains_midi(&filepath) {
            return;
        }
        self.get_workspace_mut().midis.push(MidiMeta::new(filepath));
    }
    fn remove_midi(&mut self, index: usize) {
        let workspace = self.get_workspace_mut();
        workspace.remove_midi(index);
        if Some(index) == workspace.midi_idx {
            self.stop();
        }
    }
    fn clear_midis(&mut self) {
        let workspace = self.get_workspace_mut();
        workspace.midis.clear();
        workspace.midi_idx = None;
        self.stop();
    }
    fn start(&mut self) {
        self.playing_workspace_idx = self.workspace_idx;
        self.rebuild_queue();
        self.load_song();
    }
    fn load_song(&mut self) {
        self.audioplayer.stop_playback();
        let workspace = self.get_workspace_mut();
        if workspace.font_idx.is_none() {
            println!("load_song: no sf");
            return;
        }
        if let Some(idx) = workspace.queue_idx {
            workspace.midi_idx = Some(workspace.queue[idx]);
        } else {
            println!("load_song: no queue idx");
            return;
        }
        let sf = workspace.fonts[workspace.font_idx.unwrap()].clone();
        let mid = workspace.midis[workspace.midi_idx.unwrap()].get_path();
        self.audioplayer.set_soundfont(sf);
        self.audioplayer.set_midifile(mid);

        if let Err(e) = self.audioplayer.start_playback() {
            println!("{}", e);
            return;
        }
    }
    fn stop(&mut self) {
        self.audioplayer.stop_playback();
        self.get_workspace_mut().midi_idx = None;
        self.get_workspace_mut().queue_idx = None;
    }
    fn play(&mut self) {
        println!("Play");
        self.audioplayer.play();
    }
    fn pause(&mut self) {
        self.audioplayer.pause()
    }
    fn is_playing(&self) -> bool {
        self.audioplayer.is_playing()
    }
    fn is_empty(&self) -> bool {
        self.audioplayer.is_empty()
    }
    fn get_midi_length(&self) -> Duration {
        if let Some(len) = self.audioplayer.get_midi_length() {
            return len;
        }
        Duration::ZERO
    }
    fn get_midi_position(&self) -> Duration {
        self.audioplayer.get_midi_position()
    }
    fn rebuild_queue(&mut self) {
        let shuffle = self.shuffle;
        let workspace = self.get_workspace_mut();
        workspace.queue.clear();

        // Sequential queue starting from currently selected song
        let start = workspace.midi_idx.unwrap_or(0);
        workspace.queue_idx = Some(start);
        for i in 0..workspace.midis.len() {
            workspace.queue.push(i);
        }

        if shuffle {
            workspace.queue_idx = Some(0);
            workspace.queue.retain(|&x| x != start); // Remove first song
            workspace.queue.shuffle(&mut rand::thread_rng());
            workspace.queue.insert(0, start); // Reinsert first to the beginning.
        }

        println!("queue rebuilt: {:?}", self.get_workspace().queue);
    }
    fn get_workspace(&self) -> &Workspace {
        &self.workspaces[self.workspace_idx]
    }
    fn get_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.workspace_idx]
    }
    fn new_workspace(&mut self) {
        self.workspaces.push(Workspace::default());
    }
    fn remove_workspace(&mut self, index: usize) {
        self.workspace_delet_queue.push(index);
    }
    fn get_queue(&self) -> Vec<usize> {
        self.get_workspace().queue.clone()
    }
    fn get_queue_idx(&self) -> Option<usize> {
        self.get_workspace().queue_idx.clone()
    }
    fn set_queue_idx(&mut self, queue_idx: Option<usize>) {
        self.get_workspace_mut().queue_idx = queue_idx;
    }
}

impl eframe::App for SfontPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Make sure at least one workspace exists!
        if self.workspaces.is_empty() {
            self.new_workspace();
        }

        // When previous song has ended, advance queue or stop.
        if self.audioplayer.is_playing() && self.audioplayer.is_empty() {
            let workspace = self.get_workspace_mut();
            if let Some(mut idx) = workspace.queue_idx {
                idx += 1;
                workspace.queue_idx = Some(idx);
                if idx < workspace.queue.len() {
                    // Next song.
                    workspace.midi_idx = Some(workspace.queue[idx]);
                    self.start();
                } else {
                    // Reached the end.
                    workspace.queue_idx = None;
                }
            } else {
                workspace.midi_idx = None;
                self.stop();
            }
        }

        draw_gui(ctx, self);

        if self.is_playing() {
            ctx.request_repaint();
        }

        // Deletion queues
        self.get_workspace_mut().delete_queued();
        for index in self.workspace_delet_queue.clone() {
            self.workspaces.remove(index);

            // Deletion affected index. Note that we don't go below zero.
            if index <= self.workspace_idx && self.workspace_idx > 0 {
                self.workspace_idx -= 1;
            }
            if index <= self.playing_workspace_idx && self.playing_workspace_idx > 0 {
                self.playing_workspace_idx -= 1;
            }
        }
        self.workspace_delet_queue.clear();
    }
}
