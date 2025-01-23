use eframe::egui::{Frame, ScrollArea, TextBuffer, Ui};
use rfd::FileDialog;

use crate::{gui::GuiState, player::MidiInspector};

pub fn build_midi_renderer(ui: &mut Ui, inspector: &mut MidiInspector, gui: &mut GuiState) {
    ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());

        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label("Format: WAVE");
            ui.label("Channels: 2");
            ui.label(format!(
                "Sample rate: {}",
                inspector.midi_renderer.sample_rate
            ));
            ui.label("Bit depth: 16");
        });

        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label("Filepath");
            ui.horizontal(|ui| {
                if ui.button("Choose").clicked() {
                    show_filepath_dialog(inspector);
                }
                Frame::canvas(ui.style()).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(inspector.midi_renderer.filepath.to_string_lossy().as_str());
                });
            })
        });

        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());

            if ui.button("Render").clicked() {
                match inspector.render() {
                    Ok(()) => gui.toast_success("Render finished!"),
                    Err(e) => gui.toast_error(e.to_string()),
                }
            }
        });
    });
}

pub fn show_filepath_dialog(inspector: &mut MidiInspector) {
    if let Some(filepath) = FileDialog::new()
        .add_filter("Wave file", &["wav"])
        .set_title("Render to file")
        .set_file_name(format!("{}.wav", inspector.midimeta().filename()))
        .save_file()
    {
        inspector.midi_renderer.filepath = filepath;
    }
}
