use eframe::egui::{DragValue, Label, TextWrapMode, Ui};
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;
use midi_msg::{MidiFile, MidiMsg, Track, TrackEvent};

use crate::player::{Player, PresetMapper};

const ROW_HEIGHT: f32 = 16.;

pub fn build_preset_mapper(ui: &mut Ui, player: &mut Player) {
    let height = ui.available_height();

    ui.set_width(ui.available_width());

    ui.horizontal(|ui| {
        ui.set_height(height);

        build_midi_presetlist(ui, player);
        ui.separator();
        build_font_presetlist(ui, player);
    });
}

fn build_midi_presetlist(ui: &mut Ui, player: &mut Player) {
    ui.vertical(|ui| {
        ui.label("MIDI file presets");

        let Some(inspector) = player.get_midi_inspector_mut() else {
            ui.label("No Inspector?!");
            return;
        };

        let tablebuilder = TableBuilder::new(ui)
            .id_salt("midi_presets")
            .striped(true)
            .column(Column::auto()) // Patch
            .column(Column::auto()) // Name
            .column(Column::auto()) // Demo
            .column(Column::auto()) // Arrow
            .column(Column::auto()) // Map Select
            .column(Column::auto()); // Demo

        let table = tablebuilder.header(20.0, |mut header| {
            // Patch
            header.col(|ui| {
                ui.label("Patch");
            });

            // Name
            header.col(|ui| {
                ui.label("Name (GM)");
            });

            header.col(|_| {}); // Demo

            header.col(|_| {}); // Arrow

            // Map Select
            header.col(|ui| {
                ui.label("Map to...");
            });

            header.col(|_| {}); // Demo
        });

        // Play test sound for this patch
        let mut test_patch = None;

        table.body(|mut body| {
            for patch in inspector.presets().clone().iter().sorted() {
                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        ui.label(format!("{patch}"));
                    });

                    row.col(|ui| {
                        let sound = PresetMapper::gm_sound_set_from_u8(*patch)
                            .map_or_else(|()| "Unknown".to_owned(), |sound| sound.to_string());
                        ui.add(Label::new(sound).wrap_mode(TextWrapMode::Extend));
                    });

                    row.col(|ui| {
                        if ui
                            .button("🔉")
                            .on_hover_text("Play demo (original)")
                            .clicked()
                        {
                            test_patch = Some(*patch);
                        }
                    });

                    row.col(|ui| {
                        ui.label("➡");
                    });

                    row.col(|ui| {
                        // Slider shenanigans: ´-1 == None´ and `0..=127 == Some(n)`
                        let mut value = get_map_value(*patch, &inspector.preset_mapper);
                        if ui
                            .add(DragValue::new(&mut value).range(-1..=127).custom_formatter(
                                |n, _| {
                                    if n < 0. {
                                        "None".into()
                                    } else {
                                        format!("{n}")
                                    }
                                },
                            ))
                            .changed()
                        {
                            set_map_value(*patch, value, &mut inspector.preset_mapper);
                        }
                    });

                    row.col(|ui| {
                        if ui
                            .button("🔉")
                            .on_hover_text("Play demo (mapped)")
                            .clicked()
                        {
                            let mapped = inspector.preset_mapper.mapped_patch(*patch);
                            test_patch = Some(mapped);
                        }
                    });
                });
            }
        });

        if let Some(patch) = test_patch {
            let midi_file = generate_test_midi(patch);
            let _ = player.play_midi(midi_file);
        }
    });
}

fn get_map_value(key: u8, mapper: &PresetMapper) -> isize {
    mapper.map.get(&key).map_or(-1, |value| *value as isize)
}

fn set_map_value(key: u8, value: isize, mapper: &mut PresetMapper) {
    if value < 0 {
        mapper.map.remove(&key);
        return;
    }
    mapper.map.insert(key, value as u8);
}

fn build_font_presetlist(ui: &mut Ui, player: &mut Player) {
    ui.vertical(|ui| {
        ui.label("Soundfont presets");

        let Some(inspector) = player.get_midi_inspector() else {
            ui.label("No Inspector?!");
            return;
        };

        let Some(soundfont) = inspector.soundfont() else {
            ui.label("Inspector has no soundfont.");
            return;
        };

        let tablebuilder = TableBuilder::new(ui)
            .id_salt("soundfont_presets")
            .striped(true)
            .column(Column::auto()) // Bank
            .column(Column::auto()) // Patch
            .column(Column::auto()) // Name
            .column(Column::exact(32.)); // Demo

        let table = tablebuilder.header(20.0, |mut header| {
            header.col(|ui| {
                ui.label("Bank");
            });

            header.col(|ui| {
                ui.label("Patch");
            });

            header.col(|ui| {
                ui.label("Name");
            });

            header.col(|_| {});
        });

        // Play test sound for this patch
        let mut test_patch = None;

        table.body(|mut body| {
            for preset in soundfont
                .get_presets()
                .iter()
                .sorted_by_key(|f| f.get_patch_number() + f.get_bank_number() * 128)
            {
                let bank = preset.get_bank_number();
                let patch = preset.get_patch_number();
                let name = preset.get_name();

                body.row(ROW_HEIGHT, |mut row| {
                    row.col(|ui| {
                        let mut text = format!("{bank}");
                        if bank != 0 {
                            ui.disable();
                            text += "？";
                        }
                        ui.label(text)
                            .on_disabled_hover_text("Multiple banks not yet supported.");
                    });

                    row.col(|ui| {
                        if bank != 0 {
                            ui.disable();
                        }
                        ui.label(format!("{patch}"));
                    });

                    row.col(|ui| {
                        if bank != 0 {
                            ui.disable();
                        }
                        ui.add(Label::new(name).wrap_mode(TextWrapMode::Extend));
                    });

                    row.col(|ui| {
                        if bank != 0 {
                            ui.disable();
                        }
                        if ui.button("🔉").on_hover_text("Play demo").clicked() {
                            test_patch = Some(patch as u8);
                        }
                    });
                });
            }
        });

        if let Some(patch) = test_patch {
            let midi_file = generate_test_midi(patch);
            let _ = player.play_midi(midi_file);
        }
    });
}

/// Generate a demo `MidiFile` for testing a specific patch.
fn generate_test_midi(patch: u8) -> MidiFile {
    let mut midi_file =
        MidiFile::from_midi(include_bytes!("../../assets/demo.mid")).expect("baked midi failed?");

    let event = MidiMsg::ChannelVoice {
        channel: midi_msg::Channel::Ch1,
        msg: midi_msg::ChannelVoiceMsg::ProgramChange { program: patch },
    };
    let track_event = TrackEvent {
        delta_time: 0,
        event,
        beat_or_frame: 0.,
    };

    let Track::Midi(midi_track) = &mut midi_file.tracks[0] else {
        panic!("baked midi has invalid track?!")
    };
    midi_track.insert(0, track_event);
    midi_file
}
