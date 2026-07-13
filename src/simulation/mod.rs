//! Simulation Plugin
//!
//! Authoritative gameplay systems: fixed-step physics, collision detection,
//! damage/health math, death resolution, spawn materialization.
//!
//! Per SIMULATION_CONTRACT.md, this plugin must never import presentation,
//! audio, UI, or campaign-authoring types.
//!
//! NOTE: collision.rs still contains presentation-side reactions (FX, scoring,
//! dialogue) — those will be extracted in PR #9. The registrations below
//! are transitional.

use bevy::prelude::*;

use crate::entities::{CollectiblePlugin, ProjectilePlugin};
use crate::systems::CollisionPlugin;

/// Plugin that registers all authoritative simulation systems.
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CollisionPlugin, ProjectilePlugin, CollectiblePlugin));
    }
}
