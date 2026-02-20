use crate::constants::SAMPLE_RATE;
use crate::synth::SynthEngine;

const MAX_VOICES: usize = 8;

pub struct SawSynth {
    voices: [SawVoice; MAX_VOICES],
    cutoff: f32,
    resonance: f32,
    attack: f32,
    decay: f32,
}

#[derive(Clone, Copy)]
struct SawVoice {
    phase: f64,
    freq: f64,
    active: bool,
    envelope: f32,
    note: u8,
    releasing: bool,
    // Simple one-pole filter state
    filter_state: f32,
}

impl Default for SawVoice {
    fn default() -> Self {
        Self {
            phase: 0.0,
            freq: 0.0,
            active: false,
            envelope: 0.0,
            note: 0,
            releasing: false,
            filter_state: 0.0,
        }
    }
}

impl SawSynth {
    pub fn new() -> Self {
        Self {
            voices: [SawVoice::default(); MAX_VOICES],
            cutoff: 0.5,
            resonance: 0.3,
            attack: 0.01,
            decay: 0.5,
        }
    }

    fn midi_to_freq(note: u8) -> f64 {
        440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0)
    }
}

impl SynthEngine for SawSynth {
    fn note_on(&mut self, note: u8, _velocity: f32) {
        let slot = self.voices.iter().position(|v| !v.active).unwrap_or(0);
        self.voices[slot] = SawVoice {
            phase: 0.0,
            freq: Self::midi_to_freq(note),
            active: true,
            envelope: 0.0,
            note,
            releasing: false,
            filter_state: 0.0,
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
        let filter_coeff = (self.cutoff * self.cutoff).clamp(0.001, 0.999);

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

                // Bandlimited saw: polyBLEP approximation
                let t = voice.phase as f32;
                let dt = (voice.freq / sr) as f32;
                let mut raw = 2.0 * t - 1.0;
                // PolyBLEP correction at discontinuity
                if t < dt {
                    let t_norm = t / dt;
                    raw += 2.0 * t_norm - t_norm * t_norm - 1.0;
                } else if t > 1.0 - dt {
                    let t_norm = (t - 1.0) / dt;
                    raw += t_norm * t_norm + 2.0 * t_norm + 1.0;
                }

                // Simple lowpass filter
                voice.filter_state += filter_coeff * (raw - voice.filter_state);
                let filtered = voice.filter_state + self.resonance * (voice.filter_state - raw);

                sum += filtered * voice.envelope * 0.25;

                voice.phase += voice.freq / sr;
                if voice.phase >= 1.0 {
                    voice.phase -= 1.0;
                }
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
    fn name(&self) -> &str { "SAW" }
}
