use crate::constants::SAMPLE_RATE;
use crate::synth::SynthEngine;

const MAX_VOICES: usize = 8;

pub struct FmSynth {
    voices: [FmVoice; MAX_VOICES],
    ratio: f32,
    mod_index: f32,
    attack: f32,
    decay: f32,
}

#[derive(Clone, Copy)]
struct FmVoice {
    carrier_phase: f64,
    mod_phase: f64,
    freq: f64,
    active: bool,
    envelope: f32,
    note: u8,
    releasing: bool,
}

impl Default for FmVoice {
    fn default() -> Self {
        Self {
            carrier_phase: 0.0,
            mod_phase: 0.0,
            freq: 0.0,
            active: false,
            envelope: 0.0,
            note: 0,
            releasing: false,
        }
    }
}

impl FmSynth {
    pub fn new() -> Self {
        Self {
            voices: [FmVoice::default(); MAX_VOICES],
            ratio: 2.0,
            mod_index: 1.5,
            attack: 0.01,
            decay: 0.8,
        }
    }

    fn midi_to_freq(note: u8) -> f64 {
        440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0)
    }
}

impl SynthEngine for FmSynth {
    fn note_on(&mut self, note: u8, _velocity: f32) {
        let slot = self.voices.iter().position(|v| !v.active).unwrap_or(0);
        self.voices[slot] = FmVoice {
            carrier_phase: 0.0,
            mod_phase: 0.0,
            freq: Self::midi_to_freq(note),
            active: true,
            envelope: 0.0,
            note,
            releasing: false,
        };
    }

    fn note_off(&mut self, note: u8) {
        for v in &mut self.voices {
            if v.active && v.note == note {
                v.releasing = true;
            }
        }
    }

    fn process(&mut self, output: &mut [f32]) {
        let sr = SAMPLE_RATE as f64;
        let attack_rate = 1.0 / (self.attack * SAMPLE_RATE as f32).max(1.0);
        let decay_rate = 1.0 / (self.decay * SAMPLE_RATE as f32).max(1.0);
        let ratio = self.ratio as f64;
        let mod_idx = self.mod_index as f64;

        for sample in output.iter_mut() {
            let mut sum = 0.0f32;
            for voice in &mut self.voices {
                if !voice.active {
                    continue;
                }

                if voice.releasing {
                    voice.envelope -= decay_rate;
                    if voice.envelope <= 0.0 {
                        voice.active = false;
                        continue;
                    }
                } else if voice.envelope < 1.0 {
                    voice.envelope = (voice.envelope + attack_rate).min(1.0);
                }

                // 2-operator FM: carrier + modulator
                let mod_freq = voice.freq * ratio;
                let modulator = (voice.mod_phase * std::f64::consts::TAU).sin();
                let carrier_freq_mod = voice.freq + modulator * mod_idx * voice.freq;
                let carrier = (voice.carrier_phase * std::f64::consts::TAU).sin() as f32;

                sum += carrier * voice.envelope * 0.25;

                voice.carrier_phase += carrier_freq_mod / sr;
                voice.mod_phase += mod_freq / sr;
                if voice.carrier_phase >= 1.0 { voice.carrier_phase -= 1.0; }
                if voice.mod_phase >= 1.0 { voice.mod_phase -= 1.0; }
            }
            *sample += sum;
        }
    }

    fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.ratio = (value * 8.0).round().max(1.0),
            1 => self.mod_index = value * 5.0,
            2 => self.attack = value.clamp(0.001, 2.0),
            3 => self.decay = value.clamp(0.01, 5.0),
            _ => {}
        }
    }

    fn param_count(&self) -> usize { 4 }
    fn param_name(&self, index: usize) -> &str {
        match index {
            0 => "RATIO",
            1 => "MOD IX",
            2 => "ATTACK",
            3 => "DECAY",
            _ => "",
        }
    }
    fn name(&self) -> &str { "FM" }
}
