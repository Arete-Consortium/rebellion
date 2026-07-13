//! Simulation Plugin
//!
//! Authoritative gameplay systems: fixed-step physics, collision detection,
//! damage/health math, death resolution, spawn materialization.
//!
//! Per SIMULATION_CONTRACT.md, this plugin must never import presentation,
//! audio, UI, or campaign-authoring types.

use bevy::prelude::*;

use crate::entities::{CollectiblePlugin, ProjectilePlugin};
use crate::systems::collision::SpatialGrid;
use crate::systems::CollisionPlugin;

use detect_collisions::{
    detect_enemy_projectile_hits, detect_player_projectile_hits, update_spatial_grid,
};
use resolve_damage::{
    enrich_contacts, resolve_enemy_projectile_damage, resolve_player_projectile_damage,
};
use resolve_deaths::resolve_enemy_deaths;
use sim_id::assign_sim_ids;

pub mod detect_collisions;
pub mod fixed_step;
pub mod resolve_damage;
pub mod resolve_deaths;
pub mod rng;
pub mod sim_id;

pub use fixed_step::{SimSet, FIXED_TIMESTEP_SECS};
pub use rng::{MissionSeed, SimulationRng, DEFAULT_MISSION_SEED};
pub use sim_id::SimIdGenerator;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CollisionPhase {
    Detection,
    Resolution,
}

/// Plugin that registers all authoritative simulation systems.
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialGrid::new())
            .init_resource::<MissionSeed>()
            .insert_resource(SimulationRng::from_seed(DEFAULT_MISSION_SEED))
            .init_resource::<SimIdGenerator>()
            .configure_sets(
                FixedUpdate,
                (CollisionPhase::Detection, CollisionPhase::Resolution).chain(),
            )
            .add_systems(
                FixedUpdate,
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
                FixedUpdate,
                (
                    enrich_contacts,
                    resolve_player_projectile_damage,
                    resolve_enemy_projectile_damage,
                    resolve_enemy_deaths,
                )
                    .chain()
                    .in_set(CollisionPhase::Resolution)
                    .run_if(in_state(crate::core::GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                assign_sim_ids
                    .after(CollisionPhase::Resolution)
                    .run_if(in_state(crate::core::GameState::Playing)),
            )
            .add_plugins((CollisionPlugin, ProjectilePlugin, CollectiblePlugin));
    }
}
