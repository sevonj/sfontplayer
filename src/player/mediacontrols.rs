//! OS integration for media controls and metadata
//!
//! TODO: Make this work on Windows.
//! <https://github.com/sevonj/sfontplayer/issues/82>

use std::sync::Arc;

use eframe::egui::mutex::Mutex;
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};

use super::{Player, PlayerEvent};

#[cfg(not(target_os = "windows"))]
pub(super) fn create_mediacontrols(
    event_queue: Arc<Mutex<Vec<MediaControlEvent>>>,
) -> MediaControls {
    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let hwnd = todo!();

    let config = PlatformConfig {
        dbus_name: env!("CARGO_PKG_NAME"),
        display_name: "SfontPlayer",
        hwnd,
    };

    let mut controls = MediaControls::new(config).expect("Failed to create MediaControls!");
    // controls
    //     .attach(|event: MediaControlEvent| println!("Event received: {event:?}"))
    //     .expect("MediaControls Attach failed.");
    controls
        .attach(move |event: MediaControlEvent| {
            event_queue.lock().push(event);
        })
        .expect("MediaControls Attach failed.");
    controls
}

impl Player {
    pub(super) fn mediacontrol_update_song(&mut self) {
        #[cfg(not(target_os = "windows"))]
        {
            let Some(midi_index) = self.get_playlist().get_song_idx() else {
                // Clear song
                let _ = self.mediacontrol.set_metadata(MediaMetadata::default());
                return;
            };
            let midi = &self.get_playlist().get_songs()[midi_index];

            let filename = midi.filename();
            let _ = self.mediacontrol.set_metadata(MediaMetadata {
                title: Some(&filename),
                // Give an empty name to hide "Unknown Artist"
                artist: Some(""),
                duration: midi.duration(),
                ..MediaMetadata::default()
            });

            self.mediacontrol_update_playback();
        }
    }

    pub(super) fn mediacontrol_update_playback(&mut self) {
        #[cfg(not(target_os = "windows"))]
        {
            let playback = if self.is_empty() {
                MediaPlayback::Stopped
            } else if self.is_paused() {
                MediaPlayback::Paused {
                    progress: Some(self.get_media_position()),
                }
            } else {
                MediaPlayback::Playing {
                    progress: Some(self.get_media_position()),
                }
            };

            let _ = self.mediacontrol.set_playback(playback);
        }
    }

    pub(super) fn mediacontrol_update_volume(&mut self) {
        #[cfg(target_os = "linux")]
        let _ = self.mediacontrol.set_volume(f64::from(self.volume) / 100.0);
    }

    fn get_media_position(&self) -> MediaPosition {
        MediaPosition(self.get_playback_position())
    }

    pub(super) fn mediacontrol_handle_events(&mut self) {
        #[cfg(not(target_os = "windows"))]
        {
            let mut event_queue = self.mediacontrol_events.lock().clone();
            self.mediacontrol_events.lock().clear();

            while !event_queue.is_empty() {
                match event_queue.remove(0) {
                    MediaControlEvent::Play => self.play(),
                    MediaControlEvent::Pause => self.pause(),
                    MediaControlEvent::Toggle => {
                        if self.is_paused() {
                            self.play();
                        } else {
                            self.pause();
                        }
                    }
                    MediaControlEvent::Next => {
                        let _ = self.skip();
                    }
                    MediaControlEvent::Previous => {
                        let _ = self.skip_back();
                    }
                    MediaControlEvent::Stop => self.stop(),
                    MediaControlEvent::SetVolume(vol) => self.set_volume(vol as f32 * 100.0),

                    MediaControlEvent::Seek(_)
                    | MediaControlEvent::SeekBy(_, _)
                    | MediaControlEvent::SetPosition(_) => self.push_error("Todo".into()),

                    MediaControlEvent::Raise => self.player_events.push(PlayerEvent::Raise),
                    MediaControlEvent::Quit => self.player_events.push(PlayerEvent::Quit),

                    MediaControlEvent::OpenUri(_) => {
                        self.push_error("SfontPlayer doesn't support opening URIs.".into());
                    }
                }
            }
        }
    }
}
