//! Player app logic module

use anyhow::bail;
use audio::AudioPlayer;
use eframe::egui::mutex::Mutex;
#[cfg(not(target_os = "windows"))]
use mediacontrols::create_mediacontrols;
use playlist::{font_meta::FontMeta, DeletionStatus, Playlist};
use rodio::Sink;
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use soundfont_library::FontLibrary;
use souvlaki::{MediaControlEvent, MediaControls};
use std::{error, fmt, fs::File, io::Write, path::PathBuf, sync::Arc, time::Duration, vec};

pub mod audio;
mod mediacontrols;
pub mod playlist;
mod serialize_player;
pub mod soundfont_library;
pub mod soundfont_list;

const REMOVED_PLAYLIST_HISTORY_LEN: usize = 100;

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
    InvalidPlaylistIndex { index: usize },
    CantMovePlaylist,
    CantSwitchPlaylist,
    NoQueueIndex,
    NoSoundfont,
    PlaylistAlreadyOpen,
    PlaylistSaveFailed,
    DebugBlockSaving,
}
impl error::Error for PlayerError {}
impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlaylistIndex { index } => {
                write!(f, "Playlist index {index} is out of bounds.")
            }
            Self::CantMovePlaylist => write!(f, "Can't move this playlist further."),
            Self::CantSwitchPlaylist => write!(f, "Can't switch playlists further."),
            Self::NoQueueIndex => write!(f, "No queue index!"),
            Self::NoSoundfont => write!(f, "No soundfont!"),
            Self::PlaylistAlreadyOpen => write!(f, "Playlist is already open."),
            Self::PlaylistSaveFailed => write!(f, "Failed to save playlist."),
            Self::DebugBlockSaving => write!(f, "debug_block_saving == true"),
        }
    }
}

/// The Player class does high-level app logic, which includes playlists and playback.
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
    pub font_lib: FontLibrary,
    playlists: Vec<Playlist>,
    /// Which playlist is open
    playlist_idx: usize,
    /// Which playlist was last playing music
    playing_playlist_idx: usize,
    /// For undo closing
    removed_playlists: Vec<Playlist>,

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

            volume: 100.,
            #[cfg(not(target_os = "windows"))]
            mediacontrol,
            mediacontrol_events,
            player_events: vec![],

            font_lib: FontLibrary::default(),
            playlists: vec![],
            playlist_idx: 0,
            playing_playlist_idx: 0,
            removed_playlists: vec![],

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

    pub fn get_default_soundfont(&self) -> Option<&FontMeta> {
        self.font_lib.get_selected()
    }

    /// GUI frame update
    pub fn update(&mut self) {
        self.ensure_playlist_existence();

        if !self.is_paused() && self.is_empty() {
            if let Err(e) = self.advance_queue() {
                self.push_error(e.to_string());
            }
        }

        self.get_playlist_mut().delete_queued();
        self.font_lib.update();
        self.delete_queued_playlists();

        self.mediacontrol_handle_events();
    }

    fn delete_queued_playlists(&mut self) {
        for index in (0..self.playlists.len()).rev() {
            let playlist = &mut self.playlists[index];

            match playlist.deletion_status {
                DeletionStatus::None => continue,
                DeletionStatus::Queued => {
                    if playlist.has_unsaved_changes() {
                        continue;
                    }
                }
                DeletionStatus::QueuedDiscard => (),
            }

            if self.autosave && playlist.is_portable() {
                let _ = playlist.save_portable();
            }

            self.removed_playlists.push(self.playlists.remove(index));
            while self.removed_playlists.len() > REMOVED_PLAYLIST_HISTORY_LEN {
                self.removed_playlists.remove(0);
            }

            let last_selected = self.playlist_idx == self.playlists.len();
            // First selected: Never decrement
            // Between: Decrement if smaller
            // Last selected: Always decrement (unless last == first)
            if 0 < self.playlist_idx && (index < self.playlist_idx || last_selected) {
                self.playlist_idx -= 1;
            }
            // Doesn't really matter what we do with stale playing index, as long as it's in bounds.
            if 0 < self.playing_playlist_idx && index <= self.playing_playlist_idx {
                self.playing_playlist_idx -= 1;
            }
        }
        self.ensure_playlist_existence();
    }

    // --- Playback Control

    /// Start playing (from a fully stopped state)
    pub fn start(&mut self) {
        self.playing_playlist_idx = self.playlist_idx;
        let shuffle = self.shuffle;
        self.get_playing_playlist_mut().rebuild_queue(shuffle);
        if let Err(e) = self.play_selected_song() {
            self.push_error(e.to_string());
        }
    }

    fn get_soundfont(&mut self) -> Result<&mut FontMeta, PlayerError> {
        if let Some(font_index) = self.get_playing_playlist().get_font_idx() {
            return Ok(&mut self.get_playing_playlist_mut().get_fonts_mut()[font_index]);
        }
        self.font_lib
            .get_selected_mut()
            .ok_or(PlayerError::NoSoundfont)
    }

    /// Load currently selected song & font from playlist and start playing
    fn play_selected_song(&mut self) -> anyhow::Result<()> {
        self.audioplayer.stop_playback()?;
        let Some(queue_index) = self.get_playing_playlist().queue_idx else {
            bail!(PlayerError::NoQueueIndex);
        };
        let midi_index = self.get_playing_playlist().queue[queue_index];

        let sf = self.get_soundfont()?;
        let sf_path = sf.get_path();
        sf.refresh();
        sf.get_status()?;

        let mid = &mut self.get_playing_playlist_mut().get_songs_mut()[midi_index];
        let mid_path = mid.get_path();
        mid.refresh();
        mid.get_status()?;

        let playlist = self.get_playing_playlist_mut();
        playlist.set_song_idx(Some(midi_index))?;

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
        self.get_playing_playlist_mut().queue_idx = None;
        let _ = self.get_playing_playlist_mut().set_song_idx(None);
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
        if let Some(mut index) = self.get_playing_playlist().queue_idx {
            if index > 0 {
                index -= 1;
                self.get_playing_playlist_mut().queue_idx = Some(index);
                if let Err(e) = self.play_selected_song() {
                    self.push_error(e.to_string());
                }
            } else if self.repeat == RepeatMode::Queue {
                index = self.get_playing_playlist().queue.len() - 1;
                self.get_playing_playlist_mut().queue_idx = Some(index);
                if let Err(e) = self.play_selected_song() {
                    self.push_error(e.to_string());
                }
            }
        }
    }
    /// Play next song
    pub fn skip(&mut self) {
        if let Some(index) = self.get_playing_playlist().queue_idx {
            if index < self.get_playing_playlist().queue.len() - 1 {
                self.get_playing_playlist_mut().queue_idx = Some(index + 1);
                if let Err(e) = self.play_selected_song() {
                    self.push_error(e.to_string());
                }
            }
            if self.repeat == RepeatMode::Queue {
                self.get_playing_playlist_mut().queue_idx = Some(0);
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
        self.get_playing_playlist_mut().rebuild_queue(shuffle);
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
        let playlist = self.get_playing_playlist_mut();

        let Some(mut queue_index) = playlist.queue_idx else {
            self.stop();
            bail!(PlayerError::NoQueueIndex)
        };

        // Replay the same song
        if repeat == RepeatMode::Song {
            playlist
                .set_song_idx(Some(playlist.queue[queue_index]))
                .expect("advance_queue: repeat song idx failed?!");
            self.play_selected_song()?;
            return Ok(());
        }

        queue_index += 1;

        // Queue end reached, back to start or bail out
        if queue_index == playlist.queue.len() {
            if repeat == RepeatMode::Queue {
                queue_index = 0;
            } else {
                let _ = playlist.set_song_idx(None);
                self.stop();
                return Ok(());
            }
        }

        // Play next song in queue
        playlist.queue_idx = Some(queue_index);
        playlist
            .set_song_idx(Some(playlist.queue[queue_index]))
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

    // --- Manage Playlists

    /// Index of currently open playlist
    pub const fn get_playlist_idx(&self) -> usize {
        self.playlist_idx
    }
    /// Index of currently playing playlist
    pub const fn get_playing_playlist_idx(&self) -> usize {
        self.playing_playlist_idx
    }
    /// Get a reference to the playlist list
    pub const fn get_playlists(&self) -> &Vec<Playlist> {
        &self.playlists
    }
    /// Get a mutable reference to the playlist list
    pub fn get_playlists_mut(&mut self) -> &mut Vec<Playlist> {
        &mut self.playlists
    }
    /// Get a reference to the currently open playlist
    pub fn get_playlist(&self) -> &Playlist {
        &self.playlists[self.playlist_idx]
    }
    /// Get a mutable reference to the currently open playlist
    pub fn get_playlist_mut(&mut self) -> &mut Playlist {
        &mut self.playlists[self.playlist_idx]
    }
    /// Get a reference to the currently playing playlist.
    /// If nothing's playing, it gives the currently open playlist instead.
    pub fn get_playing_playlist(&self) -> &Playlist {
        if self.is_playing {
            return &self.playlists[self.playing_playlist_idx];
        }
        &self.playlists[self.playlist_idx]
    }
    /// Get a mutable reference to the currently playing playlist.
    /// If nothing's playing, it gives the currently open playlist instead.
    pub fn get_playing_playlist_mut(&mut self) -> &mut Playlist {
        if self.is_playing {
            return &mut self.playlists[self.playing_playlist_idx];
        }
        &mut self.playlists[self.playlist_idx]
    }
    /// Switch to another playlist
    pub fn switch_to_playlist(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.playlists.len() {
            bail!(PlayerError::InvalidPlaylistIndex { index });
        }
        self.playlist_idx = index;
        self.get_playlist_mut().refresh_font_list();
        self.get_playlist_mut().refresh_song_list();
        Ok(())
    }
    pub fn switch_playlist_left(&mut self) -> anyhow::Result<()> {
        if self.playlist_idx == 0 {
            bail!(PlayerError::CantSwitchPlaylist);
        }
        self.playlist_idx -= 1;
        Ok(())
    }
    pub fn switch_playlist_right(&mut self) -> anyhow::Result<()> {
        if self.playlist_idx >= self.playlists.len() - 1 {
            bail!(PlayerError::CantSwitchPlaylist);
        }
        self.playlist_idx += 1;
        Ok(())
    }
    /// Create a new playlist
    pub fn new_playlist(&mut self) {
        self.playlists.push(Playlist::default());
    }
    /// Remove a playlist by index
    pub fn remove_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.playlists.len() {
            return Err(PlayerError::InvalidPlaylistIndex { index });
        }
        self.playlists[index].deletion_status = DeletionStatus::Queued;
        Ok(())
    }
    /// Remove a playlist by index, override unsaved check
    pub fn force_remove_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.playlists.len() {
            return Err(PlayerError::InvalidPlaylistIndex { index });
        }
        self.playlists[index].deletion_status = DeletionStatus::QueuedDiscard;
        Ok(())
    }
    pub fn cancel_remove_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.playlists.len() {
            return Err(PlayerError::InvalidPlaylistIndex { index });
        }
        self.playlists[index].deletion_status = DeletionStatus::None;
        Ok(())
    }
    /// Get a playlist waiting for delete confirm, if any exist.
    pub fn get_playlist_waiting_for_discard(&self) -> Option<usize> {
        for (i, playlist) in self.playlists.iter().enumerate() {
            if playlist.deletion_status == DeletionStatus::Queued && playlist.has_unsaved_changes()
            {
                return Some(i);
            }
        }
        None
    }
    pub fn has_removed_playlist(&self) -> bool {
        !self.removed_playlists.is_empty()
    }
    pub fn reopen_removed_playlist(&mut self) {
        if !self.removed_playlists.is_empty() {
            let last_index = self.removed_playlists.len() - 1;
            let mut playlist = self.removed_playlists.remove(last_index);
            playlist.deletion_status = DeletionStatus::None;

            if let Some(filepath) = playlist.get_portable_path() {
                if self.is_portable_playlist_open(&filepath) {
                    self.reopen_removed_playlist();
                }
            }

            self.playlists.push(playlist);
            self.playlist_idx = self.playlists.len() - 1;
        }
    }
    /// Rearrange playlists
    pub fn move_playlist(&mut self, old_index: usize, new_index: usize) -> anyhow::Result<()> {
        if old_index >= self.playlists.len() {
            bail!(PlayerError::InvalidPlaylistIndex { index: old_index });
        }
        if new_index >= self.playlists.len() {
            bail!(PlayerError::InvalidPlaylistIndex { index: new_index });
        }
        let playlist = self.playlists.remove(old_index); // Remove at old index
        self.playlists.insert(new_index, playlist); // Reinsert at new index

        // Update current playlist index if it was affected by the move
        if old_index == self.playlist_idx {
            self.playlist_idx = new_index;
        } else if old_index < self.playlist_idx && self.playlist_idx <= new_index {
            self.playlist_idx -= 1;
        } else if new_index <= self.playlist_idx && self.playlist_idx < old_index {
            self.playlist_idx += 1;
        }
        // Update playing playlist index if it was affected by the move
        if old_index == self.playing_playlist_idx {
            self.playing_playlist_idx = new_index;
        } else if old_index < self.playing_playlist_idx && self.playing_playlist_idx <= new_index {
            self.playing_playlist_idx -= 1;
        } else if new_index <= self.playing_playlist_idx && self.playing_playlist_idx < old_index {
            self.playing_playlist_idx += 1;
        }

        Ok(())
    }
    /// Move current playlist left
    pub fn move_playlist_left(&mut self) -> anyhow::Result<()> {
        if self.playlist_idx == 0 {
            bail!(PlayerError::CantMovePlaylist);
        }
        self.move_playlist(self.playlist_idx, self.playlist_idx - 1)
            .expect("move_playlist_left out of bounds?!");
        Ok(())
    }
    /// Move current playlist right
    pub fn move_playlist_right(&mut self) -> anyhow::Result<()> {
        if self.playlist_idx >= self.playlists.len() - 1 {
            bail!(PlayerError::CantMovePlaylist);
        }
        self.move_playlist(self.playlist_idx, self.playlist_idx + 1)
            .expect("move_playlist_right out of bounds?!");
        Ok(())
    }
    pub fn open_portable_playlist(&mut self, filepath: PathBuf) -> anyhow::Result<()> {
        if self.is_portable_playlist_open(&filepath) {
            bail!("Playlist is already open")
        }
        let playlist = Playlist::open_portable(filepath)?;
        self.playlists.push(playlist);
        self.playlist_idx = self.playlists.len() - 1;
        Ok(())
    }
    pub fn save_portable_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.playlists.len() {
            return Err(PlayerError::InvalidPlaylistIndex { index });
        }
        if self.debug_block_saving {
            return Err(PlayerError::DebugBlockSaving);
        }
        if self.playlists[index].save_portable().is_err() {
            return Err(PlayerError::PlaylistSaveFailed);
        }
        Ok(())
    }
    pub fn save_all_portable_playlists(&mut self) -> Result<(), PlayerError> {
        if self.debug_block_saving {
            return Err(PlayerError::DebugBlockSaving);
        }
        for playlist in &mut self.playlists {
            if playlist.is_portable() && playlist.save_portable().is_err() {
                return Err(PlayerError::PlaylistSaveFailed);
            }
        }
        Ok(())
    }
    /// Save playlist into a portable file.
    pub fn save_playlist_as(&mut self, index: usize, filepath: PathBuf) -> Result<(), PlayerError> {
        if index >= self.playlists.len() {
            return Err(PlayerError::InvalidPlaylistIndex { index });
        }
        if self.debug_block_saving {
            return Err(PlayerError::DebugBlockSaving);
        }
        if self.is_portable_playlist_open(&filepath) {
            return Err(PlayerError::PlaylistAlreadyOpen);
        }
        let mut new_playlist = self.playlists[index].clone();
        new_playlist.set_portable_path(Some(filepath.clone()));
        new_playlist.name = filepath.file_stem().map_or_else(
            || format!("{} (Copy)", self.playlists[index].name),
            |stem| {
                stem.to_str()
                    .expect("save_playlist_as(): stem.to_str()")
                    .to_owned()
            },
        );

        let Ok(mut file) = File::create(filepath) else {
            return Err(PlayerError::PlaylistSaveFailed);
        };
        if file
            .write_all(Value::from(&new_playlist).to_string().as_bytes())
            .is_err()
        {
            return Err(PlayerError::PlaylistSaveFailed);
        };

        self.playlists.push(new_playlist);
        let _ = self.switch_to_playlist(self.playlists.len() - 1);
        Ok(())
    }
    /// New playlist is stored to app data.
    pub fn duplicate_playlist(&mut self, index: usize) -> anyhow::Result<()> {
        if index >= self.playlists.len() {
            bail!(PlayerError::InvalidPlaylistIndex { index });
        }
        let mut new_playlist = self.playlists[index].clone();
        new_playlist.set_portable_path(None);
        new_playlist.name = format!("{} (Copy)", self.playlists[index].name);

        self.playlists.push(new_playlist);
        let _ = self.switch_to_playlist(self.playlists.len() - 1);
        Ok(())
    }
    /// Make sure at least one playlist exists!
    fn ensure_playlist_existence(&mut self) {
        if self.get_playlists().is_empty() {
            self.new_playlist();
            self.playlist_idx = 0;
        }
    }
    fn is_portable_playlist_open(&self, filepath: &PathBuf) -> bool {
        for playlist in &self.playlists {
            let Some(playlist_path) = playlist.get_portable_path() else {
                continue;
            };
            if playlist_path == *filepath {
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
    fn test_rearrange_playlists_cur_wksp_index() {
        let mut player = Player::default();
        player.new_playlist();
        player.new_playlist();
        player.new_playlist();

        player.switch_to_playlist(1).unwrap();
        assert_eq!(player.playlist_idx, 1);
        player.move_playlist_left().unwrap();
        assert_eq!(player.playlist_idx, 0);
        player.move_playlist_left().unwrap_err();
        assert_eq!(player.playlist_idx, 0);
        player.move_playlist_right().unwrap();
        assert_eq!(player.playlist_idx, 1);
        player.move_playlist_right().unwrap();
        assert_eq!(player.playlist_idx, 2);
        player.move_playlist_right().unwrap_err();
        assert_eq!(player.playlist_idx, 2);

        player.move_playlist(0, 1).unwrap();
        assert_eq!(player.playlist_idx, 2);

        player.move_playlist(0, 2).unwrap();
        assert_eq!(player.playlist_idx, 1);

        player.move_playlist(1, 2).unwrap();
        assert_eq!(player.playlist_idx, 2);

        player.move_playlist(2, 2).unwrap();
        assert_eq!(player.playlist_idx, 2);

        player.move_playlist(2, 0).unwrap();
        assert_eq!(player.playlist_idx, 0);

        player.move_playlist(0, 1).unwrap();
        assert_eq!(player.playlist_idx, 1);
    }

    #[test]
    fn test_rearrange_playlists_cur_wksp_index_outofbounds() {
        let mut player = Player::default();
        player.new_playlist();
        player.new_playlist();
        player.new_playlist();

        player.switch_to_playlist(0).unwrap();
        assert_eq!(player.playlist_idx, 0);
        player.move_playlist_left().unwrap_err();
        assert_eq!(player.playlist_idx, 0);

        player.switch_to_playlist(2).unwrap();
        assert_eq!(player.playlist_idx, 2);
        player.move_playlist_right().unwrap_err();
        assert_eq!(player.playlist_idx, 2);

        player.move_playlist(0, 3).unwrap_err();
        assert_eq!(player.playlist_idx, 2);
        player.move_playlist(2, 3).unwrap_err();
        assert_eq!(player.playlist_idx, 2);
        player.move_playlist(2, usize::MAX).unwrap_err();
        assert_eq!(player.playlist_idx, 2);
    }

    #[test]
    fn test_rearrange_playlists_playing_wksp_index() {
        let mut player = Player::default();
        player.new_playlist();
        player.new_playlist();
        player.new_playlist();

        player.switch_to_playlist(1).unwrap();
        player.playing_playlist_idx = 1;
        assert_eq!(player.playing_playlist_idx, 1);
        player.move_playlist_left().unwrap();
        assert_eq!(player.playing_playlist_idx, 0);
        player.move_playlist_left().unwrap_err();
        assert_eq!(player.playing_playlist_idx, 0);
        player.move_playlist_right().unwrap();
        assert_eq!(player.playing_playlist_idx, 1);
        player.move_playlist_right().unwrap();
        assert_eq!(player.playing_playlist_idx, 2);
        player.move_playlist_right().unwrap_err();
        assert_eq!(player.playing_playlist_idx, 2);

        player.move_playlist(0, 1).unwrap();
        assert_eq!(player.playing_playlist_idx, 2);

        player.move_playlist(0, 2).unwrap();
        assert_eq!(player.playing_playlist_idx, 1);

        player.move_playlist(1, 2).unwrap();
        assert_eq!(player.playing_playlist_idx, 2);

        player.move_playlist(2, 2).unwrap();
        assert_eq!(player.playing_playlist_idx, 2);

        player.move_playlist(2, 0).unwrap();
        assert_eq!(player.playing_playlist_idx, 0);

        player.move_playlist(0, 1).unwrap();
        assert_eq!(player.playing_playlist_idx, 1);
    }

    #[test]
    fn test_rearrange_playlists_playing_wksp_index_outofbounds() {
        let mut player = Player::default();
        player.new_playlist();
        player.new_playlist();
        player.new_playlist();

        player.switch_to_playlist(0).unwrap();
        player.playing_playlist_idx = 0;
        assert_eq!(player.playing_playlist_idx, 0);
        player.move_playlist_left().unwrap_err();
        assert_eq!(player.playing_playlist_idx, 0);

        player.switch_to_playlist(2).unwrap();
        player.playing_playlist_idx = 2;
        assert_eq!(player.playing_playlist_idx, 2);
        player.move_playlist_right().unwrap_err();
        assert_eq!(player.playing_playlist_idx, 2);

        player.move_playlist(0, 3).unwrap_err();
        assert_eq!(player.playing_playlist_idx, 2);
        player.move_playlist(2, 3).unwrap_err();
        assert_eq!(player.playing_playlist_idx, 2);
        player.move_playlist(2, usize::MAX).unwrap_err();
        assert_eq!(player.playing_playlist_idx, 2);
    }

    #[test]
    fn test_remove_last_playlist_decreases_index() {
        let mut player = Player::default();
        player.new_playlist();
        player.new_playlist();
        player.new_playlist();

        player.playlist_idx = 2;
        player.remove_playlist(2).unwrap();
        player.update();
        assert_eq!(player.playlist_idx, 1)
    }

    #[test]
    fn test_remove_last_playlist_decreases_playing_index() {
        let mut player = Player::default();
        player.new_playlist();
        player.new_playlist();
        player.new_playlist();

        player.playing_playlist_idx = 2;
        player.remove_playlist(2).unwrap();
        player.update();
        assert_eq!(player.playing_playlist_idx, 1)
    }

    #[test]
    fn test_remove_nonlast_playlist_keeps_index() {
        let mut player = Player::default();
        player.new_playlist();
        player.new_playlist();
        player.new_playlist();

        player.playlist_idx = 1;
        player.remove_playlist(1).unwrap();
        player.update();
        assert_eq!(player.playlist_idx, 1)
    }

    #[test]
    fn test_debug_block_saving() {
        let mut player = Player::default();
        player.debug_block_saving = true;

        player.new_playlist();
        player.new_playlist();
        player.new_playlist();

        player.playlists[0].set_portable_path(Some("fakepath".into()));

        assert_eq!(
            player.save_portable_playlist(0).unwrap_err(),
            PlayerError::DebugBlockSaving
        );
        assert_eq!(
            player.save_all_portable_playlists().unwrap_err(),
            PlayerError::DebugBlockSaving
        );
        assert_eq!(
            player.save_playlist_as(0, "fakepath2".into()).unwrap_err(),
            PlayerError::DebugBlockSaving
        );
        assert_eq!(
            player.save_state().unwrap_err().to_string(),
            PlayerError::DebugBlockSaving.to_string()
        );
    }
}
