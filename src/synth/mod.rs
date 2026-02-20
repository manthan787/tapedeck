pub mod voice;
pub mod engines;

/// Trait for a synthesizer engine
pub trait SynthEngine: Send {
    fn note_on(&mut self, note: u8, velocity: f32);
    fn note_off(&mut self, note: u8);
    fn process(&mut self, output: &mut [f32]);
    fn set_param(&mut self, index: usize, value: f32);
    fn param_count(&self) -> usize;
    fn param_name(&self, index: usize) -> &str;
    fn name(&self) -> &str;
}
