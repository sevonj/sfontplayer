//! Player app logic module
//!

mod audio;
mod enums;
mod error;
mod mediacontrols;
mod midi_inspector;
pub mod playlist;
pub mod serialize_player;
mod soundfont_library;
mod soundfont_list;

use eframe::egui::mutex::Mutex;
use midi_msg::MidiFile;
use rodio::Sink;
use serde_json::Value;
use souvlaki::{MediaControlEvent, MediaControls};
use std::{fs::File, io::Write, path::PathBuf, sync::Arc, time::Duration, vec};

use audio::AudioPlayer;
pub use enums::{PlayerEvent, RepeatMode};
pub use error::PlayerError;
#[cfg(not(target_os = "windows"))]
use mediacontrols::create_mediacontrols;
pub use midi_inspector::{MidiInspector, MidiInspectorTrack, PresetMapper};
use playlist::{FontMeta, MidiMeta, Playlist, PlaylistState};
pub use soundfont_library::FontLibrary;
pub use soundfont_list::FontSort;

const REMOVED_PLAYLIST_HISTORY_LEN: usize = 100;

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
    /// Used by inspector.
    midi_inspector: Option<MidiInspector>,

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
            midi_inspector: None,

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

        self.get_playlist_mut().remove_marked();
        self.font_lib.update();
        self.delete_queued_playlists();

        self.mediacontrol_handle_events();
    }

    fn delete_queued_playlists(&mut self) {
        for index in (0..self.playlists.len()).rev() {
            let playlist = &mut self.playlists[index];

            match playlist.state {
                PlaylistState::None => continue,
                PlaylistState::Queued => {
                    if playlist.has_unsaved_changes() {
                        continue;
                    }
                }
                PlaylistState::QueuedDiscard => (),
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
            println!("{e}");
            self.push_error(e.to_string());
        }
    }

    /// Load currently selected song & font from playlist and start playing
    fn play_selected_song(&mut self) -> Result<(), PlayerError> {
        self.audioplayer.stop_playback()?;
        let Some(queue_index) = self.get_playing_playlist().queue_idx else {
            return Err(PlayerError::PlaybackNoQueueIndex);
        };

        self.reload_font()?;

        let midi_index = self.get_playing_playlist().queue[queue_index];

        let midi_file = if let Some(inspector) = &self.midi_inspector {
            inspector.midifile()?
        } else {
            let midi_meta = &mut self.get_playing_playlist_mut().get_songs_mut()[midi_index];
            midi_meta.refresh();
            midi_meta.status()?;
            midi_meta.fetch_midifile()?
        };

        let playlist = self.get_playing_playlist_mut();
        playlist.set_song_idx(Some(midi_index))?;

        // Play
        self.audioplayer.set_midifile(midi_file);
        self.is_playing = true;

        self.update_volume();
        self.audioplayer.start_playback()?;

        self.mediacontrol_update_song();

        Ok(())
    }

    /// Override playlists and play a specific `MidiFile`. Used for test sounds.
    pub fn play_midi(&mut self, midi_file: MidiFile) -> Result<(), PlayerError> {
        self.audioplayer.stop_playback()?;
        self.reload_font()?;

        self.audioplayer.set_midifile(midi_file);
        self.is_playing = true;

        self.update_volume();
        self.audioplayer.start_playback()?;

        self.mediacontrol_update_song();
        Ok(())
    }

    pub fn get_soundfont_meta(&self) -> Option<&FontMeta> {
        let font_meta = match self.get_playlist().get_selected_font() {
            Some(font) => font,
            None => self.get_default_soundfont()?,
        };
        Some(font_meta)
    }

    /// Finds out which font is selected and loads it.
    pub fn reload_font(&mut self) -> Result<(), PlayerError> {
        if let Some(inspector) = &mut self.midi_inspector {
            inspector.set_soundfont(None);
        }
        self.audioplayer.clear_soundfont();

        let mut font_meta = self.get_playing_playlist_mut().get_selected_font_mut();
        if font_meta.is_none() {
            font_meta = self.font_lib.get_selected_mut();
        }
        let Some(font_meta) = font_meta else {
            return Err(PlayerError::PlaybackNoSoundfont);
        };
        font_meta.refresh();
        font_meta.status()?;
        let soundfont = Arc::new(font_meta.fetch_soundfont()?);

        if let Some(inspector) = &mut self.midi_inspector {
            inspector.set_soundfont(Some(soundfont.clone()));
        }
        self.audioplayer.set_soundfont(soundfont);

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

    pub fn seek_to(&mut self, t: Duration) {
        if let Err(e) = self.audioplayer.seek_to(t) {
            self.push_error(e.to_string());
        }
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
    pub fn skip_back(&mut self) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            return Err(PlayerError::MidiOverride);
        }
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
        Ok(())
    }

    /// Play next song
    pub fn skip(&mut self) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            return Err(PlayerError::MidiOverride);
        }
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
        Ok(())
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

    pub const fn cycle_repeat(&mut self) {
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
        self.update_volume();
        self.mediacontrol_update_volume();
    }

    /// Sends current volume setting to backend
    pub fn update_volume(&self) {
        // Not dividing the volume by 100 is a mistake you only make once.
        let _ = self.audioplayer.set_volume(self.volume * 0.01);
    }

    pub fn get_inspected_midi_meta(&self) -> Option<&MidiMeta> {
        self.midi_inspector.as_ref().map(MidiInspector::midimeta)
    }

    pub const fn get_midi_inspector(&self) -> Option<&MidiInspector> {
        self.midi_inspector.as_ref()
    }

    pub const fn get_midi_inspector_mut(&mut self) -> Option<&mut MidiInspector> {
        self.midi_inspector.as_mut()
    }

    pub fn open_midi_inspector(&mut self, meta: MidiMeta) -> Result<(), PlayerError> {
        let soundfont = self
            .get_soundfont_meta()
            .and_then(|f| f.fetch_soundfont().map(Arc::new).ok());
        let inspector = MidiInspector::new(meta, soundfont)?;
        self.midi_inspector = Some(inspector);
        self.stop();
        Ok(())
    }

    pub fn close_midi_inspector(&mut self) {
        self.midi_inspector = None;
        self.stop();
    }

    // When previous song has ended, advance queue or stop.
    fn advance_queue(&mut self) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            self.pause();
            return Ok(());
        }
        let repeat = self.repeat;
        let playlist = self.get_playing_playlist_mut();

        let Some(mut queue_index) = playlist.queue_idx else {
            self.stop();
            return Err(PlayerError::PlaybackNoQueueIndex);
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
    pub const fn get_playlists_mut(&mut self) -> &mut Vec<Playlist> {
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
    pub fn switch_to_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            return Err(PlayerError::MidiOverride);
        }
        if index >= self.playlists.len() {
            return Err(PlayerError::PlaylistIndex { index });
        }
        self.playlist_idx = index;
        self.get_playlist_mut().recrawl_fonts();
        self.get_playlist_mut().refresh_song_list();
        Ok(())
    }

    pub const fn switch_playlist_left(&mut self) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            return Err(PlayerError::MidiOverride);
        }
        if self.playlist_idx == 0 {
            return Err(PlayerError::PlaylistCantSwitch);
        }
        self.playlist_idx -= 1;
        Ok(())
    }

    pub fn switch_playlist_right(&mut self) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            return Err(PlayerError::MidiOverride);
        }
        if self.playlist_idx >= self.playlists.len() - 1 {
            return Err(PlayerError::PlaylistCantSwitch);
        }
        self.playlist_idx += 1;
        Ok(())
    }

    /// Create a new playlist
    pub fn new_playlist(&mut self) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            return Err(PlayerError::MidiOverride);
        }
        self.playlists.push(Playlist::default());
        Ok(())
    }

    /// Remove a playlist by index
    pub fn remove_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            return Err(PlayerError::MidiOverride);
        }
        if index >= self.playlists.len() {
            return Err(PlayerError::PlaylistIndex { index });
        }
        self.playlists[index].state = PlaylistState::Queued;
        Ok(())
    }

    /// Remove a playlist by index, override unsaved check
    pub fn force_remove_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.playlists.len() {
            return Err(PlayerError::PlaylistIndex { index });
        }
        self.playlists[index].state = PlaylistState::QueuedDiscard;
        Ok(())
    }

    pub fn cancel_remove_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.playlists.len() {
            return Err(PlayerError::PlaylistIndex { index });
        }
        self.playlists[index].state = PlaylistState::None;
        Ok(())
    }

    /// Get a playlist waiting for delete confirm, if any exist.
    pub fn get_playlist_waiting_for_discard(&self) -> Option<usize> {
        for (i, playlist) in self.playlists.iter().enumerate() {
            if playlist.state == PlaylistState::Queued && playlist.has_unsaved_changes() {
                return Some(i);
            }
        }
        None
    }

    /// Playlist tabs have been closed
    pub fn has_removed_playlist(&self) -> bool {
        !self.removed_playlists.is_empty()
    }

    /// Undo playlist close, reopen the tab.
    pub fn reopen_removed_playlist(&mut self) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            return Err(PlayerError::MidiOverride);
        }
        if self.removed_playlists.is_empty() {
            return Ok(());
        }
        let last_index = self.removed_playlists.len() - 1;
        let mut playlist = self.removed_playlists.remove(last_index);
        playlist.state = PlaylistState::None;

        // Skip this one, it's been reopened since closing.
        if let Some(filepath) = playlist.get_portable_path() {
            if self.is_portable_playlist_open(&filepath) {
                return self.reopen_removed_playlist();
            }
        }

        self.playlists.push(playlist);
        self.playlist_idx = self.playlists.len() - 1;

        Ok(())
    }

    /// Rearrange playlists
    pub fn move_playlist(&mut self, old_index: usize, new_index: usize) -> Result<(), PlayerError> {
        if old_index >= self.playlists.len() {
            return Err(PlayerError::PlaylistIndex { index: old_index });
        }
        if new_index >= self.playlists.len() {
            return Err(PlayerError::PlaylistIndex { index: new_index });
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
    pub fn move_playlist_left(&mut self) -> Result<(), PlayerError> {
        if self.playlist_idx == 0 {
            return Err(PlayerError::PlaylistCantMove);
        }
        self.move_playlist(self.playlist_idx, self.playlist_idx - 1)
            .expect("move_playlist_left out of bounds?!");
        Ok(())
    }

    /// Move current playlist right
    pub fn move_playlist_right(&mut self) -> Result<(), PlayerError> {
        if self.playlist_idx >= self.playlists.len() - 1 {
            return Err(PlayerError::PlaylistCantMove);
        }
        self.move_playlist(self.playlist_idx, self.playlist_idx + 1)
            .expect("move_playlist_right out of bounds?!");
        Ok(())
    }

    pub fn open_portable_playlist(&mut self, filepath: PathBuf) -> Result<(), PlayerError> {
        if self.is_portable_playlist_open(&filepath) {
            return Err(PlayerError::PlaylistAlreadyOpen);
        }
        let playlist = Playlist::open_portable(filepath)?;
        self.playlists.push(playlist);
        self.playlist_idx = self.playlists.len() - 1;
        Ok(())
    }

    pub fn save_portable_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if index >= self.playlists.len() {
            return Err(PlayerError::PlaylistIndex { index });
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
            return Err(PlayerError::PlaylistIndex { index });
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
        }

        self.playlists.push(new_playlist);
        let _ = self.switch_to_playlist(self.playlists.len() - 1);
        Ok(())
    }

    /// New playlist is stored to app data.
    pub fn duplicate_playlist(&mut self, index: usize) -> Result<(), PlayerError> {
        if self.midi_inspector.is_some() {
            return Err(PlayerError::MidiOverride);
        }
        if index >= self.playlists.len() {
            return Err(PlayerError::PlaylistIndex { index });
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
            let _ = self.new_playlist();
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

    pub const fn get_event_queue(&mut self) -> &mut Vec<PlayerEvent> {
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
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();

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
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();

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
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();

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
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();

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
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();

        player.playlist_idx = 2;
        player.remove_playlist(2).unwrap();
        player.update();
        assert_eq!(player.playlist_idx, 1);
    }

    #[test]
    fn test_remove_last_playlist_decreases_playing_index() {
        let mut player = Player::default();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();

        player.playing_playlist_idx = 2;
        player.remove_playlist(2).unwrap();
        player.update();
        assert_eq!(player.playing_playlist_idx, 1);
    }

    #[test]
    fn test_remove_nonlast_playlist_keeps_index() {
        let mut player = Player::default();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();

        player.playlist_idx = 1;
        player.remove_playlist(1).unwrap();
        player.update();
        assert_eq!(player.playlist_idx, 1);
    }

    #[test]
    fn test_debug_block_saving() {
        let mut player = Player::default();
        player.debug_block_saving = true;

        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();

        player.playlists[0].set_portable_path(Some("fakepath".into()));

        assert!(matches!(
            player.save_portable_playlist(0).unwrap_err(),
            PlayerError::DebugBlockSaving
        ));
        assert!(matches!(
            player.save_all_portable_playlists().unwrap_err(),
            PlayerError::DebugBlockSaving
        ));
        assert!(matches!(
            player.save_playlist_as(0, "fakepath2".into()).unwrap_err(),
            PlayerError::DebugBlockSaving
        ));
        // Weird because anyhow.
        // To be cleaned.
        assert_eq!(
            player.save_state().unwrap_err().to_string(),
            PlayerError::DebugBlockSaving.to_string()
        );
    }

    #[test]
    fn test_midioverride_cant_skip() {
        let mut player = Player::default();
        player.new_playlist().unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath1".into())
            .unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath2".into())
            .unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath3".into())
            .unwrap();
        player.get_playlist_mut().set_song_idx(Some(1)).unwrap();
        assert_eq!(player.get_playlist().get_song_idx(), Some(1));
        player
            .open_midi_inspector(MidiMeta::new("src/assets/demo.mid".into()))
            .unwrap();

        assert!(matches!(
            player.skip().unwrap_err(),
            PlayerError::MidiOverride
        ));
        assert_eq!(player.get_playlist().get_song_idx(), None);

        assert!(matches!(
            player.skip_back().unwrap_err(),
            PlayerError::MidiOverride
        ));
        assert_eq!(player.get_playlist().get_song_idx(), None);
    }

    #[test]
    fn test_midioverride_advance() {
        // Advance playback should not advance playback if override.

        let mut player = Player::default();
        player.new_playlist().unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath1".into())
            .unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath2".into())
            .unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath3".into())
            .unwrap();
        player.get_playlist_mut().set_song_idx(Some(1)).unwrap();
        assert_eq!(player.get_playlist().get_song_idx(), Some(1));
        player
            .open_midi_inspector(MidiMeta::new("src/assets/demo.mid".into()))
            .unwrap();
        assert_eq!(player.get_playlist().get_song_idx(), None);

        player.playing_playlist_idx = player.playlist_idx;
        player.get_playing_playlist_mut().rebuild_queue(false);
        player.is_playing = true;

        assert_eq!(player.get_playlist().queue_idx, Some(0));
        player.advance_queue().unwrap();
        assert_eq!(player.get_playlist().queue_idx, Some(0));
    }

    #[test]
    fn test_midioverride_cant_change_playlist() {
        let mut player = Player::default();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.switch_to_playlist(1).unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath1".into())
            .unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath2".into())
            .unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath3".into())
            .unwrap();
        assert_eq!(player.get_playlist_idx(), 1);
        player
            .open_midi_inspector(MidiMeta::new("src/assets/demo.mid".into()))
            .unwrap();

        assert!(matches!(
            player.switch_playlist_left().unwrap_err(),
            PlayerError::MidiOverride
        ));
        assert_eq!(player.get_playlist_idx(), 1);

        assert!(matches!(
            player.switch_playlist_right().unwrap_err(),
            PlayerError::MidiOverride
        ));
        assert_eq!(player.get_playlist_idx(), 1);

        assert!(matches!(
            player.switch_to_playlist(0).unwrap_err(),
            PlayerError::MidiOverride
        ));
        assert_eq!(player.get_playlist_idx(), 1);
    }

    #[test]
    fn test_midioverride_manage_playlists() {
        let mut player = Player::default();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.new_playlist().unwrap();
        player.switch_to_playlist(1).unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath1".into())
            .unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath2".into())
            .unwrap();
        player
            .get_playlist_mut()
            .add_song("fakepath3".into())
            .unwrap();
        assert_eq!(player.get_playlist_idx(), 1);
        player
            .open_midi_inspector(MidiMeta::new("src/assets/demo.mid".into()))
            .unwrap();

        assert!(matches!(
            player.new_playlist().unwrap_err(),
            PlayerError::MidiOverride
        ));
        assert_eq!(player.get_playlists().len(), 3);

        assert!(matches!(
            player.remove_playlist(1).unwrap_err(),
            PlayerError::MidiOverride
        ));
        assert_eq!(player.get_playlists().len(), 3);

        assert!(matches!(
            player.reopen_removed_playlist().unwrap_err(),
            PlayerError::MidiOverride
        ));
        assert_eq!(player.get_playlists().len(), 3);

        assert!(matches!(
            player.duplicate_playlist(1).unwrap_err(),
            PlayerError::MidiOverride
        ));
        assert_eq!(player.get_playlists().len(), 3);
    }
}
