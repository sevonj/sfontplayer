//! Player app logic module

use anyhow::bail;
use audio::AudioPlayer;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::time::Duration;
use workspace::Workspace;

pub mod audio;
pub mod workspace;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Default, Clone, Copy)]
#[repr(u8)]
pub enum RepeatMode {
    #[default]
    Disabled,
    Queue,
    Song,
}

/// The Player class does high-level app logic, which includes workspaces and playback.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Player {
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

    // -- Misc
    notification_queue: Vec<String>,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            audioplayer: AudioPlayer::default(),
            volume: 100.,
            is_playing: false,

            workspaces: vec![],
            workspace_idx: 0,
            workspace_delet_queue: vec![],
            playing_workspace_idx: 0,

            shuffle: false,
            repeat: RepeatMode::Disabled,
            notification_queue: vec![],
        }
    }
}

impl Player {
    /// GUI frame update
    pub fn update(&mut self) {
        self.ensure_workspace_existence();

        if !self.is_paused() && self.is_empty() {
            self.advance_queue();
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

    // --- Playback Control

    /// Start playing (from a fully stopped state)
    pub fn start(&mut self) {
        self.playing_workspace_idx = self.workspace_idx;
        let shuffle = self.shuffle;
        self.get_playing_workspace_mut().rebuild_queue(shuffle);
        if let Err(e) = self.play_selected_song() {
            self.notification_queue.push(e.to_string());
        }
    }
    /// Load currently selected song & font from workspace and start playing
    pub fn play_selected_song(&mut self) -> anyhow::Result<()> {
        self.audioplayer.stop_playback();
        let workspace = self.get_playing_workspace_mut();

        let Some(font_index) = workspace.get_font_idx() else {
            bail!("No soundfont selected!");
        };
        let Some(queue_index) = workspace.queue_idx else {
            bail!("Can't load song: No queue index!");
        };
        let midi_index = workspace.queue[queue_index];

        workspace.set_song_idx(Some(midi_index))?;

        // Font Error Guard
        workspace.get_fonts_mut()[font_index].refresh();
        workspace.get_fonts()[font_index].get_status()?;

        // Midi Error Guard
        workspace.get_songs_mut()[midi_index].refresh();
        workspace.get_songs()[midi_index].get_status()?;

        // Play
        let sf = workspace.get_fonts()[font_index].get_path();
        let mid = workspace.get_songs()[midi_index].get_path();
        self.audioplayer.set_soundfont(sf);
        self.audioplayer.set_midifile(mid);
        self.is_playing = true;

        self.update_volume();
        self.audioplayer.start_playback()?;

        Ok(())
    }
    /// Stop playback
    pub fn stop(&mut self) {
        self.audioplayer.stop_playback();
        self.get_playing_workspace_mut().queue_idx = None;
        self.is_playing = false;
    }
    /// Unpause
    pub fn play(&self) {
        self.audioplayer.play();
    }
    /// Pause
    pub fn pause(&self) {
        self.audioplayer.pause();
    }
    /// Play previous song
    pub fn skip_back(&mut self) {
        if let Some(mut index) = self.get_playing_workspace().queue_idx {
            if index == 0 {
                if self.repeat == RepeatMode::Queue {
                    index = self.get_playing_workspace().queue.len() - 1;
                    self.get_playing_workspace_mut().queue_idx = Some(index);
                    if let Err(e) = self.play_selected_song() {
                        self.notification_queue.push(e.to_string());
                    }
                    return;
                }
                return;
            }
            index -= 1;
            self.get_playing_workspace_mut().queue_idx = Some(index);
            if let Err(e) = self.play_selected_song() {
                self.notification_queue.push(e.to_string());
            }
        }
    }
    /// Play next song
    pub fn skip(&mut self) {
        if let Some(mut index) = self.get_playing_workspace().queue_idx {
            index += 1;
            if index >= self.get_playing_workspace().queue.len() {
                if self.repeat == RepeatMode::Queue {
                    self.get_playing_workspace_mut().queue_idx = Some(0);
                    if let Err(e) = self.play_selected_song() {
                        self.notification_queue.push(e.to_string());
                    }
                    return;
                }
                return;
            }
            self.get_playing_workspace_mut().queue_idx = Some(index);
            if let Err(e) = self.play_selected_song() {
                self.notification_queue.push(e.to_string());
            }
        }
    }
    pub const fn get_shuffle(&self) -> bool {
        self.shuffle
    }
    /// Toggle shuffle and rebuild queue
    pub fn toggle_shuffle(&mut self) {
        let shuffle = !self.shuffle;
        self.shuffle = shuffle;
        self.get_playing_workspace_mut().rebuild_queue(shuffle);
    }
    pub const fn get_repeat(&self) -> RepeatMode {
        self.repeat
    }
    pub fn cycle_repeat(&mut self) {
        match self.repeat {
            RepeatMode::Disabled => self.repeat = RepeatMode::Queue,
            RepeatMode::Queue => self.repeat = RepeatMode::Song,
            RepeatMode::Song => self.repeat = RepeatMode::Disabled,
        }
    }
    pub const fn get_volume(&self) -> f32 {
        self.volume
    }
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = f32::clamp(volume, 0., 100.);
    }
    /// Sends current volume setting to backend
    pub fn update_volume(&self) {
        // Not dividing the volume by 100 is a mistake you only make once.
        self.audioplayer.set_volume(self.volume * 0.001);
    }
    // When previous song has ended, advance queue or stop.
    fn advance_queue(&mut self) {
        let repeat = self.repeat;
        let workspace = self.get_playing_workspace_mut();

        let Some(mut queue_index) = workspace.queue_idx else {
            let _ = workspace.set_song_idx(None);
            self.stop();
            return;
        };

        if repeat == RepeatMode::Song {
            let _ = workspace.set_song_idx(Some(workspace.queue[queue_index]));
            if let Err(e) = self.play_selected_song() {
                self.notification_queue.push(e.to_string());
            }
            return;
        }

        queue_index += 1;

        // End reached, loop back or bail out
        if queue_index == workspace.queue.len() {
            if repeat == RepeatMode::Queue {
                queue_index = 0;
            } else {
                let _ = workspace.set_song_idx(None);
                self.stop();
                return;
            }
        }

        workspace.queue_idx = Some(queue_index);

        let _ = workspace.set_song_idx(Some(workspace.queue[queue_index]));
        if let Err(e) = self.play_selected_song() {
            self.notification_queue.push(e.to_string());
        }
    }

    // --- Playback Status

    pub const fn is_playing(&self) -> bool {
        self.is_playing
    }
    /// Pause status.
    pub fn is_paused(&self) -> bool {
        self.audioplayer.is_paused()
    }
    /// Finished playing, no current song.
    pub fn is_empty(&self) -> bool {
        self.audioplayer.is_empty()
    }
    /// Get total length of currently playing file
    pub const fn get_playback_length(&self) -> Duration {
        if let Some(len) = self.audioplayer.get_midi_length() {
            return len;
        }
        Duration::ZERO
    }
    pub fn get_playback_position(&self) -> Duration {
        self.audioplayer.get_midi_position()
    }

    // --- Manage Workspaces

    /// Index of currently open workspace
    pub const fn get_workspace_idx(&self) -> usize {
        self.workspace_idx
    }
    /// Index of currently playing workspace
    pub const fn get_playing_workspace_idx(&self) -> usize {
        self.playing_workspace_idx
    }
    /// Get a reference to the workspace list
    pub const fn get_workspaces(&self) -> &Vec<Workspace> {
        &self.workspaces
    }
    /// Get a reference to the currently open workspace
    pub fn get_workspace(&self) -> &Workspace {
        &self.workspaces[self.workspace_idx]
    }
    /// Get a mutable reference to the currently open workspace
    pub fn get_workspace_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.workspace_idx]
    }
    /// Get a reference to the currently playing workspace.
    /// If nothing's playing, it gives the currently open workspace instead.
    pub fn get_playing_workspace(&self) -> &Workspace {
        if self.is_playing {
            return &self.workspaces[self.playing_workspace_idx];
        }
        &self.workspaces[self.workspace_idx]
    }
    /// Get a mutable reference to the currently playing workspace.
    /// If nothing's playing, it gives the currently open workspace instead.
    pub fn get_playing_workspace_mut(&mut self) -> &mut Workspace {
        if self.is_playing {
            return &mut self.workspaces[self.playing_workspace_idx];
        }
        &mut self.workspaces[self.workspace_idx]
    }
    /// Switch to another workspace
    pub fn switch_to_workspace(&mut self, index: usize) {
        self.workspace_idx = index;
    }
    pub fn switch_workspace_left(&mut self) {
        if self.workspace_idx > 0 {
            self.workspace_idx -= 1;
        }
    }
    pub fn switch_workspace_right(&mut self) {
        if self.workspace_idx < self.workspaces.len() - 1 {
            self.workspace_idx += 1;
        }
    }
    /// Create a new workspace
    pub fn new_workspace(&mut self) {
        self.workspaces.push(Workspace::default());
        self.workspace_idx = self.workspaces.len() - 1;
    }
    /// Remove a workspace by index
    pub fn remove_workspace(&mut self, index: usize) {
        self.workspace_delet_queue.push(index);
        self.ensure_workspace_existence();
    }
    /// Rearrange workspaces
    pub fn move_workspace(&mut self, old_index: usize, new_index: usize) {
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
    pub fn move_workspace_left(&mut self) {
        if self.workspace_idx > 0 {
            self.move_workspace(self.workspace_idx, self.workspace_idx - 1);
        }
    }
    /// Move current workspace right
    pub fn move_workspace_right(&mut self) {
        if self.workspace_idx < self.workspaces.len() - 1 {
            self.move_workspace(self.workspace_idx, self.workspace_idx + 1);
        }
    }
    /// Make sure at least one workspace exists!
    fn ensure_workspace_existence(&mut self) {
        if self.get_workspaces().is_empty() {
            self.new_workspace();
        }
    }

    // --- Other

    pub fn get_notification_queue_mut(&mut self) -> &mut Vec<String> {
        &mut self.notification_queue
    }
}
