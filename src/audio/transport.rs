use crate::constants::TRACK_SAMPLES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    Stopped,
    Playing,
    Recording,
    Paused,
}

pub struct Transport {
    pub state: TransportState,
    pub position: usize,
    pub recording_track: Option<usize>,
    /// Maximum sample position reached across all tracks
    pub max_position: usize,
    loop_enabled: bool,
    loop_end: Option<usize>,
}

impl Transport {
    pub fn new() -> Self {
        Self {
            state: TransportState::Stopped,
            position: 0,
            recording_track: None,
            max_position: 0,
            loop_enabled: true,
            loop_end: None,
        }
    }

    pub fn play(&mut self) {
        match self.state {
            TransportState::Stopped | TransportState::Paused => {
                self.state = TransportState::Playing;
            }
            TransportState::Recording => {
                // Keep recording
            }
            TransportState::Playing => {}
        }
    }

    pub fn pause(&mut self) {
        match self.state {
            TransportState::Playing => {
                self.state = TransportState::Paused;
            }
            TransportState::Paused => {
                self.state = TransportState::Playing;
            }
            _ => {}
        }
    }

    pub fn stop(&mut self) {
        self.state = TransportState::Stopped;
        self.recording_track = None;
        self.position = 0;
    }

    pub fn record(&mut self, track: usize) {
        self.state = TransportState::Recording;
        self.recording_track = Some(track);
    }

    pub fn stop_record(&mut self) {
        self.recording_track = None;
        self.state = TransportState::Playing;
    }

    pub fn seek(&mut self, pos: usize) {
        let max_pos = if self.loop_enabled {
            self.loop_end.unwrap_or(TRACK_SAMPLES).saturating_sub(1)
        } else {
            TRACK_SAMPLES.saturating_sub(1)
        };
        self.position = pos.min(max_pos);
    }

    pub fn advance(&mut self) -> bool {
        match self.state {
            TransportState::Playing | TransportState::Recording => {
                self.position += 1;
                if self.loop_enabled {
                    if let Some(loop_end) = self.loop_end {
                        if loop_end > 0 && self.position >= loop_end {
                            self.position = 0;
                        }
                    }
                }
                if self.position > self.max_position {
                    self.max_position = self.position;
                }
                true
            }
            _ => false,
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.state, TransportState::Playing | TransportState::Recording)
    }

    pub fn set_loop_enabled(&mut self, enabled: bool) {
        self.loop_enabled = enabled;
        if !enabled {
            self.loop_end = None;
        }
    }

    pub fn loop_enabled(&self) -> bool {
        self.loop_enabled
    }

    pub fn set_loop_end(&mut self, loop_end: Option<usize>) {
        if self.loop_enabled {
            self.loop_end = loop_end.filter(|&v| v > 0);
        } else {
            self.loop_end = None;
        }
    }

    pub fn loop_end(&self) -> Option<usize> {
        self.loop_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seek_clamps_to_track_bounds() {
        let mut transport = Transport::new();
        transport.seek(TRACK_SAMPLES + 10_000);
        assert_eq!(transport.position, TRACK_SAMPLES - 1);
    }

    #[test]
    fn seek_keeps_valid_positions() {
        let mut transport = Transport::new();
        transport.seek(12_345);
        assert_eq!(transport.position, 12_345);
    }

    #[test]
    fn advance_wraps_at_loop_end_when_enabled() {
        let mut transport = Transport::new();
        transport.set_loop_enabled(true);
        transport.set_loop_end(Some(4));
        transport.play();
        for _ in 0..4 {
            transport.advance();
        }
        assert_eq!(transport.position, 0);
    }

    #[test]
    fn advance_does_not_wrap_when_loop_disabled() {
        let mut transport = Transport::new();
        transport.set_loop_enabled(false);
        transport.set_loop_end(Some(4));
        transport.play();
        for _ in 0..4 {
            transport.advance();
        }
        assert_eq!(transport.position, 4);
    }
}
