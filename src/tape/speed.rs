/// Variable speed playback with cubic interpolation
pub struct SpeedControl {
    pub speed: f64,
    fractional_pos: f64,
}

impl SpeedControl {
    pub fn new() -> Self {
        Self {
            speed: 1.0,
            fractional_pos: 0.0,
        }
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.clamp(0.25, 4.0);
    }

    /// Advance position and return the fractional read position
    pub fn advance(&mut self, base_position: usize) -> f64 {
        self.fractional_pos = base_position as f64;
        self.fractional_pos
    }

    /// Read a sample with cubic interpolation from a buffer
    pub fn read_interpolated(&self, buffer: &[f32], position: f64) -> f32 {
        let len = buffer.len();
        if len < 4 {
            return 0.0;
        }

        let pos = position.max(0.0);
        let idx = pos as usize;
        let frac = pos - idx as f64;

        if idx + 2 >= len {
            return if idx < len { buffer[idx] } else { 0.0 };
        }

        let y0 = if idx > 0 { buffer[idx - 1] } else { buffer[0] };
        let y1 = buffer[idx];
        let y2 = buffer[idx + 1];
        let y3 = buffer[(idx + 2).min(len - 1)];

        // Cubic Hermite interpolation
        let frac = frac as f32;
        let a = -0.5 * y0 + 1.5 * y1 - 1.5 * y2 + 0.5 * y3;
        let b = y0 - 2.5 * y1 + 2.0 * y2 - 0.5 * y3;
        let c = -0.5 * y0 + 0.5 * y2;
        let d = y1;

        a * frac * frac * frac + b * frac * frac + c * frac + d
    }
}
