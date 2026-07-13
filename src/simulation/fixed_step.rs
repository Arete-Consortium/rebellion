//! Fixed-step simulation configuration
//!
//! Defines the fixed timestep used for authoritative gameplay systems
//! and system sets for organizing simulation, gameplay, and presentation.

use bevy::prelude::*;

/// Fixed timestep for all authoritative systems (60 Hz).
pub const FIXED_TIMESTEP_SECS: f64 = 1.0 / 60.0;

/// System sets for organizing game systems by domain.
/// These enforce ordering between simulation, gameplay, and presentation
/// regardless of which schedule they run in.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    /// Authoritative simulation systems (physics, collision, damage).
    Simulation,
    /// Gameplay systems (AI, scoring, spawning, state transitions).
    Gameplay,
    /// Presentation systems (FX, audio, UI, camera).
    Presentation,
}
