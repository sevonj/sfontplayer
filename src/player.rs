//! Player app logic module

use anyhow::bail;
use audio::AudioPlayer;
use directories::ProjectDirs;
use eframe::egui::mutex::Mutex;
#[cfg(not(target_os = "windows"))]
use mediacontrols::create_mediacontrols;
use rodio::Sink;
use serde_json::{json, Value};
use serde_repr::{Deserialize_repr, Serialize_repr};
use souvlaki::{MediaControlEvent, MediaControls};
use std::{
    error, fmt,
    fs::{self, remove_file, File},
    io::Write,
    path::PathBuf,
    sync::Arc,
    time::Duration,
    vec,
};
use workspace::Workspace;

pub mod audio;
mod mediacontrols;
pub mod workspace;

/// To be handled in gui
pub enum PlayerEvent {
    /// Bring window to focus
    Raise,
    Exit,
    NotifyError(String),
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Default, Clone, Copy)]
#[repr(u8)]
pub enum RepeatMode {
    #[default]
    Disabled = 0,
    Queue = 1,
    Song = 2,
}
impl TryFrom<u8> for RepeatMode {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            x if x == Self::Disabled as u8 => Ok(Self::Disabled),
            x if x == Self::Queue as u8 => Ok(Self::Queue),
            x if x == Self::Song as u8 => Ok(Self::Song),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum PlayerError {
    InvalidWorkspaceIndex { index: usize },
    CantMoveWorkspace,
    CantSwitchWorkspace,
    NoQueueIndex,
    NoSoundfont,
}
impl error::Error for PlayerError {}
impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWorkspaceIndex { index } => {
                write!(f, "Workspace index {index} is out of bounds.")
            }
            Self::CantMoveWorkspace => write!(f, "Can't move this workspace further."),
            Self::CantSwitchWorkspace => write!(f, "Can't switch workspaces further."),
            Self::NoQueueIndex => write!(f, "No queue index!"),
            Self::NoSoundfont => write!(f, "No soundfont selected!"),
        }
    }
}

/// The Player class does high-level app logic, which includes workspaces and playback.
pub struct Player {
    // -- Audio
    audioplayer: AudioPlayer,
    /// Is there playback going on? Paused playback also counts.
    is_playing: bool,

    // -- Control
    /// Ranges 0.0..=100.0 as in percentage.
    volume: f32,
    /// OS integration
    #[cfg(not(target_os = "windows"))]
    mediacontrol: MediaControls,
    /// Events from system to the player.
    mediacontrol_events: Arc<Mutex<Vec<MediaControlEvent>>>,
    /// Events from player to the gui.
    #[allow(clippy::struct_field_names)]
    player_events: Vec<PlayerEvent>,

    // -- Data
    workspaces: Vec<Workspace>,
    /// Which workspace is open
    workspace_idx: usize,
    /// Which workspace was last playing music
    playing_workspace_idx: usize,
    /// Queued, because deletion will be requested in a loop.
    workspace_delet_queue: Vec<usize>,

    // -- settings
    shuffle: bool,
    repeat: RepeatMode,
}

impl Default for Player {
    fn default() -> Self {
        let mediacontrol_events = Arc::new(Mutex::new(vec![]));
        #[cfg(not(target_os = "windows"))]
        let mediacontrol = create_mediacontrols(Arc::clone(&mediacontrol_events));

        Self {
            audioplayer: AudioPlayer::default(),
            is_playing: false,

            volume: 100.,
            #[cfg(not(target_os = "windows"))]
            mediacontrol,
            mediacontrol_events,
            player_events: vec![],

            workspaces: vec![],
            workspace_idx: 0,
            playing_workspace_idx: 0,
            workspace_delet_queue: vec![],

            shuffle: false,
            repeat: RepeatMode::Disabled,
        }
    }
}

impl Player {
    pub fn save_state(&self) -> anyhow::Result<()> {
        let state_dir = state_dir();
        let workspace_dir = workspace_dir();
        fs::create_dir_all(&state_dir)?;
        fs::create_dir_all(&workspace_dir)?;

        let mut new_workspace_filepaths: Vec<PathBuf> = vec![];
        for (i, workspace) in self.workspaces.iter().enumerate() {
            let filename = format!(
                "{i:02}_{}.json",
                workspace
                    .name
                    .replace(|c: char| !c.is_ascii_alphanumeric(), "")
                    .to_lowercase()
            );
            let filepath = workspace_dir.join(filename);
            let mut file = File::create(&filepath)?;
            file.write_all(Value::from(workspace).to_string().as_bytes())?;
            new_workspace_filepaths.push(filepath);
        }
        let workspace_paths = fs::read_dir(workspace_dir)?;
        for file in workspace_paths {
            let filepath = file?.path();
            if !new_workspace_filepaths.contains(&filepath) {
                remove_file(filepath)?;
            }
        }

        let data = json! ({
            "shuffle": self.shuffle,
            "repeat": self.repeat,
            "workspace_idx": self.workspace_idx,
        });

        let config_file = state_dir.join("state.json");

        let mut file = File::create(config_file)?;
        file.write_all(data.to_string().as_bytes())?;

        Ok(())
    }

    pub fn load_state(&mut self) -> anyhow::Result<()> {
        let mut workspace_paths = fs::read_dir(workspace_dir())?
            .filter_map(std::result::Result::ok)
            .map(|e| e.path())
            .collect::<Vec<_>>();
        workspace_paths.sort(); // filenames are number-prefixed.

        for filepath in workspace_paths {
            println!("{filepath:?}");
            let data_string = std::fs::read_to_string(filepath)?;
            let data: Value = serde_json::from_str(&data_string)?;
            self.workspaces.push(Workspace::from(data));
        }

        let state_filepath = state_dir().join("state.json");
        let data_string = std::fs::read_to_string(state_filepath)?;
        let data: Value = serde_json::from_str(&data_string)?;

        self.shuffle = data["shuffle"].as_bool().is_some_and(|value| value);
        if let Some(repeat) = data["repeat"].as_u64() {
            self.repeat = RepeatMode::try_from(repeat as u8).unwrap_or_default();
        }
        self.workspace_idx = data["workspace_idx"].as_u64().unwrap_or(0) as usize;

        Ok(())
    }

    /// You need to give the audio player a sink before it can do anything.
    pub fn set_sink(&mut self, value: Option<Sink>) {
        self.audioplayer.set_sink(value);
    }

    /// GUI frame update
    pub fn update(&mut self) {
        self.ensure_workspace_existence();

        if !self.is_paused() && self.is_empty() {
            if let Err(e) = self.advance_queue() {
                self.push_error(e.to_string());
            }
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

        self.mediacontrol_handle_events();
    }

    // --- Playback Control

    /// Start playing (from a fully stopped state)
    pub fn start(&mut self) {
        self.playing_workspace_idx = self.workspace_idx;
        let shuffle = self.shuffle;
        self.get_playing_workspace_mut().rebuild_queue(shuffle);
        if let Err(e) = self.play_selected_song() {
            self.push_error(e.to_string());
        }
    }
    /// Load currently selected song & font from workspace and start playing
    fn play_selected_song(&mut self) -> anyhow::Result<()> {
        self.audioplayer.stop_playback()?;
        let workspace = self.get_playing_workspace_mut();

        let Some(font_index) = workspace.get_font_idx() else {
            bail!(PlayerError::NoSoundfont);
        };
        let Some(queue_index) = workspace.queue_idx else {
            bail!(PlayerError::NoQueueIndex);
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

        self.mediacontrol_update_song();

        Ok(())
    }

    /// Stop playback
    pub fn stop(&mut self) {
        let _ = self.audioplayer.stop_playback();
        self.get_playing_workspace_mut().queue_idx = None;
        let _ = self.get_playing_workspace_mut().set_song_idx(None);
        self.is_playing = false;

        self.mediacontrol_update_song();
    }
    /// Unpause
    pub fn play(&mut self) {
        if self.is_playing {
            let _ = self.audioplayer.play();
            self.mediacontrol_update_playback();
        }
    }
    /// Pause
    pub fn pause(&mut self) {
        let _ = self.audioplayer.pause();
        self.mediacontrol_update_playback();
    }
    /// Play previous song
    pub fn skip_back(&mut self) {
        if let Some(mut index) = self.get_playing_workspace().queue_idx {
            if index > 0 {
                index -= 1;
                self.get_playing_workspace_mut().queue_idx = Some(index);
                if let Err(e) = self.play_selected_song() {
                    self.push_error(e.to_string());
                }
            } else if self.repeat == RepeatMode::Queue {
                index = self.get_playing_workspace().queue.len() - 1;
                self.get_playing_workspace_mut().queue_idx = Some(index);
                if let Err(e) = self.play_selected_song() {
                    self.push_error(e.to_string());
                }
            }
        }
    }
    /// Play next song
    pub fn skip(&mut self) {
        if let Some(index) = self.get_playing_workspace().queue_idx {
            if index < self.get_playing_workspace().queue.len() - 1 {
                self.get_playing_workspace_mut().queue_idx = Some(index + 1);
                if let Err(e) = self.play_selected_song() {
                    self.push_error(e.to_string());
                }
            }
            if self.repeat == RepeatMode::Queue {
                self.get_playing_workspace_mut().queue_idx = Some(0);
                if let Err(e) = self.play_selected_song() {
                    self.push_error(e.to_string());
                }
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
        self.mediacontrol_update_volume();
    }
    /// Sends current volume setting to backend
    pub fn update_volume(&self) {
        // Not dividing the volume by 100 is a mistake you only make once.
        let _ = self.audioplayer.set_volume(self.volume * 0.001);
    }
    // When previous song has ended, advance queue or stop.
    fn advance_queue(&mut self) -> anyhow::Result<()> {
        let repeat = self.repeat;
        let workspace = self.get_playing_workspace_mut();

        let Some(mut queue_index) = workspace.queue_idx else {
            self.stop();
            bail!(PlayerError::NoQueueIndex)
        };

        // Replay the same song
        if repeat == RepeatMode::Song {
            workspace
                .set_song_idx(Some(workspace.queue[queue_index]))
                .expect("advance_queue: repeat song idx failed?!");
            self.play_selected_song()?;
            return Ok(());
        }

        queue_index += 1;

        // Queue end reached, back to start or bail out
        if queue_index == workspace.queue.len() {
            if repeat == RepeatMode::Queue {
                queue_index = 0;
            } else {
                let _ = workspace.set_song_idx(None);
                self.stop();
                return Ok(());
            }
        }

        // Play next song in queue
        workspace.queue_idx = Some(queue_index);
        workspace
            .set_song_idx(Some(workspace.queue[queue_index]))
            .expect("advance_queue: next song idx failed?!");
        self.play_selected_song()?;
        Ok(())
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
    /// Get a mutable reference to the workspace list
    pub fn get_workspaces_mut(&mut self) -> &mut Vec<Workspace> {
        &mut self.workspaces
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
    pub fn switch_to_workspace(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.workspaces.len() {
            bail!(PlayerError::InvalidWorkspaceIndex { index });
        }
        self.workspace_idx = index;
        self.get_workspace_mut().refresh_font_list();
        self.get_workspace_mut().refresh_song_list();
        Ok(())
    }
    pub fn switch_workspace_left(&mut self) -> anyhow::Result<()> {
        if self.workspace_idx == 0 {
            bail!(PlayerError::CantSwitchWorkspace);
        }
        self.workspace_idx -= 1;
        Ok(())
    }
    pub fn switch_workspace_right(&mut self) -> anyhow::Result<()> {
        if self.workspace_idx >= self.workspaces.len() - 1 {
            bail!(PlayerError::CantSwitchWorkspace);
        }
        self.workspace_idx += 1;
        Ok(())
    }
    /// Create a new workspace
    pub fn new_workspace(&mut self) {
        self.workspaces.push(Workspace::default());
    }
    /// Remove a workspace by index
    pub fn remove_workspace(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.workspaces.len() {
            bail!(PlayerError::InvalidWorkspaceIndex { index });
        }
        self.workspace_delet_queue.push(index);
        self.ensure_workspace_existence();
        Ok(())
    }
    /// Rearrange workspaces
    pub fn move_workspace(&mut self, old_index: usize, new_index: usize) -> anyhow::Result<()> {
        if old_index >= self.workspaces.len() {
            bail!(PlayerError::InvalidWorkspaceIndex { index: old_index });
        }
        if new_index >= self.workspaces.len() {
            bail!(PlayerError::InvalidWorkspaceIndex { index: new_index });
        }
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
        Ok(())
    }
    /// Move current workspace left
    pub fn move_workspace_left(&mut self) -> anyhow::Result<()> {
        if self.workspace_idx == 0 {
            bail!(PlayerError::CantMoveWorkspace);
        }
        self.move_workspace(self.workspace_idx, self.workspace_idx - 1)
            .expect("move_workspace_left out of bounds?!");
        Ok(())
    }
    /// Move current workspace right
    pub fn move_workspace_right(&mut self) -> anyhow::Result<()> {
        if self.workspace_idx >= self.workspaces.len() - 1 {
            bail!(PlayerError::CantMoveWorkspace);
        }
        self.move_workspace(self.workspace_idx, self.workspace_idx + 1)
            .expect("move_workspace_right out of bounds?!");
        Ok(())
    }
    /// Make sure at least one workspace exists!
    fn ensure_workspace_existence(&mut self) {
        if self.get_workspaces().is_empty() {
            self.new_workspace();
            self.workspace_idx = 0;
        }
    }

    // --- Other

    pub fn get_event_queue(&mut self) -> &mut Vec<PlayerEvent> {
        &mut self.player_events
    }
    fn push_error(&mut self, message: String) {
        self.player_events.push(PlayerEvent::NotifyError(message));
    }
}

fn workspace_dir() -> PathBuf {
    project_dirs().data_dir().join("workspaces")
}

fn state_dir() -> PathBuf {
    project_dirs()
        .state_dir()
        .map(std::borrow::ToOwned::to_owned)
        .map_or_else(
            || project_dirs().data_dir().join("fallback_state_dir"),
            |path| path,
        )
}

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("fi", "sevonj", env!("CARGO_PKG_NAME"))
        .expect("Failed to create project dirs.")
}
