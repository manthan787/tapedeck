use crate::effects::Effect;

pub struct Filter {
    cutoff: f32,
    resonance: f32,
    mode: FilterMode,
    lp: f32,
    bp: f32,
    bypassed: bool,
}

#[derive(Clone, Copy)]
pub enum FilterMode {
    LowPass,
    HighPass,
    BandPass,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            cutoff: 0.5,
            resonance: 0.3,
            mode: FilterMode::LowPass,
            lp: 0.0,
            bp: 0.0,
            bypassed: false,
        }
    }
}

impl Effect for Filter {
    fn process(&mut self, input: &mut [f32]) {
        let f = (self.cutoff * self.cutoff).clamp(0.001, 0.99);
        let q = 1.0 - self.resonance.clamp(0.0, 0.95);

        for sample in input.iter_mut() {
            self.lp += f * self.bp;
            let hp = *sample - self.lp - q * self.bp;
            self.bp += f * hp;

            *sample = match self.mode {
                FilterMode::LowPass => self.lp,
                FilterMode::HighPass => hp,
                FilterMode::BandPass => self.bp,
            };
        }
    }

    fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.cutoff = value.clamp(0.01, 1.0),
            1 => self.resonance = value.clamp(0.0, 0.95),
            2 => {
                self.mode = if value < 0.33 {
                    FilterMode::LowPass
                } else if value < 0.66 {
                    FilterMode::HighPass
                } else {
                    FilterMode::BandPass
                };
            }
            _ => {}
        }
    }

    fn param_count(&self) -> usize { 3 }
    fn param_name(&self, index: usize) -> &str {
        match index { 0 => "CUTOFF", 1 => "RESO", 2 => "MODE", _ => "" }
    }
    fn name(&self) -> &str { "FILTER" }
    fn set_bypass(&mut self, bypass: bool) { self.bypassed = bypass; }
    fn is_bypassed(&self) -> bool { self.bypassed }
}
