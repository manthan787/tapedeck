use crate::effects::Effect;

pub struct Reverb {
    delay_lines: [Vec<f32>; 4],
    write_positions: [usize; 4],
    mix: f32,
    decay: f32,
    bypassed: bool,
}

impl Reverb {
    pub fn new() -> Self {
        // 4 delay lines with prime-number lengths for diffuse reverb
        let lengths = [1557, 1617, 1491, 1422];
        Self {
            delay_lines: [
                vec![0.0; lengths[0]],
                vec![0.0; lengths[1]],
                vec![0.0; lengths[2]],
                vec![0.0; lengths[3]],
            ],
            write_positions: [0; 4],
            mix: 0.3,
            decay: 0.6,
            bypassed: false,
        }
    }
}

impl Effect for Reverb {
    fn process(&mut self, input: &mut [f32]) {
        for sample in input.iter_mut() {
            let dry = *sample;
            let mut wet = 0.0f32;

            for i in 0..4 {
                let len = self.delay_lines[i].len();
                let read_pos = (self.write_positions[i] + 1) % len;
                let delayed = self.delay_lines[i][read_pos];
                wet += delayed * 0.25;

                self.delay_lines[i][self.write_positions[i]] = dry + delayed * self.decay;
                self.write_positions[i] = (self.write_positions[i] + 1) % len;
            }

            *sample = dry * (1.0 - self.mix) + wet * self.mix;
        }
    }

    fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.mix = value.clamp(0.0, 1.0),
            1 => self.decay = value.clamp(0.1, 0.95),
            _ => {}
        }
    }

    fn param_count(&self) -> usize { 2 }
    fn param_name(&self, index: usize) -> &str {
        match index { 0 => "MIX", 1 => "DECAY", _ => "" }
    }
    fn name(&self) -> &str { "REVERB" }
    fn set_bypass(&mut self, bypass: bool) { self.bypassed = bypass; }
    fn is_bypassed(&self) -> bool { self.bypassed }
}
