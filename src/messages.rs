/// Recording source selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordSource {
    Mic,
    Synth,
    Drum,
    All,
}

impl RecordSource {
    pub fn next(self) -> Self {
        match self {
            RecordSource::Mic => RecordSource::Synth,
            RecordSource::Synth => RecordSource::Drum,
            RecordSource::Drum => RecordSource::All,
            RecordSource::All => RecordSource::Mic,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RecordSource::Mic => "MIC",
            RecordSource::Synth => "SYNTH",
            RecordSource::Drum => "DRUM",
            RecordSource::All => "ALL",
        }
    }
}

/// Messages from UI thread → Control thread
#[derive(Debug, Clone)]
pub enum UiEvent {
    TogglePlayPause,
    StartRecord,
    StopTransport,
    CycleRecordSource,
    SelectTrack(usize),
    ArmTrack(usize),
    MuteTrack(usize),
    SoloTrack(usize),
    Seek(i64),
    CycleMode,
    SetLevel(usize, f32),
    SetPan(usize, f32),
    /// Synth note on: (note_number, velocity)
    NoteOn(u8, f32),
    /// Synth note off
    NoteOff(u8),
    /// Synth engine selection
    SelectEngine(usize),
    /// Parameter change (param_index, value)
    SetParam(usize, f32),
    /// Drum sequencer: toggle step (instrument, step)
    ToggleStep(usize, usize),
    /// Set BPM
    SetBpm(f32),
    /// Select drum instrument
    SelectInstrument(usize),
    /// Toggle tape simulation
    ToggleTapeSim,
    /// Set tape speed (0.5, 1.0, 2.0)
    SetTapeSpeed(f32),
    /// Toggle effect bypass (track, slot)
    ToggleEffect(usize, usize),
    /// Set effect parameter (track, slot, param, value)
    SetEffectParam(usize, usize, usize, f32),
    SaveProject,
    LoadProject(String),
    Quit,
}

/// Messages from Control thread → Audio thread
#[derive(Debug, Clone)]
pub enum AudioCmd {
    Play,
    Pause,
    Stop,
    Record(usize),
    StopRecord,
    Seek(usize),
    SetLevel(usize, f32),
    SetPan(usize, f32),
    SetMute(usize, bool),
    SetSolo(usize, bool),
    NoteOn(u8, f32),
    NoteOff(u8),
    SelectEngine(usize),
    SetParam(usize, f32),
    ToggleStep(usize, usize),
    SetBpm(f32),
    ToggleTapeSim,
    SetTapeSpeed(f32),
    ToggleEffect(usize, usize),
    SetEffectParam(usize, usize, usize, f32),
    SetRecordSource(RecordSource),
}

/// Messages from Audio thread → Control thread
#[derive(Debug, Clone)]
pub enum AudioMsg {
    Position(usize),
    Levels([f32; 4]),
    Peaks([f32; 4]),
    MasterLevel(f32, f32),
}

/// Messages from Control thread → UI thread
#[derive(Debug, Clone)]
pub enum UiUpdate {
    TransportState(TransportDisplay),
    Position(usize),
    Levels([f32; 4]),
    Peaks([f32; 4]),
    MasterLevel(f32, f32),
    TrackStates([TrackDisplay; 4]),
    Mode(super::app::AppMode),
}

#[derive(Debug, Clone, Copy)]
pub enum TransportDisplay {
    Stopped,
    Playing,
    Recording,
    Paused,
}

#[derive(Debug, Clone, Copy)]
pub struct TrackDisplay {
    pub armed: bool,
    pub muted: bool,
    pub solo: bool,
    pub level: f32,
    pub pan: f32,
}

impl Default for TrackDisplay {
    fn default() -> Self {
        Self {
            armed: false,
            muted: false,
            solo: false,
            level: 0.8,
            pan: 0.0,
        }
    }
}
