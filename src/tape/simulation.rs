use crate::constants::SAMPLE_RATE;

/// Tape simulation processor: wow, flutter, saturation, hiss, HF rolloff
pub struct TapeSimulation {
    pub enabled: bool,
    // Wow: slow pitch modulation
    wow_phase: f64,
    pub wow_depth: f32,
    pub wow_rate: f32,
    // Flutter: fast pitch modulation
    flutter_phase: f64,
    pub flutter_depth: f32,
    pub flutter_rate: f32,
    // Saturation
    pub drive: f32,
    // Hiss
    noise_state: u32,
    pub hiss_level: f32,
    // HF rolloff (one-pole lowpass)
    pub rolloff_freq: f32,
    lp_state_l: f32,
    lp_state_r: f32,
}

impl TapeSimulation {
    pub fn new() -> Self {
        Self {
            enabled: false,
            wow_phase: 0.0,
            wow_depth: 0.002,
            wow_rate: 1.0,
            flutter_phase: 0.0,
            flutter_depth: 0.0003,
            flutter_rate: 8.0,
            drive: 2.0,
            noise_state: 42,
            hiss_level: 0.005,
            rolloff_freq: 14000.0,
            lp_state_l: 0.0,
            lp_state_r: 0.0,
        }
    }

    /// Process a stereo pair (left, right)
    pub fn process_stereo(&mut self, left: &mut f32, right: &mut f32) {
        if !self.enabled {
            return;
        }

        // Saturation: tanh waveshaping
        *left = (*left * self.drive).tanh() / self.drive.tanh();
        *right = (*right * self.drive).tanh() / self.drive.tanh();

        // HF rolloff (simple one-pole lowpass)
        let sr = SAMPLE_RATE as f32;
        let rc = 1.0 / (2.0 * std::f32::consts::PI * self.rolloff_freq);
        let dt = 1.0 / sr;
        let alpha = dt / (rc + dt);

        self.lp_state_l += alpha * (*left - self.lp_state_l);
        self.lp_state_r += alpha * (*right - self.lp_state_r);
        *left = self.lp_state_l;
        *right = self.lp_state_r;

        // Hiss (pink-ish noise)
        self.noise_state = self.noise_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = (self.noise_state as f32 / u32::MAX as f32) * 2.0 - 1.0;
        *left += noise * self.hiss_level;
        *right += noise * self.hiss_level * 0.8; // Slightly different per channel
    }

    /// Get wow+flutter pitch offset in fractional samples
    pub fn get_pitch_offset(&mut self) -> f64 {
        let sr = SAMPLE_RATE as f64;

        // Wow LFO
        let wow = (self.wow_phase * std::f64::consts::TAU).sin() * self.wow_depth as f64 * sr;
        self.wow_phase += self.wow_rate as f64 / sr;
        if self.wow_phase >= 1.0 { self.wow_phase -= 1.0; }

        // Flutter LFO
        let flutter = (self.flutter_phase * std::f64::consts::TAU).sin() * self.flutter_depth as f64 * sr;
        self.flutter_phase += self.flutter_rate as f64 / sr;
        if self.flutter_phase >= 1.0 { self.flutter_phase -= 1.0; }

        wow + flutter
    }
}
