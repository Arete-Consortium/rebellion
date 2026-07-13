//! Simulation Plugin
//!
//! Authoritative gameplay systems: fixed-step physics, collision detection,
//! damage/health math, death resolution, spawn materialization.
//!
//! Per SIMULATION_CONTRACT.md, this plugin must never import presentation,
//! audio, UI, or campaign-authoring types.

use bevy::prelude::*;

/// Plugin that registers all authoritative simulation systems.
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, _app: &mut App) {
        // Empty shell for Mission 1 — systems will migrate here in later PRs.
        // Intentionally no-op so existing gameplay is unchanged.
    }
}
