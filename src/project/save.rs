use std::path::Path;
use std::sync::atomic::Ordering;

use crate::audio::buffer::SharedBuffers;
use crate::constants::{SAMPLE_RATE, TRACK_COUNT};
use crate::project::metadata::ProjectMeta;

pub fn save_project(
    dir: &Path,
    meta: &ProjectMeta,
    buffers: &SharedBuffers,
) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dir)?;

    // Save each track as WAV
    for i in 0..TRACK_COUNT {
        let track = &buffers.tracks[i];
        let len = track.len.load(Ordering::Relaxed);
        if len == 0 {
            continue;
        }

        let path = dir.join(&meta.tracks[i].filename);
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;
        for j in 0..len {
            writer.write_sample(track.data[j])?;
        }
        writer.finalize()?;
    }

    // Save metadata
    let meta_path = dir.join("meta.json");
    let json = serde_json::to_string_pretty(meta)?;
    std::fs::write(meta_path, json)?;

    Ok(())
}
