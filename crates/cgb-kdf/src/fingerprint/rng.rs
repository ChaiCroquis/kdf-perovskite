//! Deterministic RNG backed by ChaCha8 for reproducible fingerprinting.
//!
//! Replaces a hand-rolled xorshift64. Two reasons:
//!   1. xorshift64 emits an all-zero stream from seed=0, which any caller
//!      could trigger by accident.
//!   2. `(u64 as f64) / u64::MAX` biases the upper end toward 1.0; ChaCha8's
//!      `gen::<f64>()` returns a uniform value in [0, 1) by construction.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub(crate) struct SimpleRng {
    inner: ChaCha8Rng,
}

impl SimpleRng {
    pub(crate) fn new(seed: u64) -> Self {
        Self {
            inner: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub(crate) fn next_f64(&mut self) -> f64 {
        self.inner.r#gen::<f64>()
    }

    pub(crate) fn next_usize(&mut self) -> usize {
        self.inner.r#gen::<usize>()
    }
}
