use crate::effects::Effect;
use crate::constants::SAMPLE_RATE;

pub struct Delay {
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_pos: usize,
    time: f32,      // in seconds
    feedback: f32,
    mix: f32,
    bypassed: bool,
}

impl Delay {
    pub fn new() -> Self {
        let max_samples = SAMPLE_RATE as usize * 2; // 2 sec max delay
        Self {
            buffer_l: vec![0.0; max_samples],
            buffer_r: vec![0.0; max_samples],
            write_pos: 0,
            time: 0.375,
            feedback: 0.4,
            mix: 0.3,
            bypassed: false,
        }
    }
}

impl Effect for Delay {
    fn process(&mut self, input: &mut [f32]) {
        let delay_samples = (self.time * SAMPLE_RATE as f32) as usize;
        let delay_samples = delay_samples.min(self.buffer_l.len() - 1).max(1);

        for sample in input.iter_mut() {
            let read_pos = (self.write_pos + self.buffer_l.len() - delay_samples) % self.buffer_l.len();
            let delayed = self.buffer_l[read_pos];

            self.buffer_l[self.write_pos] = *sample + delayed * self.feedback;
            self.write_pos = (self.write_pos + 1) % self.buffer_l.len();

            *sample = *sample * (1.0 - self.mix) + delayed * self.mix;
        }
    }

    fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.time = value.clamp(0.01, 2.0),
            1 => self.feedback = value.clamp(0.0, 0.9),
            2 => self.mix = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn param_count(&self) -> usize { 3 }
    fn param_name(&self, index: usize) -> &str {
        match index { 0 => "TIME", 1 => "FDBK", 2 => "MIX", _ => "" }
    }
    fn name(&self) -> &str { "DELAY" }
    fn set_bypass(&mut self, bypass: bool) { self.bypassed = bypass; }
    fn is_bypassed(&self) -> bool { self.bypassed }
}
