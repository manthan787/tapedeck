use crate::constants::SAMPLE_RATE;

/// Drum instrument synthesized from scratch
pub struct DrumVoice {
    phase: f64,
    freq: f64,
    envelope: f32,
    decay: f32,
    active: bool,
    noise_state: u32,
    noise_amount: f32,
    // Pitch envelope
    pitch_env: f32,
    pitch_decay: f32,
    pitch_amount: f64,
}

impl DrumVoice {
    pub fn trigger(&mut self) {
        self.phase = 0.0;
        self.envelope = 1.0;
        self.pitch_env = 1.0;
        self.active = true;
    }

    pub fn process(&mut self) -> f32 {
        if !self.active {
            return 0.0;
        }

        // Pitch envelope
        self.pitch_env *= self.pitch_decay;
        let freq = self.freq + self.pitch_amount * self.pitch_env as f64;

        // Oscillator
        let osc = (self.phase * std::f64::consts::TAU).sin() as f32;
        self.phase += freq / SAMPLE_RATE as f64;
        if self.phase >= 1.0 { self.phase -= 1.0; }

        // Noise
        self.noise_state = self.noise_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = (self.noise_state as f32 / u32::MAX as f32) * 2.0 - 1.0;

        // Mix
        let sample = osc * (1.0 - self.noise_amount) + noise * self.noise_amount;

        // Envelope
        self.envelope *= self.decay;
        if self.envelope < 0.001 {
            self.active = false;
        }

        sample * self.envelope
    }
}

pub struct DrumKit {
    pub voices: Vec<DrumVoice>,
    pub names: Vec<&'static str>,
}

impl DrumKit {
    pub fn new() -> Self {
        let voices = vec![
            // Kick — long boom, ~300ms decay
            DrumVoice {
                phase: 0.0, freq: 55.0, envelope: 0.0, decay: 0.99995,
                active: false, noise_state: 1, noise_amount: 0.05,
                pitch_env: 0.0, pitch_decay: 0.999, pitch_amount: 200.0,
            },
            // Snare — sharp crack + noise tail, ~200ms
            DrumVoice {
                phase: 0.0, freq: 180.0, envelope: 0.0, decay: 0.99992,
                active: false, noise_state: 2, noise_amount: 0.6,
                pitch_env: 0.0, pitch_decay: 0.998, pitch_amount: 80.0,
            },
            // Hi-hat — bright noise, ~80ms
            DrumVoice {
                phase: 0.0, freq: 800.0, envelope: 0.0, decay: 0.99984,
                active: false, noise_state: 3, noise_amount: 0.95,
                pitch_env: 0.0, pitch_decay: 0.999, pitch_amount: 0.0,
            },
            // Clap — noise burst, ~120ms
            DrumVoice {
                phase: 0.0, freq: 400.0, envelope: 0.0, decay: 0.99988,
                active: false, noise_state: 4, noise_amount: 0.8,
                pitch_env: 0.0, pitch_decay: 0.997, pitch_amount: 50.0,
            },
            // Tom — medium boom, ~250ms
            DrumVoice {
                phase: 0.0, freq: 100.0, envelope: 0.0, decay: 0.99994,
                active: false, noise_state: 5, noise_amount: 0.1,
                pitch_env: 0.0, pitch_decay: 0.9995, pitch_amount: 150.0,
            },
            // Rim — short click, ~60ms
            DrumVoice {
                phase: 0.0, freq: 600.0, envelope: 0.0, decay: 0.99980,
                active: false, noise_state: 6, noise_amount: 0.3,
                pitch_env: 0.0, pitch_decay: 0.996, pitch_amount: 100.0,
            },
        ];

        Self {
            voices,
            names: vec!["KICK", "SNARE", "HAT", "CLAP", "TOM", "RIM"],
        }
    }

    pub fn trigger(&mut self, instrument: usize) {
        if instrument < self.voices.len() {
            self.voices[instrument].trigger();
        }
    }

    pub fn process(&mut self) -> f32 {
        let mut sum = 0.0f32;
        for voice in &mut self.voices {
            sum += voice.process();
        }
        sum * 0.5
    }
}
