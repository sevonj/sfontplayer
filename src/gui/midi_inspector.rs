use super::GuiState;
use crate::midi_inspector::MidiInspector;
use eframe::egui::{Color32, Frame, ScrollArea, Style, Ui};
use egui_extras::{Column, TableBuilder};
use midi_msg::{MidiMsg, Track, TrackEvent};
use std::path::Path;

const TRACKHEAD_WIDTH: f32 = 128.;

pub fn midi_inspector(ui: &mut Ui, inspector: &MidiInspector, gui: &mut GuiState) {
    let midifile = &inspector.midifile;

    inspector_toolbar(ui, gui);
    ui.separator();

    ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());

        header_panel(ui, &midifile.header, &inspector.filepath);
        for (i, track) in midifile.tracks.iter().enumerate() {
            ui.separator();
            ui.push_id(format!("track_ui_{i}"), |ui| match track {
                Track::Midi(content) => midi_track_panel(ui, i, content),
                Track::AlienChunk(content) => nonstandard_track_panel(ui, i, content),
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

fn nonstandard_track_panel(ui: &mut Ui, i: usize, content: &[u8]) {
    Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());

        ui.label(format!("Track {i}"));
        ui.label("Unknown");
        ui.label(format!(
            "This is a nonstandard track. Length: {:?} bytes",
            content.len()
        ));
    });
}

fn midi_track_panel(ui: &mut Ui, i: usize, content: &[TrackEvent]) {
    ui.horizontal(|ui| {
        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(TRACKHEAD_WIDTH);

            ui.vertical(|ui| {
                ui.label(format!("Track {i}"));
                ui.label("MIDI track");
                ui.label(format!("Events:   {:?}", content.len()));
            });
        });

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
                body.rows(24., content.len(), |mut row| {
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
                                ui.strong(format!("{event:?}",));
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
