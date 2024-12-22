//! `RustySynth` integration: This makes [`rustysynth::Synthesizer`] compatible
//! with `MidiSequencer` and `midi_msg` crate's event format.
//!

use midi_msg::MidiMsg;
use rustysynth::Synthesizer;

use super::midisequencer::MidiSink;

impl MidiSink for Synthesizer {
    fn receive_midi(&mut self, msg: &MidiMsg) -> Result<(), ()> {
        let raw = msg.to_midi();

        if let 2..=3 = raw.len() {
            send_raw_event(self, &raw);
            return Ok(());
        }

        if raw.len() == 5 {
            // Break a message that contains MSB and LSB in one into two
            // separate ones for rustysynth consumption.
            if let 0x62 | 0x64 = raw[1] {
                let msb = vec![raw[0], raw[3], raw[4]];
                let lsb = &raw[0..3];
                send_raw_event(self, &msb);
                send_raw_event(self, lsb);
                return Ok(());
            }
        }

        Err(())
    }
    fn reset(&mut self) {
        self.reset();
    }
}

fn send_raw_event(synth: &mut Synthesizer, raw: &[u8]) {
    let channel = raw[0] & 0x0f;
    let command = raw[0] & 0xf0;
    let data1;
    let data2;
    match raw.len() {
        2 => {
            data1 = raw[1];
            data2 = 0;
        }
        3 => {
            data1 = raw[1];
            data2 = raw[2];
        }
        _ => panic!("This shouldn't happen. Check length before calling."),
    }
    synth.process_midi_message(channel.into(), command.into(), data1.into(), data2.into());
}
