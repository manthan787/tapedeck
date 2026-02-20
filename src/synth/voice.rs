/// Polyphonic voice allocator with 8 voices and oldest-note stealing
pub struct VoiceAllocator {
    voices: Vec<Voice>,
    max_voices: usize,
}

struct Voice {
    note: u8,
    active: bool,
    age: u64,
}

impl VoiceAllocator {
    pub fn new(max_voices: usize) -> Self {
        let voices = (0..max_voices)
            .map(|_| Voice {
                note: 0,
                active: false,
                age: 0,
            })
            .collect();
        Self { voices, max_voices }
    }

    /// Allocate a voice for the given note, returns voice index
    pub fn note_on(&mut self, note: u8) -> usize {
        // Check if this note is already playing
        if let Some(idx) = self.voices.iter().position(|v| v.active && v.note == note) {
            self.voices[idx].age = self.current_age();
            return idx;
        }

        // Find a free voice
        if let Some(idx) = self.voices.iter().position(|v| !v.active) {
            self.voices[idx].note = note;
            self.voices[idx].active = true;
            self.voices[idx].age = self.current_age();
            return idx;
        }

        // Steal oldest voice
        let oldest = self
            .voices
            .iter()
            .enumerate()
            .min_by_key(|(_, v)| v.age)
            .map(|(i, _)| i)
            .unwrap_or(0);

        self.voices[oldest].note = note;
        self.voices[oldest].active = true;
        self.voices[oldest].age = self.current_age();
        oldest
    }

    /// Release a voice for the given note, returns voice index if found
    pub fn note_off(&mut self, note: u8) -> Option<usize> {
        if let Some(idx) = self.voices.iter().position(|v| v.active && v.note == note) {
            self.voices[idx].active = false;
            Some(idx)
        } else {
            None
        }
    }

    pub fn active_voices(&self) -> impl Iterator<Item = (usize, u8)> + '_ {
        self.voices
            .iter()
            .enumerate()
            .filter(|(_, v)| v.active)
            .map(|(i, v)| (i, v.note))
    }

    fn current_age(&self) -> u64 {
        self.voices.iter().map(|v| v.age).max().unwrap_or(0) + 1
    }
}
