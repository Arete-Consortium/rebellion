//! Gameplay Plugin
//!
//! Shared gameplay systems: AI, weapons, spawning, scoring, objectives,
//! abilities, maneuvers, campaign progression.
//!
//! Systems here may read simulation state but must not perform collision
//! detection or mutate presentation resources directly.

use bevy::prelude::*;

/// Plugin that registers all shared gameplay systems.
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, _app: &mut App) {
        // Empty shell for Mission 1 — systems will migrate here in later PRs.
    }
}
