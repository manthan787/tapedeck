use crate::constants::SAMPLE_RATE;
use crate::synth::SynthEngine;

const MAX_VOICES: usize = 8;

/// Karplus-Strong plucked string synthesis
pub struct StringSynth {
    voices: [StringVoice; MAX_VOICES],
    brightness: f32,
    damping: f32,
    attack: f32,
    decay: f32,
}

struct StringVoice {
    delay_line: Vec<f32>,
    write_pos: usize,
    active: bool,
    note: u8,
    envelope: f32,
    releasing: bool,
    prev_sample: f32,
}

impl Clone for StringVoice {
    fn clone(&self) -> Self {
        Self {
            delay_line: self.delay_line.clone(),
            write_pos: self.write_pos,
            active: self.active,
            note: self.note,
            envelope: self.envelope,
            releasing: self.releasing,
            prev_sample: self.prev_sample,
        }
    }
}

impl Default for StringVoice {
    fn default() -> Self {
        Self {
            delay_line: vec![0.0; 1024],
            write_pos: 0,
            active: false,
            note: 0,
            envelope: 0.0,
            releasing: false,
            prev_sample: 0.0,
        }
    }
}

impl StringSynth {
    pub fn new() -> Self {
        Self {
            voices: std::array::from_fn(|_| StringVoice::default()),
            brightness: 0.5,
            damping: 0.996,
            attack: 0.001,
            decay: 2.0,
        }
    }

    fn midi_to_freq(note: u8) -> f64 {
        440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0)
    }
}

impl SynthEngine for StringSynth {
    fn note_on(&mut self, note: u8, _velocity: f32) {
        let slot = self.voices.iter().position(|v| !v.active).unwrap_or(0);
        let freq = Self::midi_to_freq(note);
        let delay_len = (SAMPLE_RATE as f64 / freq) as usize;
        let delay_len = delay_len.max(2).min(4096);

        // Initialize delay line with noise burst (the "pluck")
        let mut delay_line = vec![0.0f32; delay_len];
        let mut rng_state = note as u32 * 1664525 + 1013904223;
        for sample in &mut delay_line {
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            *sample = (rng_state as f32 / u32::MAX as f32) * 2.0 - 1.0;
            *sample *= self.brightness;
        }

        self.voices[slot] = StringVoice {
            delay_line,
            write_pos: 0,
            active: true,
            note,
            envelope: 1.0,
            releasing: false,
            prev_sample: 0.0,
        };
    }

    fn note_off(&mut self, note: u8) {
        for v in &mut self.voices {
            if v.active && v.note == note {
                v.releasing = true;
            }
        }
    }

    fn process(&mut self, output: &mut [f32]) {
        let damping = self.damping;
        let decay_rate = 1.0 / (self.decay * SAMPLE_RATE as f32).max(1.0);

        for sample in output.iter_mut() {
            let mut sum = 0.0f32;
            for voice in &mut self.voices {
                if !voice.active {
                    continue;
                }

                if voice.releasing {
                    voice.envelope -= decay_rate;
                    if voice.envelope <= 0.0 {
                        voice.active = false;
                        continue;
                    }
                }

                let len = voice.delay_line.len();
                if len < 2 {
                    voice.active = false;
                    continue;
                }

                // Read from delay line
                let read_pos = voice.write_pos;
                let current = voice.delay_line[read_pos];

                // Average with previous sample (lowpass filter)
                let filtered = (current + voice.prev_sample) * 0.5 * damping;
                voice.prev_sample = current;

                // Write back
                voice.delay_line[voice.write_pos] = filtered;
                voice.write_pos = (voice.write_pos + 1) % len;

                sum += filtered * voice.envelope * 0.4;
            }
            *sample += sum;
        }
    }

    fn set_param(&mut self, index: usize, value: f32) {
        match index {
            0 => self.brightness = value.clamp(0.1, 1.0),
            1 => self.damping = 0.99 + value * 0.009, // 0.99 to 0.999
            2 => self.attack = value.clamp(0.001, 0.1),
            3 => self.decay = value.clamp(0.1, 10.0),
            _ => {}
        }
    }

    fn param_count(&self) -> usize { 4 }
    fn param_name(&self, index: usize) -> &str {
        match index {
            0 => "BRIGHT",
            1 => "DAMP",
            2 => "ATTACK",
            3 => "DECAY",
            _ => "",
        }
    }
    fn name(&self) -> &str { "STRING" }
}
