use crate::constants::SAMPLE_RATE;
use crate::synth::SynthEngine;

const MAX_VOICES: usize = 8;

/// Filtered noise synthesizer (useful for percussion)
pub struct NoiseSynth {
    voices: [NoiseVoice; MAX_VOICES],
    cutoff: f32,
    resonance: f32,
    attack: f32,
    decay: f32,
}

#[derive(Clone, Copy)]
struct NoiseVoice {
    active: bool,
    note: u8,
    envelope: f32,
    releasing: bool,
    rng_state: u32,
    filter_lp: f32,
    filter_bp: f32,
}

impl Default for NoiseVoice {
    fn default() -> Self {
        Self {
            active: false,
            note: 0,
            envelope: 0.0,
            releasing: false,
            rng_state: 12345,
            filter_lp: 0.0,
            filter_bp: 0.0,
        }
    }
}

impl NoiseSynth {
    pub fn new() -> Self {
        Self {
            voices: [NoiseVoice::default(); MAX_VOICES],
            cutoff: 0.4,
            resonance: 0.3,
            attack: 0.001,
            decay: 0.2,
        }
    }
}

impl SynthEngine for NoiseSynth {
    fn note_on(&mut self, note: u8, _velocity: f32) {
        let slot = self.voices.iter().position(|v| !v.active).unwrap_or(0);
        self.voices[slot] = NoiseVoice {
            active: true,
            note,
            envelope: 0.0,
            releasing: false,
            rng_state: note as u32 * 1664525 + 1013904223,
            filter_lp: 0.0,
            filter_bp: 0.0,
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
        let attack_rate = 1.0 / (self.attack * SAMPLE_RATE as f32).max(1.0);
        let decay_rate = 1.0 / (self.decay * SAMPLE_RATE as f32).max(1.0);
        let f = (self.cutoff * self.cutoff * 0.99).clamp(0.001, 0.99);
        let q = 1.0 - self.resonance.clamp(0.0, 0.95);

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

                // White noise
                voice.rng_state = voice.rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
                let noise = (voice.rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

                // State variable filter
                voice.filter_lp += f * voice.filter_bp;
                let hp = noise - voice.filter_lp - q * voice.filter_bp;
                voice.filter_bp += f * hp;

                sum += voice.filter_lp * voice.envelope * 0.3;
            }
            *sample += sum;
        }
    }

    fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.cutoff = value.clamp(0.01, 1.0),
            1 => self.resonance = value.clamp(0.0, 0.95),
            2 => self.attack = value.clamp(0.001, 2.0),
            3 => self.decay = value.clamp(0.01, 5.0),
            _ => {}
        }
    }

    fn param_count(&self) -> usize { 4 }
    fn param_name(&self, index: usize) -> &str {
        match index {
            0 => "CUTOFF",
            1 => "RESO",
            2 => "ATTACK",
            3 => "DECAY",
            _ => "",
        }
    }
    fn name(&self) -> &str { "NOISE" }
}
