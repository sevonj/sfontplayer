//! Player app logic module

use anyhow::bail;
use audio::AudioPlayer;
use eframe::egui::mutex::Mutex;
#[cfg(not(target_os = "windows"))]
use mediacontrols::create_mediacontrols;
use rodio::Sink;
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use souvlaki::{MediaControlEvent, MediaControls};
use std::{error, fmt, fs::File, io::Write, path::PathBuf, sync::Arc, time::Duration, vec};
use workspace::{font_meta::FontMeta, DeletionStatus, Workspace};

pub mod audio;
mod mediacontrols;
mod serialize_player;
pub mod workspace;

const REMOVED_WORKSPACE_HISTORY_LEN: usize = 100;

/// To be handled in gui
pub enum PlayerEvent {
    /// Bring window to focus
    Raise,
    Quit,
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

#[derive(Debug, PartialEq, Eq)]
pub enum PlayerError {
    InvalidWorkspaceIndex { index: usize },
    CantMoveWorkspace,
    CantSwitchWorkspace,
    NoQueueIndex,
    NoSoundfont,
    WorkspaceAlreadyOpen,
    WorkspaceSaveFailed,
    DebugBlockSaving,
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
            Self::NoSoundfont => write!(f, "No soundfont!"),
            Self::WorkspaceAlreadyOpen => write!(f, "Workspace is already open."),
            Self::WorkspaceSaveFailed => write!(f, "Failed to save workspace."),
            Self::DebugBlockSaving => write!(f, "debug_block_saving == true"),
        }
    }
}

/// The Player class does high-level app logic, which includes workspaces and playback.
pub struct Player {
    // -- Audio
    audioplayer: AudioPlayer,
    /// Is there playback going on? Paused playback also counts.
    is_playing: bool,
    /// Used when workspace has no soundfont selected.
    default_soundfont: Option<FontMeta>,

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
    /// For undo closing
    removed_workspaces: Vec<Workspace>,

    // -- settings
    shuffle: bool,
    repeat: RepeatMode,
    pub autosave: bool,
    pub debug_block_saving: bool,
}

impl Default for Player {
    fn default() -> Self {
        let mediacontrol_events = Arc::new(Mutex::new(vec![]));
        #[cfg(not(target_os = "windows"))]
        let mediacontrol = create_mediacontrols(Arc::clone(&mediacontrol_events));

        Self {
            audioplayer: AudioPlayer::default(),
            is_playing: false,
            default_soundfont: None,

            volume: 100.,
            #[cfg(not(target_os = "windows"))]
            mediacontrol,
            mediacontrol_events,
            player_events: vec![],

            workspaces: vec![],
            workspace_idx: 0,
            playing_workspace_idx: 0,
            removed_workspaces: vec![],

            shuffle: false,
            repeat: RepeatMode::Disabled,
            autosave: true,
            debug_block_saving: false,
        }
    }
}

impl Player {
    /// You need to give the audio player a sink before it can do anything.
    pub fn set_sink(&mut self, value: Option<Sink>) {
        self.audioplayer.set_sink(value);
    }

    pub fn get_default_soundfont(&self) -> Option<FontMeta> {
        self.default_soundfont.clone()
    }
    pub fn set_default_soundfont(&mut self, value: Option<FontMeta>) {
        self.default_soundfont = value;
    }

    /// GUI frame update
    pub fn update(&mut self) {
        self.ensure_workspace_existence();

        if !self.is_paused() && self.is_empty() {
            if let Err(e) = self.advance_queue() {
                self.push_error(e.to_string());
            }
        }

        self.get_workspace_mut().delete_queued();

        self.delete_queued_workspaces();

        self.mediacontrol_handle_events();
    }

    fn delete_queued_workspaces(&mut self) {
        for index in (0..self.workspaces.len()).rev() {
            let workspace = &mut self.workspaces[index];

            match workspace.deletion_status {
                DeletionStatus::None => continue,
                DeletionStatus::Queued => {
                    if workspace.has_unsaved_changes() {
                        continue;
                    }
                }
                DeletionStatus::QueuedDiscard => (),
            }

            if self.autosave && workspace.is_portable() {
                let _ = workspace.save_portable();
            }

            self.removed_workspaces.push(self.workspaces.remove(index));
            while self.removed_workspaces.len() > REMOVED_WORKSPACE_HISTORY_LEN {
                self.removed_workspaces.remove(0);
            }

            let last_selected = self.workspace_idx == self.workspaces.len();
            // First selected: Never decrement
            // Between: Decrement if smaller
            // Last selected: Always decrement (unless last == first)
            if 0 < self.workspace_idx && (index < self.workspace_idx || last_selected) {
                self.workspace_idx -= 1;
            }
            // Doesn't really matter what we do with stale playing index, as long as it's in bounds.
            if 0 < self.playing_workspace_idx && index <= self.playing_workspace_idx {
                self.playing_workspace_idx -= 1;
            }
        }
        self.ensure_workspace_existence();
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

    fn get_soundfont(&mut self) -> anyhow::Result<&mut FontMeta> {
        if let Some(font_index) = self.get_playing_workspace().get_font_idx() {
            return Ok(&mut self.get_playing_workspace_mut().get_fonts_mut()[font_index]);
        }
        let Some(default) = &mut self.default_soundfont else {
            bail!(PlayerError::NoSoundfont);
        };
        Ok(default)
    }

    /// Load currently selected song & font from workspace and start playing
    fn play_selected_song(&mut self) -> anyhow::Result<()> {
        self.audioplayer.stop_playback()?;
        let Some(queue_index) = self.get_playing_workspace().queue_idx else {
            bail!(PlayerError::NoQueueIndex);
        };
        let midi_index = self.get_playing_workspace().queue[queue_index];

        let sf = self.get_soundfont()?;
        let sf_path = sf.get_path();
        sf.refresh();
        sf.get_status()?;

        let mid = &mut self.get_playing_workspace_mut().get_songs_mut()[midi_index];
        let mid_path = mid.get_path();
        mid.refresh();
        mid.get_status()?;

        let workspace = self.get_playing_workspace_mut();
        workspace.set_song_idx(Some(midi_index))?;

        // Play
        self.audioplayer.set_soundfont(sf_path);
        self.audioplayer.set_midifile(mid_path);
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
    pub fn remove_workspace(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.workspaces.len() {
            return Err(PlayerError::InvalidWorkspaceIndex { index });
        }
        self.workspaces[index].deletion_status = DeletionStatus::Queued;
        Ok(())
    }
    /// Remove a workspace by index, override unsaved check
    pub fn force_remove_workspace(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.workspaces.len() {
            return Err(PlayerError::InvalidWorkspaceIndex { index });
        }
        self.workspaces[index].deletion_status = DeletionStatus::QueuedDiscard;
        Ok(())
    }
    pub fn cancel_remove_workspace(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.workspaces.len() {
            return Err(PlayerError::InvalidWorkspaceIndex { index });
        }
        self.workspaces[index].deletion_status = DeletionStatus::None;
        Ok(())
    }
    /// Get a workspace waiting for delete confirm, if any exist.
    pub fn get_workspace_waiting_for_discard(&self) -> Option<usize> {
        for (i, workspace) in self.workspaces.iter().enumerate() {
            if workspace.deletion_status == DeletionStatus::Queued
                && workspace.has_unsaved_changes()
            {
                return Some(i);
            }
        }
        None
    }
    pub fn has_removed_workspaces(&self) -> bool {
        !self.removed_workspaces.is_empty()
    }
    pub fn reopen_removed_workspace(&mut self) {
        if !self.removed_workspaces.is_empty() {
            let last_index = self.removed_workspaces.len() - 1;
            let mut workspace = self.removed_workspaces.remove(last_index);
            workspace.deletion_status = DeletionStatus::None;

            if let Some(filepath) = workspace.get_portable_path() {
                if self.is_portable_workspace_open(&filepath) {
                    self.reopen_removed_workspace();
                }
            }

            self.workspaces.push(workspace);
            self.workspace_idx = self.workspaces.len() - 1;
        }
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
        } else if old_index < self.workspace_idx && self.workspace_idx <= new_index {
            self.workspace_idx -= 1;
        } else if new_index <= self.workspace_idx && self.workspace_idx < old_index {
            self.workspace_idx += 1;
        }
        // Update playing workspace index if it was affected by the move
        if old_index == self.playing_workspace_idx {
            self.playing_workspace_idx = new_index;
        } else if old_index < self.playing_workspace_idx && self.playing_workspace_idx <= new_index
        {
            self.playing_workspace_idx -= 1;
        } else if new_index <= self.playing_workspace_idx && self.playing_workspace_idx < old_index
        {
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
    pub fn open_portable_workspace(&mut self, filepath: PathBuf) -> anyhow::Result<()> {
        if self.is_portable_workspace_open(&filepath) {
            bail!("Workspace is already open")
        }
        let workspace = Workspace::open_portable(filepath)?;
        self.workspaces.push(workspace);
        self.workspace_idx = self.workspaces.len() - 1;
        Ok(())
    }
    pub fn save_portable_workspace(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.workspaces.len() {
            return Err(PlayerError::InvalidWorkspaceIndex { index });
        }
        if self.debug_block_saving {
            return Err(PlayerError::DebugBlockSaving);
        }
        if self.workspaces[index].save_portable().is_err() {
            return Err(PlayerError::WorkspaceSaveFailed);
        }
        Ok(())
    }
    pub fn save_all_portable_workspaces(&mut self) -> Result<(), PlayerError> {
        if self.debug_block_saving {
            return Err(PlayerError::DebugBlockSaving);
        }
        for workspace in &mut self.workspaces {
            if workspace.is_portable() && workspace.save_portable().is_err() {
                return Err(PlayerError::WorkspaceSaveFailed);
            }
        }
        Ok(())
    }
    /// Save workspace into a portable file.
    pub fn save_workspace_as(
        &mut self,
        index: usize,
        filepath: PathBuf,
    ) -> Result<(), PlayerError> {
        if index >= self.workspaces.len() {
            return Err(PlayerError::InvalidWorkspaceIndex { index });
        }
        if self.debug_block_saving {
            return Err(PlayerError::DebugBlockSaving);
        }
        if self.is_portable_workspace_open(&filepath) {
            return Err(PlayerError::WorkspaceAlreadyOpen);
        }
        let mut new_workspace = self.workspaces[index].clone();
        new_workspace.set_portable_path(Some(filepath.clone()));
        new_workspace.name = filepath.file_stem().map_or_else(
            || format!("{} (Copy)", self.workspaces[index].name),
            |stem| {
                stem.to_str()
                    .expect("save_workspace_as(): stem.to_str()")
                    .to_owned()
            },
        );

        let Ok(mut file) = File::create(filepath) else {
            return Err(PlayerError::WorkspaceSaveFailed);
        };
        if file
            .write_all(Value::from(&new_workspace).to_string().as_bytes())
            .is_err()
        {
            return Err(PlayerError::WorkspaceSaveFailed);
        };

        self.workspaces.push(new_workspace);
        let _ = self.switch_to_workspace(self.workspaces.len() - 1);
        Ok(())
    }
    /// New workspace is stored to app data.
    pub fn duplicate_workspace(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.workspaces.len() {
            bail!(PlayerError::InvalidWorkspaceIndex { index });
        }
        let mut new_workspace = self.workspaces[index].clone();
        new_workspace.set_portable_path(None);
        new_workspace.name = format!("{} (Copy)", self.workspaces[index].name);

        self.workspaces.push(new_workspace);
        let _ = self.switch_to_workspace(self.workspaces.len() - 1);
        Ok(())
    }
    /// Make sure at least one workspace exists!
    fn ensure_workspace_existence(&mut self) {
        if self.get_workspaces().is_empty() {
            self.new_workspace();
            self.workspace_idx = 0;
        }
    }
    fn is_portable_workspace_open(&self, filepath: &PathBuf) -> bool {
        for workspace in &self.workspaces {
            let Some(workspace_path) = workspace.get_portable_path() else {
                continue;
            };
            if workspace_path == *filepath {
                return true;
            }
        }
        false
    }

    // --- Other

    pub fn get_event_queue(&mut self) -> &mut Vec<PlayerEvent> {
        &mut self.player_events
    }
    fn push_error(&mut self, message: String) {
        self.player_events.push(PlayerEvent::NotifyError(message));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_rearrange_workspaces_cur_wksp_index() {
        let mut player = Player::default();
        player.new_workspace();
        player.new_workspace();
        player.new_workspace();

        player.switch_to_workspace(1).unwrap();
        assert_eq!(player.workspace_idx, 1);
        player.move_workspace_left().unwrap();
        assert_eq!(player.workspace_idx, 0);
        player.move_workspace_left().unwrap_err();
        assert_eq!(player.workspace_idx, 0);
        player.move_workspace_right().unwrap();
        assert_eq!(player.workspace_idx, 1);
        player.move_workspace_right().unwrap();
        assert_eq!(player.workspace_idx, 2);
        player.move_workspace_right().unwrap_err();
        assert_eq!(player.workspace_idx, 2);

        player.move_workspace(0, 1).unwrap();
        assert_eq!(player.workspace_idx, 2);

        player.move_workspace(0, 2).unwrap();
        assert_eq!(player.workspace_idx, 1);

        player.move_workspace(1, 2).unwrap();
        assert_eq!(player.workspace_idx, 2);

        player.move_workspace(2, 2).unwrap();
        assert_eq!(player.workspace_idx, 2);

        player.move_workspace(2, 0).unwrap();
        assert_eq!(player.workspace_idx, 0);

        player.move_workspace(0, 1).unwrap();
        assert_eq!(player.workspace_idx, 1);
    }

    #[test]
    fn test_rearrange_workspaces_cur_wksp_index_outofbounds() {
        let mut player = Player::default();
        player.new_workspace();
        player.new_workspace();
        player.new_workspace();

        player.switch_to_workspace(0).unwrap();
        assert_eq!(player.workspace_idx, 0);
        player.move_workspace_left().unwrap_err();
        assert_eq!(player.workspace_idx, 0);

        player.switch_to_workspace(2).unwrap();
        assert_eq!(player.workspace_idx, 2);
        player.move_workspace_right().unwrap_err();
        assert_eq!(player.workspace_idx, 2);

        player.move_workspace(0, 3).unwrap_err();
        assert_eq!(player.workspace_idx, 2);
        player.move_workspace(2, 3).unwrap_err();
        assert_eq!(player.workspace_idx, 2);
        player.move_workspace(2, usize::MAX).unwrap_err();
        assert_eq!(player.workspace_idx, 2);
    }

    #[test]
    fn test_rearrange_workspaces_playing_wksp_index() {
        let mut player = Player::default();
        player.new_workspace();
        player.new_workspace();
        player.new_workspace();

        player.switch_to_workspace(1).unwrap();
        player.playing_workspace_idx = 1;
        assert_eq!(player.playing_workspace_idx, 1);
        player.move_workspace_left().unwrap();
        assert_eq!(player.playing_workspace_idx, 0);
        player.move_workspace_left().unwrap_err();
        assert_eq!(player.playing_workspace_idx, 0);
        player.move_workspace_right().unwrap();
        assert_eq!(player.playing_workspace_idx, 1);
        player.move_workspace_right().unwrap();
        assert_eq!(player.playing_workspace_idx, 2);
        player.move_workspace_right().unwrap_err();
        assert_eq!(player.playing_workspace_idx, 2);

        player.move_workspace(0, 1).unwrap();
        assert_eq!(player.playing_workspace_idx, 2);

        player.move_workspace(0, 2).unwrap();
        assert_eq!(player.playing_workspace_idx, 1);

        player.move_workspace(1, 2).unwrap();
        assert_eq!(player.playing_workspace_idx, 2);

        player.move_workspace(2, 2).unwrap();
        assert_eq!(player.playing_workspace_idx, 2);

        player.move_workspace(2, 0).unwrap();
        assert_eq!(player.playing_workspace_idx, 0);

        player.move_workspace(0, 1).unwrap();
        assert_eq!(player.playing_workspace_idx, 1);
    }

    #[test]
    fn test_rearrange_workspaces_playing_wksp_index_outofbounds() {
        let mut player = Player::default();
        player.new_workspace();
        player.new_workspace();
        player.new_workspace();

        player.switch_to_workspace(0).unwrap();
        player.playing_workspace_idx = 0;
        assert_eq!(player.playing_workspace_idx, 0);
        player.move_workspace_left().unwrap_err();
        assert_eq!(player.playing_workspace_idx, 0);

        player.switch_to_workspace(2).unwrap();
        player.playing_workspace_idx = 2;
        assert_eq!(player.playing_workspace_idx, 2);
        player.move_workspace_right().unwrap_err();
        assert_eq!(player.playing_workspace_idx, 2);

        player.move_workspace(0, 3).unwrap_err();
        assert_eq!(player.playing_workspace_idx, 2);
        player.move_workspace(2, 3).unwrap_err();
        assert_eq!(player.playing_workspace_idx, 2);
        player.move_workspace(2, usize::MAX).unwrap_err();
        assert_eq!(player.playing_workspace_idx, 2);
    }

    #[test]
    fn test_remove_last_workspace_decreases_index() {
        let mut player = Player::default();
        player.new_workspace();
        player.new_workspace();
        player.new_workspace();

        player.workspace_idx = 2;
        player.remove_workspace(2).unwrap();
        player.update();
        assert_eq!(player.workspace_idx, 1)
    }

    #[test]
    fn test_remove_last_workspace_decreases_playing_index() {
        let mut player = Player::default();
        player.new_workspace();
        player.new_workspace();
        player.new_workspace();

        player.playing_workspace_idx = 2;
        player.remove_workspace(2).unwrap();
        player.update();
        assert_eq!(player.playing_workspace_idx, 1)
    }

    #[test]
    fn test_remove_nonlast_workspace_keeps_index() {
        let mut player = Player::default();
        player.new_workspace();
        player.new_workspace();
        player.new_workspace();

        player.workspace_idx = 1;
        player.remove_workspace(1).unwrap();
        player.update();
        assert_eq!(player.workspace_idx, 1)
    }

    #[test]
    fn test_debug_block_saving() {
        let mut player = Player::default();
        player.debug_block_saving = true;

        player.new_workspace();
        player.new_workspace();
        player.new_workspace();

        player.workspaces[0].set_portable_path(Some("fakepath".into()));

        assert_eq!(
            player.save_portable_workspace(0).unwrap_err(),
            PlayerError::DebugBlockSaving
        );
        assert_eq!(
            player.save_all_portable_workspaces().unwrap_err(),
            PlayerError::DebugBlockSaving
        );
        assert_eq!(
            player.save_workspace_as(0, "fakepath2".into()).unwrap_err(),
            PlayerError::DebugBlockSaving
        );
        assert_eq!(
            player.save_state().unwrap_err().to_string(),
            PlayerError::DebugBlockSaving.to_string()
        );
    }
}
