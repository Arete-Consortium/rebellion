//! Simulation Plugin
//!
//! Authoritative gameplay systems: fixed-step physics, collision detection,
//! damage/health math, death resolution, spawn materialization.
//!
//! Per SIMULATION_CONTRACT.md, this plugin must never import presentation,
//! audio, UI, or campaign-authoring types.

use bevy::prelude::*;

use crate::core::GameState;
use crate::entities::{CollectiblePlugin, ProjectilePlugin};
use crate::entities::environment::{
    tick_contact_cooldowns, EnvironmentDamageAppliedEvent, EnvironmentDestroyedEvent,
    PlayerEnvironmentContact, ProjectileEnvironmentContact,
};
use crate::systems::collision::SpatialGrid;
use crate::systems::CollisionPlugin;

use detect_collisions::{
    detect_enemy_projectile_environment_hits, detect_enemy_projectile_hits,
    detect_player_environment_contacts, detect_player_projectile_environment_hits,
    detect_player_projectile_hits, update_spatial_grid,
};
use resolve_damage::{
    enrich_contacts, resolve_enemy_projectile_damage, resolve_player_environment_contacts,
    resolve_player_projectile_damage, resolve_projectile_environment_contacts,
};
use resolve_deaths::resolve_enemy_deaths;
use sim_id::assign_sim_ids;
use state_hash::{compute_state_hash_system, SimStateHash};

pub mod detect_collisions;
pub mod fixed_step;
pub mod resolve_damage;
pub mod resolve_deaths;
pub mod rng;
pub mod sim_id;
pub mod state_hash;

pub use fixed_step::{SimSet, FIXED_TIMESTEP_SECS};
pub use rng::{MissionSeed, SimulationRng, DEFAULT_MISSION_SEED};
pub use sim_id::SimIdGenerator;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum CollisionPhase {
    Detection,
    Resolution,
}

/// Run condition: simulation runs during active gameplay and boss fights.
fn simulation_active(state: Res<State<GameState>>) -> bool {
    matches!(*state.get(), GameState::Playing | GameState::BossFight)
}

/// Plugin that registers all authoritative simulation systems.
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialGrid::new())
            .init_resource::<MissionSeed>()
            .insert_resource(SimulationRng::from_seed(DEFAULT_MISSION_SEED))
            .init_resource::<SimIdGenerator>()
            .init_resource::<SimStateHash>()
            .add_event::<PlayerEnvironmentContact>()
            .add_event::<ProjectileEnvironmentContact>()
            .add_event::<EnvironmentDamageAppliedEvent>()
            .add_event::<EnvironmentDestroyedEvent>()
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
                    detect_player_environment_contacts,
                    detect_player_projectile_environment_hits,
                    detect_enemy_projectile_environment_hits,
                )
                    .chain()
                    .in_set(CollisionPhase::Detection)
                    .run_if(simulation_active),
            )
            .add_systems(
                FixedUpdate,
                (
                    enrich_contacts,
                    resolve_player_projectile_damage,
                    resolve_enemy_projectile_damage,
                    resolve_player_environment_contacts,
                    resolve_projectile_environment_contacts,
                    resolve_enemy_deaths,
                )
                    .chain()
                    .in_set(CollisionPhase::Resolution)
                    .run_if(simulation_active),
            )
            .add_systems(
                FixedUpdate,
                assign_sim_ids
                    .after(CollisionPhase::Resolution)
                    .run_if(simulation_active),
            )
            .add_systems(
                FixedUpdate,
                compute_state_hash_system
                    .after(assign_sim_ids)
                    .run_if(simulation_active),
            )
            .add_systems(
                FixedUpdate,
                tick_contact_cooldowns
                    .after(compute_state_hash_system)
                    .run_if(simulation_active),
            )
            .add_plugins((CollisionPlugin, ProjectilePlugin, CollectiblePlugin));
    }
}
