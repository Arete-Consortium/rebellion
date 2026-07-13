//! Simulation Plugin
//!
//! Authoritative gameplay systems: fixed-step physics, collision detection,
//! damage/health math, death resolution, spawn materialization.
//!
//! Per SIMULATION_CONTRACT.md, this plugin must never import presentation,
//! audio, UI, or campaign-authoring types.
//!
//! NOTE: collision.rs still contains presentation-side reactions (FX, scoring,
//! dialogue) — those will be extracted in later sub-PRs of PR #9. The
//! registrations below are transitional.

use bevy::prelude::*;

use crate::entities::{CollectiblePlugin, ProjectilePlugin};
use crate::systems::collision::{
    enemy_projectile_player_collision, player_projectile_enemy_collision, tick_chain_bolts,
    SpatialGrid,
};
use crate::systems::CollisionPlugin;

use detect_collisions::{
    detect_enemy_projectile_hits, detect_player_projectile_hits, update_spatial_grid,
};

pub mod detect_collisions;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum CollisionPhase {
    Detection,
    Resolution,
}

/// Plugin that registers all authoritative simulation systems.
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialGrid::new())
            .configure_sets(
                Update,
                (CollisionPhase::Detection, CollisionPhase::Resolution).chain(),
            )
            .add_systems(
                Update,
                (
                    update_spatial_grid,
                    detect_player_projectile_hits,
                    detect_enemy_projectile_hits,
                )
                    .chain()
                    .in_set(CollisionPhase::Detection)
                    .run_if(in_state(crate::core::GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    player_projectile_enemy_collision,
                    enemy_projectile_player_collision,
                    tick_chain_bolts,
                )
                    .chain()
                    .in_set(CollisionPhase::Resolution)
                    .run_if(in_state(crate::core::GameState::Playing)),
            )
            .add_plugins((CollisionPlugin, ProjectilePlugin, CollectiblePlugin));
    }
}
