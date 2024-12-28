use super::{custom_controls::collapse_button, GuiState};
use crate::midi_inspector::{MidiInspector, MidiInspectorTrack};
use eframe::egui::{Color32, Frame, Label, RichText, ScrollArea, Style, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use midi_msg::{MidiMsg, Track};
use std::path::Path;

const TRACKHEAD_WIDTH: f32 = 128.;

pub fn midi_inspector(ui: &mut Ui, inspector: &mut MidiInspector, gui: &mut GuiState) {
    inspector_toolbar(ui, gui);
    ui.separator();

    ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());

        header_panel(ui, &inspector.header, &inspector.filepath);
        for i in 0..inspector.tracks.len() {
            let track = &mut inspector.tracks[i];
            ui.separator();
            ui.push_id(format!("track_ui_{i}"), |ui| match &track.track {
                Track::Midi(..) => midi_track_panel(ui, i, track),
                Track::AlienChunk(..) => nonstandard_track_panel(ui, i, track),
            });
        }
    });
}

fn inspector_toolbar(ui: &mut Ui, gui: &mut GuiState) {
    ui.horizontal(|ui| {
        ui.label("MIDI Inspector");
        if ui.button("close").clicked() {
            gui.update_flags.close_midi_inspector = true;
        }
    });
}

/// MIDI Header
fn header_panel(ui: &mut Ui, header: &midi_msg::Header, filepath: &Path) {
    Frame::group(ui.style())
        .fill(ui.style().visuals.panel_fill)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(format!("{filepath:?}"));
            ui.label(format!("Format:   {:?}", header.format));
            ui.label(format!("Tracks:   {:?}", header.num_tracks));
            ui.label(format!("Division: {:?}", header.num_tracks));
        });
}

/// MIDI Track - Unknown track type placeholder.
fn nonstandard_track_panel(ui: &mut Ui, i: usize, track: &MidiInspectorTrack) {
    Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());

        ui.label(format!("Track {i} [UNKNOWN]"));
        ui.label(format!(
            "This is a nonstandard track. Length: {:?} bytes",
            track.track.len()
        ));
    });
}

/// MIDI Track - Normal
fn midi_track_panel(ui: &mut Ui, i: usize, track: &mut MidiInspectorTrack) {
    let content = track.track.events();
    let bgcol = ui.visuals().code_bg_color;

    ui.horizontal(|ui| {
        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(TRACKHEAD_WIDTH);

            ui.vertical(|ui| {
                ui.add(Label::new(format!("Track {i} [MIDI]")).wrap_mode(TextWrapMode::Truncate));
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.add_enabled_ui(track.name.is_some(), |ui| {
                        let name_str = track.name.as_deref().unwrap_or("[NO NAME]");
                        ui.add(
                            Label::new(RichText::new(name_str).background_color(bgcol))
                                .wrap_mode(TextWrapMode::Truncate),
                        );
                    });
                });
                ui.label(format!("Events:   {:?}", content.len()));
            });
        });

        let open = &mut track.open;
        ui.add(collapse_button(open));

        if !track.open {
            return;
        }

        ui.vertical(|ui| {
            let tablebuilder = TableBuilder::new(ui)
                .id_salt(format!("tracktable{i}"))
                .striped(true)
                .vscroll(false)
                //.sense(Sense::click())
                .column(Column::exact(32.)) // index
                .column(Column::exact(48.)) // delta_t
                .column(Column::exact(64.)) // abs_t
                .column(Column::remainder()); // Message

            let table = tablebuilder.header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("idx");
                });
                header.col(|ui| {
                    ui.label("delta_t");
                });
                header.col(|ui| {
                    ui.label("time");
                });
                header.col(|ui| {
                    ui.label("event");
                });
            });

            table.body(|body| {
                body.rows(28., content.len(), |mut row| {
                    let index = row.index();
                    let track_event = &content[index];
                    let delta_t = track_event.delta_time;
                    let beat_or_frame = track_event.beat_or_frame;
                    let event = &track_event.event;

                    row.col(|ui| {
                        ui.label(format!("{index}"));
                    });
                    row.col(|ui| {
                        ui.label(format!("{delta_t}",));
                    });
                    row.col(|ui| {
                        ui.label(format!("{beat_or_frame}",));
                    });
                    row.col(|ui| {
                        Frame::group(ui.style())
                            .fill(event_color(ui.style(), event))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.strong(format!("{event:?}"));
                                    ui.strong(format!("raw: {:02X?}", event.to_midi()));
                                });
                            });
                    });
                });
            });
        });
    });
}

fn event_color(style: &Style, msg: &MidiMsg) -> Color32 {
    let color = match msg {
        MidiMsg::ChannelVoice { .. } => Color32::from_hex("#458588"),
        MidiMsg::RunningChannelVoice { .. } => Color32::from_hex("#98971A"),
        MidiMsg::ChannelMode { .. } => Color32::from_hex("#D79921"),
        MidiMsg::RunningChannelMode { .. } => Color32::from_hex("#CC241d"),
        MidiMsg::SystemCommon { .. } => Color32::from_hex("#B16286"),
        MidiMsg::SystemRealTime { .. } => Color32::from_hex("#689D6A"),
        MidiMsg::SystemExclusive { .. } => Color32::from_hex("#A89984"),
        MidiMsg::Meta { .. } => Color32::from_hex("#D65D0E"),
        MidiMsg::Invalid { .. } => Ok(style.visuals.panel_fill),
    };
    color.expect("impossible")
}

//fn event_widget(ui: &mut Ui, track_event: &TrackEvent) {
//    Frame::group(ui.style()).show(ui, |ui| {
//        ui.horizontal(|ui| {
//            ui.label(format!("Delta t: {}", track_event.delta_time));
//            ui.label(format!("Absolute t: {}", track_event.beat_or_frame));
//        });
//
//        ui.label(format!("{:?}", track_event.event));
//    });
//}
