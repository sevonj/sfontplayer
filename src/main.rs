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

/// GUI update flags. Cleared at the end of frame update.
#[derive(Default)]
struct UpdateFlags {
    scroll_to_song: bool,
}
impl UpdateFlags {
    fn clear(&mut self) {
        self.scroll_to_song = false;
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct SfontPlayer {
    // -- Audio
    #[serde(skip)]
    audioplayer: AudioPlayer,
    /// Ranges 0.0..=100.0 as in percentage.
    volume: f32,

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
    #[serde(skip)]
    show_about_modal: bool,
    #[serde(skip)]
    show_shortcut_modal: bool,

    #[serde(skip)]
    update_flags: UpdateFlags,
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

    // --- Playback Control

    /// Start playing (from a fully stopped state)
    fn start(&mut self) {
        self.playing_workspace_idx = self.workspace_idx;
        self.rebuild_queue();
        self.set_queue_idx(Some(0));
        self.play_selected_song();
    }
    /// Load currently selected song & font from workspace and start playing
    fn play_selected_song(&mut self) {
        self.audioplayer.stop_playback();
        let workspace = self.get_workspace_mut();
        if workspace.font_idx.is_none() {
            println!("load_song: no soundfont");
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

        self.update_volume();
        if let Err(e) = self.audioplayer.start_playback() {
            println!("{}", e);
        }
    }
    /// Stop playback
    fn stop(&mut self) {
        self.audioplayer.stop_playback();
        self.get_workspace_mut().queue_idx = None;
    }
    /// Unpause
    fn play(&mut self) {
        self.audioplayer.play();
    }
    /// Pause
    fn pause(&mut self) {
        self.audioplayer.pause()
    }
    /// Play previous song
    fn skip_back(&mut self) -> Result<(), ()> {
        if let Some(mut index) = self.get_queue_idx() {
            if index == 0 {
                return Ok(());
            }
            index -= 1;
            self.set_queue_idx(Some(index));
            self.play_selected_song();
            return Ok(());
        }
        // Ignore for now.
        Ok(())
    }
    /// Play next song
    fn skip(&mut self) -> Result<(), ()> {
        if let Some(mut index) = self.get_queue_idx() {
            index += 1;
            if index >= self.get_queue().len() {
                return Ok(());
            }
            self.set_queue_idx(Some(index));
            self.play_selected_song();
            return Ok(());
        }
        // Ignore for now.
        Ok(())
    }
    /// Toggles shuffle and rebuilds queue
    fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        self.rebuild_queue();
    }
    /// Set volume (safe)
    fn set_volume(&mut self, volume: f32) {
        self.volume = f32::clamp(volume, 0., 100.)
    }
    /// Sends current volume setting to backend
    fn update_volume(&mut self) {
        // Not dividing the volume by 100 is a mistake you only make once.
        self.audioplayer.set_volume(self.volume * 0.001)
    }

    // --- Playback Status

    /// Pause status.
    fn is_paused(&self) -> bool {
        self.audioplayer.is_paused()
    }
    /// Finished playing, no current song.
    fn is_empty(&self) -> bool {
        self.audioplayer.is_empty()
    }
    /// Get total length of currently playing file
    fn get_midi_length(&self) -> Duration {
        if let Some(len) = self.audioplayer.get_midi_length() {
            return len;
        }
        Duration::ZERO
    }
    /// Current playback position
    fn get_midi_position(&self) -> Duration {
        self.audioplayer.get_midi_position()
    }

    // --- Manage Workspaces

    /// Get a reference to the workspace list
    fn get_workspaces(&self) -> &Vec<Workspace> {
        &self.workspaces
    }
    /// Get a reference to the currently open workspace
    fn get_workspace(&self) -> &Workspace {
        &self.workspaces[self.workspace_idx]
    }
    /// Get a mutable reference to the currently open workspace
    fn get_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.workspace_idx]
    }
    /// Switch to another workspace
    fn switch_workspace(&mut self, index: usize) {
        self.workspace_idx = index
    }
    fn switch_workspace_left(&mut self) {
        if self.workspace_idx > 0 {
            self.workspace_idx -= 1
        }
    }
    fn switch_workspace_right(&mut self) {
        if self.workspace_idx < self.workspaces.len() - 1 {
            self.workspace_idx += 1
        }
    }
    /// Create a new workspace
    fn new_workspace(&mut self) {
        self.workspaces.push(Workspace::default());
        self.workspace_idx = self.workspaces.len() - 1;
    }
    /// Remove a workspace by index
    fn remove_workspace(&mut self, index: usize) {
        self.workspace_delet_queue.push(index);
    }
    /// Rearrange workspaces
    fn move_workspace(&mut self, old_index: usize, new_index: usize) {
        let workspace = self.workspaces.remove(old_index); // Remove at old index
        self.workspaces.insert(new_index, workspace); // Reinsert at new index

        // Update current workspace index if it was affected by the move
        if old_index == self.workspace_idx {
            self.workspace_idx = new_index;
        } else if old_index > self.workspace_idx && self.workspace_idx <= new_index {
            self.playing_workspace_idx -= 1;
        } else if new_index > self.workspace_idx && self.workspace_idx <= old_index {
            self.playing_workspace_idx += 1;
        }
    }
    /// Move current workspace left
    fn move_workspace_left(&mut self) {
        if self.workspace_idx > 0 {
            self.move_workspace(self.workspace_idx, self.workspace_idx - 1);
        }
    }
    /// Move current workspace right
    fn move_workspace_right(&mut self) {
        if self.workspace_idx < self.workspaces.len() - 1 {
            self.move_workspace(self.workspace_idx, self.workspace_idx + 1);
        }
    }

    // --- Manage Workspace Soundfonts

    /// Get selected soundfont from currently open workspace
    fn get_font_idx(&mut self) -> Option<usize> {
        self.get_workspace().font_idx
    }
    /// Set selected soundfont in currently open workspace
    fn set_font_idx(&mut self, index: usize) {
        self.get_workspace_mut().font_idx = Some(index);
        self.play_selected_song();
    }
    /// Get a reference to the soundfont list of currently open workspace
    fn get_fonts(&mut self) -> &Vec<PathBuf> {
        &self.get_workspace().fonts
    }
    /// Add a soundfont to currently open workspace
    fn add_font(&mut self, path: PathBuf) {
        let workspace = self.get_workspace_mut();
        if !workspace.fonts.contains(&path) {
            workspace.fonts.push(path);
        }
    }
    /// Remove a soundfont from currently open workspace
    fn remove_font(&mut self, index: usize) {
        let workspace = self.get_workspace_mut();
        workspace.remove_font(index);
        if Some(index) == workspace.font_idx {
            self.stop();
        }
    }
    /// Remove all soundfonts from currently open workspace
    fn clear_fonts(&mut self) {
        let workspace = self.get_workspace_mut();
        workspace.fonts.clear();
        workspace.font_idx = None;
        self.stop();
    }

    // --- Manage Workspace Songs

    /// Get selected song from currently open workspace
    fn get_midi_idx(&mut self) -> Option<usize> {
        self.get_workspace().midi_idx
    }
    /// Set selected song in currently open workspace
    fn set_midi_idx(&mut self, index: usize) {
        self.get_workspace_mut().midi_idx = Some(index);
    }
    /// Get a reference to the song list of currently open workspace
    fn get_midis(&self) -> &Vec<MidiMeta> {
        &self.get_workspace().midis
    }
    /// Add a song to currently open workspace
    fn add_midi(&mut self, filepath: PathBuf) {
        if self.get_workspace().contains_midi(&filepath) {
            return;
        }
        self.get_workspace_mut().midis.push(MidiMeta::new(filepath));
    }
    /// Remove a song from currently open workspace
    fn remove_midi(&mut self, index: usize) {
        let workspace = self.get_workspace_mut();
        workspace.remove_midi(index);
        if Some(index) == workspace.midi_idx {
            self.stop();
        }
    }
    /// Remove all songs from currently open workspace
    fn clear_midis(&mut self) {
        let workspace = self.get_workspace_mut();
        workspace.midis.clear();
        workspace.midi_idx = None;
        self.stop();
    }

    // --- Playback queue

    /// Get a reference to the playback queue
    fn get_queue(&self) -> &Vec<usize> {
        &self.get_workspace().queue
    }
    /// Get the current index in queue
    fn get_queue_idx(&self) -> Option<usize> {
        self.get_workspace().queue_idx
    }
    /// Set the current index in queue
    fn set_queue_idx(&mut self, queue_idx: Option<usize>) {
        self.get_workspace_mut().queue_idx = queue_idx;
    }
    /// Create a new queue from currently available songs.
    /// To be called when song list changes, or shuffle is toggled
    fn rebuild_queue(&mut self) {
        let shuffle = self.shuffle;
        let workspace = self.get_workspace_mut();
        workspace.queue.clear();

        // Sequential queue starting from currently selected song
        let first_song_idx = workspace.midi_idx;
        for i in 0..workspace.midis.len() {
            workspace.queue.push(i);
        }

        if shuffle {
            workspace.queue.shuffle(&mut rand::thread_rng());
            // Put current selected song to the beginnning.
            // If it doesn't exist, the first song is random result of the shuffle.
            if let Some(song_idx) = first_song_idx {
                workspace.queue.retain(|&x| x != song_idx); // Remove song from queue
                workspace.queue.insert(0, song_idx); // Insert it to the beginning.
            }
        }
    }
}

impl eframe::App for SfontPlayer {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Make sure umage loader exists!
        egui_extras::install_image_loaders(ctx);

        // Make sure at least one workspace exists!
        if self.workspaces.is_empty() {
            self.new_workspace();
        }

        // When previous song has ended, advance queue or stop.
        if !self.audioplayer.is_paused() && self.audioplayer.is_empty() {
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

        if !self.is_paused() {
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

        self.update_flags.clear();
    }
}
