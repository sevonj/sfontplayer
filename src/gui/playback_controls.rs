use eframe::egui::{
    include_image, Button, Image, ImageButton, RichText, SelectableLabel, Slider, Ui,
};

use crate::SfontPlayer;

use super::conversions::format_duration;
pub(crate) fn playback_panel(ui: &mut Ui, app: &mut SfontPlayer) {
    ui.horizontal(|ui| {
        playback_controls(ui, app);

        position_control(ui, app);

        volume_control(ui, app);
    });
}

fn playback_controls(ui: &mut Ui, app: &mut SfontPlayer) {
    let (prev_enabled, next_enabled) = if let Some(idx) = app.get_queue_idx() {
        (idx > 0, idx < app.get_queue().len() - 1)
    } else {
        (false, false)
    };

    ui.label(RichText::new("🎵"));

    // Shuffle button
    if ui.add(SelectableLabel::new(app.shuffle, "🔀")).clicked() {
        app.shuffle = !app.shuffle;
        app.rebuild_queue();
    };

    // Prev button
    if ui.add_enabled(prev_enabled, Button::new("⏪")).clicked() {
        app.set_queue_idx(Some(app.get_queue_idx().unwrap() - 1));
        app.play_selected_song();
    }
    // PlayPause button
    if app.is_paused() {
        if ui.button("▶").clicked() {
            if app.is_empty() {
                app.start();
            } else {
                app.play();
            }
        }
    } else {
        if ui.button("⏸").clicked() {
            app.pause();
        }
    }
    // Next button
    if ui.add_enabled(next_enabled, Button::new("⏩")).clicked() {
        app.set_queue_idx(Some(app.get_queue_idx().unwrap() + 1));
        app.play_selected_song();
    }
    // Stop button
    if ui.add_enabled(!app.is_paused(), Button::new("⏹")).clicked() {
        app.stop()
    }

    /*

    // Image based buttons
    // Disabled until hover color change is figured out.

    let tint = ui.style().visuals.text_color();
    let img_play = Image::new(if app.is_paused() {
        include_image!("../../icon_play.svg")
    } else {
        include_image!("../../icon_pause.svg")
    })
    .tint(tint);
    let img_stop = Image::new(include_image!("../../icon_stop.svg")).tint(tint);
    let img_prev = Image::new(include_image!("../../icon_prev.svg")).tint(tint);
    let img_next = Image::new(include_image!("../../icon_next.svg")).tint(tint);

    // Prev button
    if ui
        .add_enabled(prev_enabled, ImageButton::new(img_prev).frame(true))
        .clicked()
    {
        app.set_queue_idx(Some(app.get_queue_idx().unwrap() - 1));
        app.play_selected_song();
    }
    // PlayPause button
    if ui.add(ImageButton::new(img_play).frame(true)).clicked() {
        if app.is_paused() {
            if app.is_empty() {
                app.start();
            } else {
                app.play();
            }
        } else {
            app.pause();
        }
    }
    // Next button
    if ui
        .add_enabled(next_enabled, ImageButton::new(img_next).frame(true))
        .clicked()
    {
        app.set_queue_idx(Some(app.get_queue_idx().unwrap() + 1));
        app.play_selected_song();
    }
    // Stop button
    if ui
        .add_enabled(!app.is_paused(), ImageButton::new(img_stop).frame(true))
        .clicked()
    {
        app.stop()
    }
    */ // Image based buttons
}

/// Song position slider
fn position_control(ui: &mut Ui, app: &mut SfontPlayer) {
    let len = app.get_midi_length();
    let pos = app.get_midi_position();

    // This stops the slider from showing halfway if len is zero.
    let slider_len = if len.is_zero() { 1. } else { len.as_secs_f64() };

    ui.horizontal(|ui| {
        ui.spacing_mut().slider_width = f32::max(ui.available_width() - 128., 64.);
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

    ui.menu_button(speaker_icon_str, |ui| {
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
