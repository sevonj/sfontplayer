mod event_browser;
mod midi_renderer;
mod preset_mapper;

use eframe::egui::Ui;
use midi_renderer::build_midi_renderer;
use preset_mapper::build_preset_mapper;

use super::GuiState;
use crate::player::Player;
use event_browser::build_event_browser;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum MidiInspectorTab {
    #[default]
    EventBrowser,
    PresetMapper,
    MidiRenderer,
}

pub fn build_midi_inspector(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    build_inspector_toolbar(ui, player, gui);

    ui.separator();

    let Some(inspector) = player.get_midi_inspector_mut() else {
        return;
    };

    match gui.midi_inspector_tab {
        MidiInspectorTab::EventBrowser => build_event_browser(ui, inspector),
        MidiInspectorTab::PresetMapper => build_preset_mapper(ui, player),
        MidiInspectorTab::MidiRenderer => build_midi_renderer(ui, inspector, gui),
    }
}

fn build_inspector_toolbar(ui: &mut Ui, player: &mut Player, gui: &mut GuiState) {
    ui.horizontal(|ui| {
        ui.label("MIDI Inspector");
        if ui.button("close").clicked() {
            player.close_midi_inspector();
        }

        ui.separator();

        ui.selectable_value(
            &mut gui.midi_inspector_tab,
            MidiInspectorTab::EventBrowser,
            "Event Browser",
        );
        ui.selectable_value(
            &mut gui.midi_inspector_tab,
            MidiInspectorTab::PresetMapper,
            "Preset Mapper",
        );
        ui.selectable_value(
            &mut gui.midi_inspector_tab,
            MidiInspectorTab::MidiRenderer,
            "Render",
        );
    });
}
