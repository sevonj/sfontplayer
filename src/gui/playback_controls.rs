use eframe::egui::{
    include_image, Button, Image, ImageSource, Response, RichText, SelectableLabel, Sense, Slider,
    Ui, UiBuilder,
};

use crate::{
    player::{Player, RepeatMode},
    GuiState,
};

const ICON_SIZE: f32 = 20.;

use super::conversions::format_duration;
pub fn playback_panel(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.horizontal(|ui| {
        playback_controls(ui, player, gui);

        let slider_width = f32::max(ui.available_width() - 144., 64.);
        position_control(ui, player, slider_width);

        volume_control(ui, player);
    });
}

fn playback_controls(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    let (back_enabled, skip_enabled) = if player.get_playing_playlist().queue.is_empty() {
        (false, false)
    } else if player.get_repeat() == RepeatMode::Queue && player.is_playing() {
        (true, true)
    } else if let Some(idx) = player.get_playing_playlist().queue_idx {
        (idx > 0, idx < player.get_playing_playlist().queue.len() - 1)
    } else {
        (false, false)
    };

    // Current song info
    let current_hover_text = format!(
        "Currently {}: {}",
        if player.is_empty() {
            "selected"
        } else {
            "playing"
        },
        player.get_playing_playlist().get_song_idx().map_or_else(
            || "Nothing".into(),
            |index| player.get_playing_playlist().get_songs()[index].get_name()
        )
    );
    if ui
        .add_enabled(
            player.get_playing_playlist().get_song_idx().is_some(),
            Button::new(RichText::new("🎵").size(ICON_SIZE)).frame(false),
        )
        .on_hover_text(current_hover_text)
        .clicked()
    {
        let _ = player.switch_to_playlist(player.get_playing_playlist_idx());
        gui.update_flags.scroll_to_song = true;
    }

    // Shuffle button
    if ui
        .add(SelectableLabel::new(
            player.get_shuffle(),
            RichText::new("🔀").size(ICON_SIZE),
        ))
        .clicked()
    {
        player.toggle_shuffle();
    };
    // Repeat
    let repeat_text = if player.get_repeat() == RepeatMode::Song {
        "🔂"
    } else {
        "🔁"
    };
    if ui
        .add(SelectableLabel::new(
            player.get_repeat() != RepeatMode::Disabled,
            RichText::new(repeat_text).size(ICON_SIZE),
        ))
        .clicked()
    {
        player.cycle_repeat();
    };

    // Skip back
    ui.add_enabled_ui(back_enabled, |ui| {
        if icon_button(ui, include_image!("../assets/icon_prev.svg"), "back").clicked() {
            player.skip_back();
        }
    });
    // Playpause
    if player.is_paused() {
        if icon_button(ui, include_image!("../assets/icon_play.svg"), "play").clicked() {
            if player.is_empty() {
                player.start();
            } else {
                player.play();
            }
        };
    } else if icon_button(ui, include_image!("../assets/icon_pause.svg"), "pause").clicked() {
        player.pause();
    }
    // Skip
    ui.add_enabled_ui(skip_enabled, |ui| {
        if icon_button(ui, include_image!("../assets/icon_next.svg"), "skip").clicked() {
            player.skip();
        }
    });
    // Skip
    ui.add_enabled_ui(!player.is_empty(), |ui| {
        if icon_button(ui, include_image!("../assets/icon_stop.svg"), "stop").clicked() {
            player.stop();
        }
    });
}

/// Icon Button that reacts to hovering.
/// Image should be monochromatic (white) as it'll be tinted to intended color.
fn icon_button(ui: &mut Ui, source: ImageSource, id: &str) -> Response {
    // Doesn't work properly without using is_salt()?
    ui.scope_builder(UiBuilder::new().id_salt(id).sense(Sense::click()), |ui| {
        let color = ui.style().interact(&ui.response()).text_color();
        ui.add(Image::new(source).tint(color));
    })
    .response
}

/// Song position slider
fn position_control(ui: &mut Ui, player: &Player, width: f32) {
    let len = player.get_playback_length();
    let pos = player.get_playback_position();

    // This stops the slider from showing halfway if len is zero.
    let slider_len = if len.is_zero() { 1. } else { len.as_secs_f64() };

    ui.horizontal(|ui| {
        ui.spacing_mut().slider_width = width;
        ui.add_enabled(
            !len.is_zero(),
            Slider::new(&mut pos.as_secs_f64(), 0.0..=slider_len)
                .show_value(false)
                .trailing_fill(true),
        );
    });

    ui.label(format!("{}/{}", format_duration(pos), format_duration(len)));
}

fn volume_control(ui: &mut Ui, player: &mut Player) {
    let speaker_icon_str = match player.get_volume() {
        x if x == 0.0 => "🔇",
        x if (0.0..33.0).contains(&x) => "🔈",
        x if (33.0..66.0).contains(&x) => "🔉",
        _ => "🔊",
    };

    ui.menu_button(RichText::new(speaker_icon_str).size(ICON_SIZE), |ui| {
        let mut volume = player.get_volume();
        if ui
            .add(
                Slider::new(&mut volume, 0.0..=100.)
                    .vertical()
                    .show_value(false)
                    .trailing_fill(true),
            )
            .changed()
        {
            player.set_volume(volume);
        }
    });

    ui.label(format!("{:00}", player.get_volume()));
}
