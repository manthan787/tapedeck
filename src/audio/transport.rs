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
}

impl Transport {
    pub fn new() -> Self {
        Self {
            state: TransportState::Stopped,
            position: 0,
            recording_track: None,
            max_position: 0,
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
        self.position = pos.min(TRACK_SAMPLES.saturating_sub(1));
    }

    pub fn advance(&mut self) -> bool {
        match self.state {
            TransportState::Playing | TransportState::Recording => {
                self.position += 1;
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
}
