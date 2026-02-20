use crate::effects::Effect;

pub struct Distortion {
    drive: f32,
    mix: f32,
    bypassed: bool,
}

impl Distortion {
    pub fn new() -> Self {
        Self {
            drive: 2.0,
            mix: 0.5,
            bypassed: false,
        }
    }
}

impl Effect for Distortion {
    fn process(&mut self, input: &mut [f32]) {
        for sample in input.iter_mut() {
            let dry = *sample;
            // Tape saturation: normalized tanh waveshaping
            let driven = (*sample * self.drive).tanh() / self.drive.tanh();
            *sample = dry * (1.0 - self.mix) + driven * self.mix;
        }
    }

    fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.drive = value.clamp(1.0, 10.0),
            1 => self.mix = value.clamp(0.0, 1.0),
            _ => {}
        }
    }

    fn param_count(&self) -> usize { 2 }
    fn param_name(&self, index: usize) -> &str {
        match index { 0 => "DRIVE", 1 => "MIX", _ => "" }
    }
    fn name(&self) -> &str { "DIST" }
    fn set_bypass(&mut self, bypass: bool) { self.bypassed = bypass; }
    fn is_bypassed(&self) -> bool { self.bypassed }
}
