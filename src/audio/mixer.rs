use crate::constants::TRACK_COUNT;

pub struct MixerState {
    pub levels: [f32; TRACK_COUNT],
    pub pans: [f32; TRACK_COUNT],
    pub mutes: [bool; TRACK_COUNT],
    pub solos: [bool; TRACK_COUNT],
}

impl MixerState {
    pub fn new() -> Self {
        Self {
            levels: [0.8; TRACK_COUNT],
            pans: [0.0; TRACK_COUNT],
            mutes: [false; TRACK_COUNT],
            solos: [false; TRACK_COUNT],
        }
    }

    /// Returns (left_gain, right_gain) for a given track
    pub fn track_gain(&self, track: usize) -> (f32, f32) {
        if self.mutes[track] {
            return (0.0, 0.0);
        }

        let any_solo = self.solos.iter().any(|&s| s);
        if any_solo && !self.solos[track] {
            return (0.0, 0.0);
        }

        let level = self.levels[track];
        let pan = self.pans[track]; // -1.0 to 1.0
        let left = level * (1.0 - pan.max(0.0));
        let right = level * (1.0 + pan.min(0.0));
        (left, right)
    }

    /// Mix track samples into stereo output
    pub fn mix(&self, track_samples: &[f32; TRACK_COUNT]) -> (f32, f32) {
        let mut left = 0.0f32;
        let mut right = 0.0f32;
        for i in 0..TRACK_COUNT {
            let (gl, gr) = self.track_gain(i);
            left += track_samples[i] * gl;
            right += track_samples[i] * gr;
        }
        (left, right)
    }
}
