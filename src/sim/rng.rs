//! Reproducible random number generator — xorshift64 implementation.
//!
//! Guarantees game determinism: with the same seed and the same sequence of
//! operations, every random call produces identical results.
//! Records the type and result of every random call for replay verification.
//!
//! # Example
//!
//! ```rust
//! use orange_stone::sim::rng::GameRng;
//!
//! // Same seed produces the same random sequence
//! let mut rng1 = GameRng::new(42);
//! let mut rng2 = GameRng::new(42);
//! let seq1: Vec<u32> = (0..5).map(|_| rng1.next_u32()).collect();
//! let seq2: Vec<u32> = (0..5).map(|_| rng2.next_u32()).collect();
//! assert_eq!(seq1, seq2);
//! ```
use serde::{Deserialize, Serialize};

/// Record of a random call — used for replay verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RngCall {
    /// A `next_u32` call
    NextU32 {
        /// Internal RNG state at call time
        state_before: u64,
        /// The returned result
        result: u32,
    },
    /// A `next_usize` call
    NextUsize {
        /// Internal RNG state at call time
        state_before: u64,
        /// The `max` argument passed in
        max: usize,
        /// The returned result
        result: usize,
    },
}

/// Reproducible random number generator.
///
/// Based on the xorshift64 algorithm, with the state stored in a single `u64`.
/// Records every random call for replay verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRng {
    /// Current internal state
    state: u64,
    /// Initial seed
    seed: u64,
    /// Record of all random calls (for replay)
    pub calls: Vec<RngCall>,
}

impl GameRng {
    /// Creates a new random number generator with the given seed.
    ///
    /// The seed must not be 0 (xorshift requires a non-zero state).
    #[must_use]
    pub fn new(seed: u64) -> Self {
        // xorshift requires a non-zero state
        let state = if seed == 0 {
            0xDEAD_BEEF_CAFE_BABE
        } else {
            seed
        };
        Self {
            state,
            seed,
            calls: Vec::new(),
        }
    }

    /// Returns the seed used at initialization.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Generates the next `u32` random number.
    ///
    /// Uses the xorshift64 algorithm.
    pub fn next_u32(&mut self) -> u32 {
        let state_before = self.state;
        // xorshift64
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        let result = (x >> 32) as u32 ^ x as u32;
        self.calls.push(RngCall::NextU32 {
            state_before,
            result,
        });
        result
    }

    /// Generates a random `usize` in the range `[0, max)`.
    ///
    /// # Panics
    ///
    /// If `max == 0`.
    pub fn next_usize(&mut self, max: usize) -> usize {
        assert!(max > 0, "max must be > 0, got {max}");
        let state_before = self.state;
        // Use rejection sampling to avoid bias
        let mask = u32::MAX;
        let limit = mask - (mask % max as u32);
        loop {
            let val = self.next_u32();
            if val < limit {
                let result = (val as usize) % max;
                // Overwrite the automatic record from next_u32 (we want a more precise RngCall)
                self.calls.pop();
                self.calls.push(RngCall::NextUsize {
                    state_before,
                    max,
                    result,
                });
                return result;
            }
        }
    }

    /// Resets the RNG to its initial state (for replay).
    pub fn reset(&mut self) {
        self.state = if self.seed == 0 {
            0xDEAD_BEEF_CAFE_BABE
        } else {
            self.seed
        };
        self.calls.clear();
    }

    /// Returns the number of calls in this RNG session.
    #[must_use]
    pub fn call_count(&self) -> usize {
        self.calls.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(42);

        let seq1: Vec<u32> = (0..100).map(|_| rng1.next_u32()).collect();
        let seq2: Vec<u32> = (0..100).map(|_| rng2.next_u32()).collect();

        assert_eq!(seq1, seq2);
    }

    #[test]
    fn different_seeds_produce_different_sequences() {
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(99);

        let seq1: Vec<u32> = (0..10).map(|_| rng1.next_u32()).collect();
        let seq2: Vec<u32> = (0..10).map(|_| rng2.next_u32()).collect();

        assert_ne!(seq1, seq2);
    }

    #[test]
    fn zero_seed_is_handled() {
        let rng = GameRng::new(0);
        assert_eq!(rng.seed(), 0);
        // Should not panic
    }

    #[test]
    fn calls_are_recorded() {
        let mut rng = GameRng::new(1);
        rng.next_u32();
        rng.next_usize(10);

        assert_eq!(rng.call_count(), 2);
        assert!(matches!(rng.calls[0], RngCall::NextU32 { .. }));
        assert!(matches!(rng.calls[1], RngCall::NextUsize { max: 10, .. }));
    }

    #[test]
    fn reset_restores_state() {
        let mut rng = GameRng::new(77);
        let first = rng.next_u32();
        rng.next_u32();
        rng.next_u32();
        rng.reset();
        assert_eq!(rng.next_u32(), first);
        assert_eq!(rng.call_count(), 1);
    }

    #[test]
    fn next_usize_in_range() {
        let mut rng = GameRng::new(123);
        for _ in 0..1000 {
            let val = rng.next_usize(5);
            assert!(val < 5, "value {val} should be < 5");
        }
    }

    #[test]
    #[should_panic(expected = "max must be > 0")]
    fn next_usize_zero_panics() {
        let mut rng = GameRng::new(1);
        rng.next_usize(0);
    }
}
