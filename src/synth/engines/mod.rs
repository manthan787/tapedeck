pub mod sine;
pub mod saw;
pub mod fm;
pub mod string;
pub mod noise;

use crate::synth::SynthEngine;

pub fn create_engine(index: usize) -> Box<dyn SynthEngine> {
    match index {
        0 => Box::new(sine::SineSynth::new()),
        1 => Box::new(saw::SawSynth::new()),
        2 => Box::new(fm::FmSynth::new()),
        3 => Box::new(string::StringSynth::new()),
        4 => Box::new(noise::NoiseSynth::new()),
        _ => Box::new(sine::SineSynth::new()),
    }
}

pub const ENGINE_COUNT: usize = 5;
pub const ENGINE_NAMES: [&str; 5] = ["SINE", "SAW", "FM", "STRING", "NOISE"];
