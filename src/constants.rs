pub const SAMPLE_RATE: u32 = 44_100;
pub const BUFFER_SIZE: usize = 512;
pub const TRACK_COUNT: usize = 4;
/// Maximum recording duration in seconds
pub const MAX_DURATION_SECS: usize = 360;
/// Total samples per track (mono)
pub const TRACK_SAMPLES: usize = SAMPLE_RATE as usize * MAX_DURATION_SECS;
/// UI refresh rate target
pub const UI_FPS: u64 = 60;
/// Channel capacity for inter-thread messages
pub const CHANNEL_CAPACITY: usize = 1024;
