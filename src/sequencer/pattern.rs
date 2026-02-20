/// 16-step pattern for a single instrument
#[derive(Clone)]
pub struct Pattern {
    pub steps: [bool; 16],
}

impl Pattern {
    pub fn new() -> Self {
        Self { steps: [false; 16] }
    }

    pub fn toggle(&mut self, step: usize) {
        if step < 16 {
            self.steps[step] = !self.steps[step];
        }
    }

    pub fn is_active(&self, step: usize) -> bool {
        step < 16 && self.steps[step]
    }
}

/// Collection of patterns for all instruments
pub struct PatternBank {
    pub patterns: Vec<Pattern>,
}

impl PatternBank {
    pub fn new(instrument_count: usize) -> Self {
        Self {
            patterns: (0..instrument_count).map(|_| Pattern::new()).collect(),
        }
    }
}
