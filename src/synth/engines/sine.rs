use crate::constants::SAMPLE_RATE;
use crate::synth::SynthEngine;

const MAX_VOICES: usize = 8;

pub struct SineSynth {
    voices: [SineVoice; MAX_VOICES],
    attack: f32,
    decay: f32,
}

struct SineVoice {
    phase: f64,
    freq: f64,
    active: bool,
    envelope: f32,
    note: u8,
    releasing: bool,
}

impl Default for SineVoice {
    fn default() -> Self {
        Self {
            phase: 0.0,
            freq: 0.0,
            active: false,
            envelope: 0.0,
            note: 0,
            releasing: false,
        }
    }
}

impl SineSynth {
    pub fn new() -> Self {
        Self {
            voices: [SineVoice::default(); MAX_VOICES],
            attack: 0.01,
            decay: 0.3,
        }
    }

    fn midi_to_freq(note: u8) -> f64 {
        440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0)
    }
}

impl Clone for SineVoice {
    fn clone(&self) -> Self {
        Self {
            phase: self.phase,
            freq: self.freq,
            active: self.active,
            envelope: self.envelope,
            note: self.note,
            releasing: self.releasing,
        }
    }
}

impl Copy for SineVoice {}

impl SynthEngine for SineSynth {
    fn note_on(&mut self, note: u8, _velocity: f32) {
        // Find free voice or steal oldest
        let slot = self
            .voices
            .iter()
            .position(|v| !v.active)
            .unwrap_or(0);
        self.voices[slot] = SineVoice {
            phase: 0.0,
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

        for sample in output.iter_mut() {
            let mut sum = 0.0f32;
            for voice in &mut self.voices {
                if !voice.active {
                    continue;
                }

                // Envelope
                if voice.releasing {
                    voice.envelope -= decay_rate;
                    if voice.envelope <= 0.0 {
                        voice.active = false;
                        voice.envelope = 0.0;
                        continue;
                    }
                } else if voice.envelope < 1.0 {
                    voice.envelope = (voice.envelope + attack_rate).min(1.0);
                }

                // Oscillator
                let val = (voice.phase * std::f64::consts::TAU).sin() as f32;
                sum += val * voice.envelope * 0.3;

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
            0 => {} // Freq offset - not used for sine
            1 => {} // Resonance - not applicable
            2 => self.attack = value.clamp(0.001, 2.0),
            3 => self.decay = value.clamp(0.01, 5.0),
            _ => {}
        }
    }

    fn param_count(&self) -> usize { 4 }
    fn param_name(&self, index: usize) -> &str {
        match index {
            0 => "DETUNE",
            1 => "--",
            2 => "ATTACK",
            3 => "DECAY",
            _ => "",
        }
    }
    fn name(&self) -> &str { "SINE" }
}
