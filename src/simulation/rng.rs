//! Simulation RNG
//!
//! Seeded, deterministic randomness for gameplay-critical systems.
//! `SimulationRng` is only ever accessed from `FixedUpdate` systems.
//! Cosmetic / presentation randomness lives in `PresentationRng` (PR #5).

use bevy::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

/// Default seed used when no mission seed is configured (e.g. tests).
pub const DEFAULT_MISSION_SEED: u64 = 42;

/// Mission-level seed set from menu / campaign select.
/// Persisted in save files so replays are deterministic.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionSeed(pub u64);

impl Default for MissionSeed {
    fn default() -> Self {
        Self(DEFAULT_MISSION_SEED)
    }
}

/// Seeded RNG for simulation-critical randomness (crits, drops, spawn offsets).
///
/// # Invariants
/// - Only systems in `FixedUpdate` may read/write this resource.
/// - Same `MissionSeed` + same input → same RNG sequence.
#[derive(Resource)]
pub struct SimulationRng {
    pub rng: StdRng,
}

impl SimulationRng {
    /// Create from a mission seed.
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Reset RNG to the current `MissionSeed`.
    #[allow(dead_code)]
    pub fn reseed(&mut self, seed: u64) {
        self.rng = StdRng::seed_from_u64(seed);
    }

    /// Generate a random `f32` in the range `[0.0, 1.0)`.
    pub fn f32(&mut self) -> f32 {
        self.rng.gen::<f32>()
    }

    /// Generate a random `f32` in the range `[min, max)`.
    #[allow(dead_code)]
    pub fn f32_range(&mut self, min: f32, max: f32) -> f32 {
        self.rng.gen_range(min..max)
    }

    /// Generate a random `usize` in the range `[0, bound)`.
    #[allow(dead_code)]
    pub fn usize(&mut self, bound: usize) -> usize {
        self.rng.gen_range(0..bound)
    }

    /// Generate a random `u32` in the range `[min, max)`.
    #[allow(dead_code)]
    pub fn u32_range(&mut self, min: u32, max: u32) -> u32 {
        self.rng.gen_range(min..max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut a = SimulationRng::from_seed(123);
        let mut b = SimulationRng::from_seed(123);
        for _ in 0..100 {
            assert_eq!(a.f32(), b.f32());
        }
    }

    #[test]
    fn different_seeds_produce_different_first_value() {
        let a = SimulationRng::from_seed(1).f32();
        let b = SimulationRng::from_seed(2).f32();
        assert_ne!(a, b);
    }
}
