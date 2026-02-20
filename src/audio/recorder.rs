use crate::audio::buffer::SharedBuffers;
use crate::constants::TRACK_SAMPLES;
use std::sync::atomic::Ordering;

/// Write an input sample to the armed track at the given position
pub fn write_sample(buffers: &mut SharedBuffers, track: usize, position: usize, sample: f32) {
    if position < TRACK_SAMPLES {
        buffers.tracks[track].data[position] = sample;
        let current_len = buffers.tracks[track].len.load(Ordering::Relaxed);
        if position >= current_len {
            buffers.tracks[track].len.store(position + 1, Ordering::Relaxed);
        }
    }
}
