use crate::constants::SAMPLE_RATE;
use crate::effects::Effect;

pub struct Chorus {
    buffer: Vec<f32>,
    write_pos: usize,
    lfo_phase: f64,
    rate: f32,
    depth: f32,
    mix: f32,
    bypassed: bool,
}

impl Chorus {
    pub fn new() -> Self {
        Self {
            buffer: vec![0.0; SAMPLE_RATE as usize],
            write_pos: 0,
            lfo_phase: 0.0,
            rate: 0.5,
            depth: 0.003,
            mix: 0.5,
            bypassed: false,
        }
    }
}

impl Effect for Chorus {
    fn process(&mut self, input: &mut [f32]) {
        let sr = SAMPLE_RATE as f64;
        let buf_len = self.buffer.len();

        for sample in input.iter_mut() {
            let dry = *sample;

            // Write to buffer
            self.buffer[self.write_pos] = *sample;

            // LFO modulates delay time
            let lfo = (self.lfo_phase * std::f64::consts::TAU).sin();
            let delay_samples = (self.depth as f64 * sr) * (1.0 + lfo) * 0.5;
            let delay_samples = delay_samples.max(1.0) as usize;

            // Read from buffer with delay
            let read_pos = (self.write_pos + buf_len - delay_samples) % buf_len;
            let delayed = self.buffer[read_pos];

            self.write_pos = (self.write_pos + 1) % buf_len;
            self.lfo_phase += self.rate as f64 / sr;
            if self.lfo_phase >= 1.0 { self.lfo_phase -= 1.0; }

            *sample = dry * (1.0 - self.mix) + delayed * self.mix;
        }
    }

    fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.rate = value.clamp(0.1, 5.0),
            1 => self.depth = value.clamp(0.001, 0.02),
            2 => self.mix = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn param_count(&self) -> usize { 3 }
    fn param_name(&self, index: usize) -> &str {
        match index { 0 => "RATE", 1 => "DEPTH", 2 => "MIX", _ => "" }
    }
    fn name(&self) -> &str { "CHORUS" }
    fn set_bypass(&mut self, bypass: bool) { self.bypassed = bypass; }
    fn is_bypassed(&self) -> bool { self.bypassed }
}
