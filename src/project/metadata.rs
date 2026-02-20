use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ProjectMeta {
    pub name: String,
    pub bpm: f32,
    pub track_count: usize,
    pub sample_rate: u32,
    pub tracks: Vec<TrackMeta>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TrackMeta {
    pub index: usize,
    pub level: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    pub armed: bool,
    pub filename: String,
}

impl ProjectMeta {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bpm: 120.0,
            sample_rate: crate::constants::SAMPLE_RATE,
            track_count: crate::constants::TRACK_COUNT,
            tracks: (0..crate::constants::TRACK_COUNT)
                .map(|i| TrackMeta {
                    index: i,
                    level: 0.8,
                    pan: 0.0,
                    muted: false,
                    solo: false,
                    armed: false,
                    filename: format!("track_{}.wav", i + 1),
                })
                .collect(),
        }
    }
}
