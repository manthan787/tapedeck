pub mod reverb;
pub mod delay;
pub mod filter;
pub mod distortion;
pub mod chorus;

/// Trait for an audio effect
pub trait Effect: Send {
    fn process(&mut self, input: &mut [f32]);
    fn set_param(&mut self, index: usize, value: f32);
    fn param_count(&self) -> usize;
    fn param_name(&self, index: usize) -> &str;
    fn name(&self) -> &str;
    fn set_bypass(&mut self, bypass: bool);
    fn is_bypassed(&self) -> bool;
}

/// Chain of effects applied to a track
pub struct EffectChain {
    pub effects: Vec<Box<dyn Effect>>,
}

impl EffectChain {
    pub fn new() -> Self {
        Self { effects: vec![] }
    }

    pub fn process(&mut self, buffer: &mut [f32]) {
        for effect in &mut self.effects {
            if !effect.is_bypassed() {
                effect.process(buffer);
            }
        }
    }

    pub fn add(&mut self, effect: Box<dyn Effect>) {
        self.effects.push(effect);
    }
}
