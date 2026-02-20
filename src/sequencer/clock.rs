use crate::constants::SAMPLE_RATE;

/// BPM clock that derives step timing from sample position
pub struct SequencerClock {
    bpm: f32,
    last_step: usize,
}

impl SequencerClock {
    pub fn new(bpm: f32) -> Self {
        Self {
            bpm,
            last_step: usize::MAX,
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.clamp(40.0, 300.0);
    }

    pub fn bpm(&self) -> f32 {
        self.bpm
    }

    /// Given a sample position, return the current 16th-note step (0-15)
    /// and whether we just advanced to a new step
    pub fn tick(&mut self, sample_position: usize) -> (usize, bool) {
        let samples_per_beat = (SAMPLE_RATE as f32 * 60.0 / self.bpm) as usize;
        let samples_per_step = samples_per_beat / 4; // 16th notes
        let step = (sample_position / samples_per_step.max(1)) % 16;
        let new_step = step != self.last_step;
        self.last_step = step;
        (step, new_step)
    }

    pub fn reset(&mut self) {
        self.last_step = usize::MAX;
    }
}
