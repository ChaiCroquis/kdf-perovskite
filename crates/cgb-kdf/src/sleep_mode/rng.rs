//! Simple deterministic random number generator

/// Simple deterministic RNG
pub(super) struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub(super) fn new(seed: u64) -> Self {
        Self {
            state: seed.wrapping_add(1),
        }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    pub(super) fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    pub(super) fn next_usize(&mut self) -> usize {
        self.next() as usize
    }
}
