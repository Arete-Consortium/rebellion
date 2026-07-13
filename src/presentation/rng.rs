//! Presentation RNG
//!
//! Unseeded randomness for cosmetic / visual effects.
//! `PresentationRng` is only ever accessed from `Update` systems.
//! Gameplay-critical randomness lives in `SimulationRng` (PR #4).

use bevy::prelude::*;
use rand::prelude::*;
use rand::rngs::StdRng;

/// Unseeded RNG for presentation-only randomness (particle jitter, screen shake,
/// damage number drift, chain bolt wobble).
///
/// # Invariants
/// - Only systems in `Update` may read/write this resource.
/// - Does NOT affect gameplay outcomes (score, damage, drops, enemy positions).
#[derive(Resource)]
pub struct PresentationRng {
    pub rng: StdRng,
}

impl Default for PresentationRng {
    fn default() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }
}

impl PresentationRng {
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
}
