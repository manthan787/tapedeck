use std::path::Path;
use std::sync::atomic::Ordering;

use crate::audio::buffer::SharedBuffers;
use crate::constants::TRACK_COUNT;
use crate::project::metadata::ProjectMeta;

pub fn load_project(
    dir: &Path,
    buffers: &mut SharedBuffers,
) -> Result<ProjectMeta, Box<dyn std::error::Error>> {
    let meta_path = dir.join("meta.json");
    let json = std::fs::read_to_string(meta_path)?;
    let meta: ProjectMeta = serde_json::from_str(&json)?;

    // Load each track from WAV
    for i in 0..TRACK_COUNT.min(meta.tracks.len()) {
        let path = dir.join(&meta.tracks[i].filename);
        if !path.exists() {
            continue;
        }

        let mut reader = hound::WavReader::open(path)?;
        let samples: Vec<f32> = reader
            .samples::<f32>()
            .filter_map(|s| s.ok())
            .collect();

        let len = samples.len().min(buffers.tracks[i].data.len());
        buffers.tracks[i].data[..len].copy_from_slice(&samples[..len]);
        buffers.tracks[i].len.store(len, Ordering::Relaxed);
    }

    Ok(meta)
}
