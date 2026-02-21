use crate::constants::TRACK_COUNT;
use crate::messages::{RecordSource, TrackDisplay, TransportDisplay};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Tape,
    Synth,
    Drum,
    Mixer,
}

impl AppMode {
    pub fn next(self) -> Self {
        match self {
            AppMode::Tape => AppMode::Synth,
            AppMode::Synth => AppMode::Drum,
            AppMode::Drum => AppMode::Mixer,
            AppMode::Mixer => AppMode::Tape,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AppMode::Tape => "TAPE",
            AppMode::Synth => "SYNTH",
            AppMode::Drum => "DRUM",
            AppMode::Mixer => "MIXER",
        }
    }
}

pub struct AppState {
    pub mode: AppMode,
    pub selected_track: usize,
    pub transport: TransportDisplay,
    pub loop_enabled: bool,
    pub position: usize,
    pub track_displays: [TrackDisplay; TRACK_COUNT],
    pub levels: [f32; TRACK_COUNT],
    pub peaks: [f32; TRACK_COUNT],
    pub master_level: (f32, f32),
    pub should_quit: bool,
    /// Selected synth engine index
    pub synth_engine: usize,
    /// Synth parameter values
    pub synth_params: [f32; 4],
    /// Drum sequencer BPM
    pub bpm: f32,
    /// Selected drum instrument
    pub selected_instrument: usize,
    /// Drum patterns: [instrument][step]
    pub drum_patterns: [[bool; 16]; 6],
    /// Current sequencer step (for display)
    pub current_step: usize,
    /// Tape simulation enabled
    pub tape_sim_enabled: bool,
    /// Tape speed multiplier
    pub tape_speed: f32,
    /// Waveform data for display (downsampled per track)
    pub waveform_data: [Vec<f32>; TRACK_COUNT],
    /// Effect names per track per slot
    pub effect_names: [[String; 3]; TRACK_COUNT],
    /// Effect bypassed per track per slot
    pub effect_bypassed: [[bool; 3]; TRACK_COUNT],
    /// Recording source
    pub record_source: RecordSource,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Tape,
            selected_track: 0,
            transport: TransportDisplay::Stopped,
            loop_enabled: true,
            position: 0,
            track_displays: [TrackDisplay::default(); TRACK_COUNT],
            levels: [0.0; TRACK_COUNT],
            peaks: [0.0; TRACK_COUNT],
            master_level: (0.0, 0.0),
            should_quit: false,
            synth_engine: 0,
            synth_params: [0.5; 4],
            bpm: 120.0,
            selected_instrument: 0,
            drum_patterns: [[false; 16]; 6],
            current_step: 0,
            tape_sim_enabled: false,
            tape_speed: 1.0,
            waveform_data: [vec![], vec![], vec![], vec![]],
            effect_names: Default::default(),
            effect_bypassed: Default::default(),
            record_source: RecordSource::Internal,
        }
    }

    pub fn position_secs(&self) -> f64 {
        self.position as f64 / crate::constants::SAMPLE_RATE as f64
    }

    pub fn position_display(&self) -> String {
        let secs = self.position_secs();
        let mins = (secs / 60.0) as u32;
        let s = secs % 60.0;
        format!("{:02}:{:05.2}", mins, s)
    }
}
