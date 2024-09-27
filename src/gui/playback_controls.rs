use eframe::egui::{RichText, Slider, Ui};
use egui::{include_image, Image, ImageSource, SelectableLabel, Sense, UiBuilder};

use crate::SfontPlayer;

const ICON_SIZE: f32 = 20.;

use super::conversions::format_duration;
pub(crate) fn playback_panel(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.horizontal(|ui| {
        playback_controls(ui, app);

        let slider_width = f32::max(ui.available_width() - 144., 64.);
        position_control(ui, app, slider_width);

        volume_control(ui, app);
    });
}

fn playback_controls(ui: &mut Ui, app: &mut SfontPlayer) {
    let (back_enabled, skip_enabled) = if let Some(idx) = app.get_workspace().queue_idx {
        (idx > 0, idx < app.get_workspace().queue.len() - 1)
    } else {
        (false, false)
    };

    // Current song info
    let current_hover_text = format!(
        "Currently {}: {}",
        if app.is_empty() {
            "selected"
        } else {
            "playing"
        },
        if let Some(index) = app.get_workspace().midi_idx {
            app.get_workspace().midis[index].get_name()
        } else {
            "Nothing".into()
        }
    );
    if ui
        .add_enabled(
            app.get_workspace().midi_idx.is_some(),
            egui::Button::new(RichText::new("🎵").size(ICON_SIZE)).frame(false),
        )
        .on_hover_text(current_hover_text)
        .clicked()
    {
        app.update_flags.scroll_to_song = true;
    }

    // Shuffle button
    if ui
        .add(SelectableLabel::new(
            app.shuffle,
            RichText::new("🔀").size(ICON_SIZE),
        ))
        .clicked()
    {
        app.toggle_shuffle();
    };

    // Skip back
    ui.add_enabled_ui(back_enabled, |ui| {
        if icon_button(ui, include_image!("../assets/icon_prev.svg"), "back").clicked() {
            let _ = app.skip_back();
        }
    });
    // Playpause
    if app.is_paused() {
        if icon_button(ui, include_image!("../assets/icon_play.svg"), "play").clicked() {
            if app.is_empty() {
                app.start();
            } else {
                app.play();
            }
        };
    } else if icon_button(ui, include_image!("../assets/icon_pause.svg"), "pause").clicked() {
        app.pause();
    }
    // Skip
    ui.add_enabled_ui(skip_enabled, |ui| {
        if icon_button(ui, include_image!("../assets/icon_next.svg"), "skip").clicked() {
            let _ = app.skip();
        }
    });
    // Skip
    ui.add_enabled_ui(!app.is_empty(), |ui| {
        if icon_button(ui, include_image!("../assets/icon_stop.svg"), "stop").clicked() {
            app.stop();
        }
    });
}

/// Icon Button that reacts to hovering.
/// Image should be monochromatic (white) as it'll be tinted to intended color.
fn icon_button(ui: &mut Ui, source: ImageSource, id: &str) -> egui::Response {
    // Doesn't work properly without using is_salt()?
    ui.scope_builder(UiBuilder::new().id_salt(id).sense(Sense::click()), |ui| {
        let color = ui.style().interact(&ui.response()).text_color();
        ui.add(Image::new(source).tint(color));
    })
    .response
}

/// Song position slider
fn position_control(ui: &mut Ui, app: &mut SfontPlayer, width: f32) {
    let len = app.get_midi_length();
    let pos = app.get_midi_position();

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

fn volume_control(ui: &mut Ui, app: &mut SfontPlayer) {
    let speaker_icon_str = match app.volume {
        x if x == 0.0 => "🔇",
        x if (0.0..33.0).contains(&x) => "🔈",
        x if (33.0..66.0).contains(&x) => "🔉",
        _ => "🔊",
    };

    ui.menu_button(RichText::new(speaker_icon_str).size(ICON_SIZE), |ui| {
        if ui
            .add(
                Slider::new(&mut app.volume, 0.0..=100.)
                    .vertical()
                    .show_value(false)
                    .trailing_fill(true),
            )
            .changed()
        {
            app.update_volume();
        }
    });

    ui.label(format!("{:00}", app.volume));
}
