use std::path::Path;
use std::sync::atomic::Ordering;

use crate::audio::buffer::SharedBuffers;
use crate::constants::TRACK_COUNT;
use crate::project::metadata::ProjectMeta;
use hound::{SampleFormat, WavReader};

pub fn load_project(
    dir: &Path,
    buffers: &mut SharedBuffers,
) -> Result<ProjectMeta, Box<dyn std::error::Error>> {
    let meta_path = dir.join("meta.json");
    let json = std::fs::read_to_string(meta_path)?;
    let meta: ProjectMeta = serde_json::from_str(&json)?;

    // Reset all track lengths first so missing files don't retain stale audio.
    for track in &mut buffers.tracks {
        track.len.store(0, Ordering::Relaxed);
    }

    // Load each track from WAV
    for i in 0..TRACK_COUNT.min(meta.tracks.len()) {
        let path = dir.join(&meta.tracks[i].filename);
        if !path.exists() {
            continue;
        }

        let samples = read_wav_mono_f32(&path)?;

        let len = samples.len().min(buffers.tracks[i].data.len());
        buffers.tracks[i].data[..len].copy_from_slice(&samples[..len]);
        buffers.tracks[i].len.store(len, Ordering::Relaxed);
    }

    Ok(meta)
}

fn read_wav_mono_f32(path: &Path) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut reader = WavReader::open(path)?;
    let spec = reader.spec();

    let samples = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Float, 32) => reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()?,
        (SampleFormat::Int, 8) => reader
            .samples::<i8>()
            .map(|s| s.map(|v| v as f32 / i8::MAX as f32))
            .collect::<Result<Vec<_>, _>>()?,
        (SampleFormat::Int, 16) => reader
            .samples::<i16>()
            .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
            .collect::<Result<Vec<_>, _>>()?,
        (SampleFormat::Int, 24 | 32) => {
            let denom = (1_i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / denom))
                .collect::<Result<Vec<_>, _>>()?
        }
        _ => {
            return Err(
                format!(
                    "Unsupported WAV format in {} ({}-bit {:?})",
                    path.display(),
                    spec.bits_per_sample,
                    spec.sample_format
                )
                .into(),
            )
        }
    };

    if spec.channels == 1 {
        return Ok(samples);
    }

    // Downmix interleaved multichannel audio to mono.
    let ch = spec.channels as usize;
    let mut mono = Vec::with_capacity(samples.len() / ch);
    for frame in samples.chunks_exact(ch) {
        mono.push(frame.iter().copied().sum::<f32>() / ch as f32);
    }
    Ok(mono)
}
