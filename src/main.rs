use std::time::Duration;
extern crate rand;

use audio::AudioPlayer;
use eframe::egui;
use gui::draw_gui;
use serde_repr::{Deserialize_repr, Serialize_repr};
use workspace::Workspace;

mod audio;
mod gui;
mod workspace;

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

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Default, Clone, Copy)]
#[repr(u8)]
enum RepeatMode {
    #[default]
    Disabled,
    Queue,
    Song,
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct SfontPlayer {
    // -- Audio
    #[serde(skip)]
    audioplayer: AudioPlayer,
    /// Ranges 0.0..=100.0 as in percentage.
    volume: f32,
    /// Is there playback going on? Paused playback also counts.
    #[serde(skip)]
    is_playing: bool,

    // -- Data
    workspaces: Vec<Workspace>,
    workspace_idx: usize,
    /// Queued, because deletion will be requested in a loop.
    workspace_delet_queue: Vec<usize>,
    /// Which workspace was last playing music
    playing_workspace_idx: usize,

    // -- settings
    shuffle: bool,
    repeat: RepeatMode,

    // --- GUI
    show_soundfonts: bool,
    #[serde(skip)]
    show_about_modal: bool,
    #[serde(skip)]
    show_shortcut_modal: bool,
    #[serde(skip)]
    update_flags: UpdateFlags,
}

impl Default for SfontPlayer {
    fn default() -> Self {
        Self {
            audioplayer: Default::default(),
            volume: 100.,
            is_playing: false,

            workspaces: Default::default(),
            workspace_idx: 0,
            workspace_delet_queue: Default::default(),
            playing_workspace_idx: Default::default(),

            shuffle: false,
            repeat: RepeatMode::Disabled,

            show_soundfonts: Default::default(),
            show_about_modal: false,
            show_shortcut_modal: false,
            update_flags: Default::default(),
        }
    }
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
        let shuffle = self.shuffle;
        self.get_playing_workspace_mut().rebuild_queue(shuffle);
        self.play_selected_song();
    }
    /// Load currently selected song & font from workspace and start playing
    fn play_selected_song(&mut self) {
        self.audioplayer.stop_playback();
        let workspace = self.get_playing_workspace_mut();

        let font_index = match workspace.get_font_idx() {
            Some(index) => index,
            None => {
                println!("load_song: no soundfont");
                return;
            }
        };
        let queue_index = match workspace.queue_idx {
            Some(index) => index,
            None => {
                println!("load_song: no queue idx");
                return;
            }
        };
        let midi_index = workspace.queue[queue_index];
        workspace.set_song_idx(Some(midi_index));

        // Font Error Guard
        workspace.fonts[font_index].refresh();
        if let Some(e) = workspace.fonts[font_index].get_error() {
            println!("{}", e);
            return;
        }
        // Midi Error Guard
        workspace.midis[midi_index].refresh();
        if let Some(e) = workspace.midis[midi_index].get_error() {
            println!("{}", e);
            self.advance_queue();
            return;
        }

        // Play
        let sf = workspace.fonts[font_index].get_path();
        let mid = workspace.midis[midi_index].get_path();
        self.audioplayer.set_soundfont(sf);
        self.audioplayer.set_midifile(mid);
        self.is_playing = true;

        self.update_volume();
        if let Err(e) = self.audioplayer.start_playback() {
            println!("{}", e);
        }
    }
    /// Stop playback
    fn stop(&mut self) {
        self.audioplayer.stop_playback();
        self.get_playing_workspace_mut().queue_idx = None;
        self.is_playing = false;
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
    fn skip_back(&mut self) {
        if let Some(mut index) = self.get_playing_workspace().queue_idx {
            if index == 0 {
                if self.repeat == RepeatMode::Queue {
                    index = self.get_playing_workspace().queue.len() - 1;
                    self.get_playing_workspace_mut().queue_idx = Some(index);
                    self.play_selected_song();
                    return;
                }
                return;
            }
            index -= 1;
            self.get_playing_workspace_mut().queue_idx = Some(index);
            self.play_selected_song();
        }
    }
    /// Play next song
    fn skip(&mut self) {
        if let Some(mut index) = self.get_playing_workspace().queue_idx {
            index += 1;
            if index >= self.get_playing_workspace().queue.len() {
                if self.repeat == RepeatMode::Queue {
                    self.get_playing_workspace_mut().queue_idx = Some(0);
                    self.play_selected_song();
                    return;
                }
                return;
            }
            self.get_playing_workspace_mut().queue_idx = Some(index);
            self.play_selected_song();
        }
    }
    /// Toggles shuffle and rebuilds queue
    fn toggle_shuffle(&mut self) {
        let shuffle = !self.shuffle;
        self.shuffle = shuffle;
        self.get_playing_workspace_mut().rebuild_queue(shuffle);
    }
    /// Cycles repeat modes
    fn cycle_repeat(&mut self) {
        match self.repeat {
            RepeatMode::Disabled => self.repeat = RepeatMode::Queue,
            RepeatMode::Queue => self.repeat = RepeatMode::Song,
            RepeatMode::Song => self.repeat = RepeatMode::Disabled,
        }
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
    /// Get a reference to the currently playing workspace.
    /// If nothing's playing, it gives the currently open workspace instead.
    fn get_playing_workspace(&self) -> &Workspace {
        if self.is_playing {
            return &self.workspaces[self.playing_workspace_idx];
        }
        &self.workspaces[self.workspace_idx]
    }
    /// Get a mutable reference to the currently playing workspace.
    /// If nothing's playing, it gives the currently open workspace instead.
    fn get_playing_workspace_mut(&mut self) -> &mut Workspace {
        if self.is_playing {
            return &mut self.workspaces[self.playing_workspace_idx];
        }
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

    // When previous song has ended, advance queue or stop.
    fn advance_queue(&mut self) {
        let repeat = self.repeat;
        let workspace = self.get_playing_workspace_mut();

        let mut queue_index = match workspace.queue_idx {
            Some(value) => value,
            None => {
                workspace.set_song_idx(None);
                self.stop();
                return;
            }
        };

        if repeat == RepeatMode::Song {
            workspace.set_song_idx(Some(workspace.queue[queue_index]));
            self.play_selected_song();
            return;
        }

        queue_index += 1;

        // End reached, loop back or bail out
        if queue_index == workspace.queue.len() {
            if repeat == RepeatMode::Queue {
                queue_index = 0;
            } else {
                workspace.set_song_idx(None);
                self.stop();
                return;
            }
        }

        workspace.queue_idx = Some(queue_index);

        workspace.set_song_idx(Some(workspace.queue[queue_index]));
        self.play_selected_song();
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

        if !self.audioplayer.is_paused() && self.audioplayer.is_empty() {
            self.advance_queue();
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
