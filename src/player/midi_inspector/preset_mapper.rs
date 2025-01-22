use std::collections::HashMap;

use midi_msg::{ChannelVoiceMsg, GMSoundSet, MidiFile, MidiMsg, Track};

#[derive(Debug, Default, Clone)]
pub struct PresetMapper {
    pub map: HashMap<u8, u8>,
}

impl PresetMapper {
    pub fn remap_value(&self, patch: u8) -> u8 {
        self.map.get(&patch).map_or_else(|| patch, |f| *f)
    }

    pub fn remap_midi(&self, midi_file: &mut MidiFile) {
        for track in &mut midi_file.tracks {
            let Track::Midi(midi_track) = track else {
                continue;
            };
            for track_event in midi_track {
                let MidiMsg::ChannelVoice { msg, .. } = &mut track_event.event else {
                    continue;
                };
                let ChannelVoiceMsg::ProgramChange { program } = msg else {
                    continue;
                };
                let remapped = self.remap_value(*program);
                *msg = ChannelVoiceMsg::ProgramChange { program: remapped };
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub const fn gm_sound_set_from_u8(value: u8) -> Result<GMSoundSet, ()> {
        match value {
            0 => Ok(GMSoundSet::AcousticGrandPiano),
            1 => Ok(GMSoundSet::BrightAcousticPiano),
            2 => Ok(GMSoundSet::ElectricGrandPiano),
            3 => Ok(GMSoundSet::HonkytonkPiano),
            4 => Ok(GMSoundSet::ElectricPiano1),
            5 => Ok(GMSoundSet::ElectricPiano2),
            6 => Ok(GMSoundSet::Harpsichord),
            7 => Ok(GMSoundSet::Clavi),
            8 => Ok(GMSoundSet::Celesta),
            9 => Ok(GMSoundSet::Glockenspiel),
            10 => Ok(GMSoundSet::MusicBox),
            11 => Ok(GMSoundSet::Vibraphone),
            12 => Ok(GMSoundSet::Marimba),
            13 => Ok(GMSoundSet::Xylophone),
            14 => Ok(GMSoundSet::TubularBells),
            15 => Ok(GMSoundSet::Dulcimer),
            16 => Ok(GMSoundSet::DrawbarOrgan),
            17 => Ok(GMSoundSet::PercussiveOrgan),
            18 => Ok(GMSoundSet::RockOrgan),
            19 => Ok(GMSoundSet::ChurchOrgan),
            20 => Ok(GMSoundSet::ReedOrgan),
            21 => Ok(GMSoundSet::Accordion),
            22 => Ok(GMSoundSet::Harmonica),
            23 => Ok(GMSoundSet::TangoAccordion),
            24 => Ok(GMSoundSet::AcousticGuitarNylon),
            25 => Ok(GMSoundSet::AcousticGuitarSteel),
            26 => Ok(GMSoundSet::ElectricGuitarJazz),
            27 => Ok(GMSoundSet::ElectricGuitarClean),
            28 => Ok(GMSoundSet::ElectricGuitarMuted),
            29 => Ok(GMSoundSet::OverdrivenGuitar),
            30 => Ok(GMSoundSet::DistortionGuitar),
            31 => Ok(GMSoundSet::GuitarHarmonics),
            32 => Ok(GMSoundSet::AcousticBass),
            33 => Ok(GMSoundSet::ElectricBassFinger),
            34 => Ok(GMSoundSet::ElectricBassPick),
            35 => Ok(GMSoundSet::FretlessBass),
            36 => Ok(GMSoundSet::SlapBass1),
            37 => Ok(GMSoundSet::SlapBass2),
            38 => Ok(GMSoundSet::SynthBass1),
            39 => Ok(GMSoundSet::SynthBass2),
            40 => Ok(GMSoundSet::Violin),
            41 => Ok(GMSoundSet::Viola),
            42 => Ok(GMSoundSet::Cello),
            43 => Ok(GMSoundSet::Contrabass),
            44 => Ok(GMSoundSet::TremoloStrings),
            45 => Ok(GMSoundSet::PizzicatoStrings),
            46 => Ok(GMSoundSet::OrchestralHarp),
            47 => Ok(GMSoundSet::Timpani),
            48 => Ok(GMSoundSet::StringEnsemble1),
            49 => Ok(GMSoundSet::StringEnsemble2),
            50 => Ok(GMSoundSet::SynthStrings1),
            51 => Ok(GMSoundSet::SynthStrings2),
            52 => Ok(GMSoundSet::ChoirAahs),
            53 => Ok(GMSoundSet::VoiceOohs),
            54 => Ok(GMSoundSet::SynthVoice),
            55 => Ok(GMSoundSet::OrchestraHit),
            56 => Ok(GMSoundSet::Trumpet),
            57 => Ok(GMSoundSet::Trombone),
            58 => Ok(GMSoundSet::Tuba),
            59 => Ok(GMSoundSet::MutedTrumpet),
            60 => Ok(GMSoundSet::FrenchHorn),
            61 => Ok(GMSoundSet::BrassSection),
            62 => Ok(GMSoundSet::SynthBrass1),
            63 => Ok(GMSoundSet::SynthBrass2),
            64 => Ok(GMSoundSet::SopranoSax),
            65 => Ok(GMSoundSet::AltoSax),
            66 => Ok(GMSoundSet::TenorSax),
            67 => Ok(GMSoundSet::BaritoneSax),
            68 => Ok(GMSoundSet::Oboe),
            69 => Ok(GMSoundSet::EnglishHorn),
            70 => Ok(GMSoundSet::Bassoon),
            71 => Ok(GMSoundSet::Clarinet),
            72 => Ok(GMSoundSet::Piccolo),
            73 => Ok(GMSoundSet::Flute),
            74 => Ok(GMSoundSet::Recorder),
            75 => Ok(GMSoundSet::PanFlute),
            76 => Ok(GMSoundSet::BlownBottle),
            77 => Ok(GMSoundSet::Shakuhachi),
            78 => Ok(GMSoundSet::Whistle),
            79 => Ok(GMSoundSet::Ocarina),
            80 => Ok(GMSoundSet::Lead1),
            81 => Ok(GMSoundSet::Lead2),
            82 => Ok(GMSoundSet::Lead3),
            83 => Ok(GMSoundSet::Lead4),
            84 => Ok(GMSoundSet::Lead5),
            85 => Ok(GMSoundSet::Lead6),
            86 => Ok(GMSoundSet::Lead7),
            87 => Ok(GMSoundSet::Lead8),
            88 => Ok(GMSoundSet::Pad1),
            89 => Ok(GMSoundSet::Pad2),
            90 => Ok(GMSoundSet::Pad3),
            91 => Ok(GMSoundSet::Pad4),
            92 => Ok(GMSoundSet::Pad5),
            93 => Ok(GMSoundSet::Pad6),
            94 => Ok(GMSoundSet::Pad7),
            95 => Ok(GMSoundSet::Pad8),
            96 => Ok(GMSoundSet::FX1),
            97 => Ok(GMSoundSet::FX2),
            98 => Ok(GMSoundSet::FX3),
            99 => Ok(GMSoundSet::FX4),
            100 => Ok(GMSoundSet::FX5),
            101 => Ok(GMSoundSet::FX6),
            102 => Ok(GMSoundSet::FX7),
            103 => Ok(GMSoundSet::FX8),
            104 => Ok(GMSoundSet::Sitar),
            105 => Ok(GMSoundSet::Banjo),
            106 => Ok(GMSoundSet::Shamisen),
            107 => Ok(GMSoundSet::Koto),
            108 => Ok(GMSoundSet::Kalimba),
            109 => Ok(GMSoundSet::Bagpipe),
            110 => Ok(GMSoundSet::Fiddle),
            111 => Ok(GMSoundSet::Shanai),
            112 => Ok(GMSoundSet::TinkleBell),
            113 => Ok(GMSoundSet::Agogo),
            114 => Ok(GMSoundSet::SteelDrums),
            115 => Ok(GMSoundSet::Woodblock),
            116 => Ok(GMSoundSet::TaikoDrum),
            117 => Ok(GMSoundSet::MelodicTom),
            118 => Ok(GMSoundSet::SynthDrum),
            119 => Ok(GMSoundSet::ReverseCymbal),
            120 => Ok(GMSoundSet::GuitarFretNoise),
            121 => Ok(GMSoundSet::BreathNoise),
            122 => Ok(GMSoundSet::Seashore),
            123 => Ok(GMSoundSet::BirdTweet),
            124 => Ok(GMSoundSet::TelephoneRing),
            125 => Ok(GMSoundSet::Helicopter),
            126 => Ok(GMSoundSet::Applause),
            127 => Ok(GMSoundSet::Gunshot),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_get_remapped() {
        let mut mapper = PresetMapper::default();
        mapper.map.insert(4, 7);
        mapper.map.insert(5, 7);

        assert_eq!(mapper.remap_value(2), 2);
        assert_eq!(mapper.remap_value(3), 3);
        assert_eq!(mapper.remap_value(4), 7); // mapped
        assert_eq!(mapper.remap_value(5), 7); // mapped
        assert_eq!(mapper.remap_value(6), 6);
        assert_eq!(mapper.remap_value(7), 7);
    }
}
