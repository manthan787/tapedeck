use crate::constants::{TRACK_COUNT, TRACK_SAMPLES};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Per-track audio buffer - fixed size, lock-free position tracking
pub struct TrackBuffer {
    pub data: Vec<f32>,
    pub len: AtomicUsize,
}

impl TrackBuffer {
    pub fn new() -> Self {
        Self {
            data: vec![0.0; TRACK_SAMPLES],
            len: AtomicUsize::new(0),
        }
    }

    pub fn read(&self, pos: usize) -> f32 {
        if pos < self.len.load(Ordering::Relaxed) {
            self.data[pos]
        } else {
            0.0
        }
    }

    pub fn has_data(&self) -> bool {
        self.len.load(Ordering::Relaxed) > 0
    }

    pub fn sample_count(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }
}

/// Shared track buffers accessible from audio and UI threads
pub struct SharedBuffers {
    pub tracks: [TrackBuffer; TRACK_COUNT],
}

impl SharedBuffers {
    pub fn new() -> Self {
        Self {
            tracks: [
                TrackBuffer::new(),
                TrackBuffer::new(),
                TrackBuffer::new(),
                TrackBuffer::new(),
            ],
        }
    }
}

/// Downsampled waveform data for UI display
pub fn downsample_track(buffer: &TrackBuffer, width: usize) -> Vec<f32> {
    let len = buffer.sample_count();
    if len == 0 || width == 0 {
        return vec![0.0; width];
    }
    let samples_per_pixel = len / width.max(1);
    if samples_per_pixel == 0 {
        return vec![0.0; width];
    }
    (0..width)
        .map(|i| {
            let start = i * samples_per_pixel;
            let end = (start + samples_per_pixel).min(len);
            let mut peak: f32 = 0.0;
            for j in start..end {
                peak = peak.max(buffer.data[j].abs());
            }
            peak
        })
        .collect()
}
