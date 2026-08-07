//! Gameplay Plugin
//!
//! Shared gameplay systems: AI, weapons, spawning, scoring, objectives,
//! abilities, maneuvers, campaign progression.
//!
//! Systems here may read simulation state but must not perform collision
//! detection or mutate presentation resources directly.

use bevy::prelude::*;

use crate::core::{
    CampaignState, CurrentStage, Difficulty, EndlessMode, GameProgress, GameSession, ItchMode,
    SelectedShip, ShipUnlocks,
};
use crate::entities::{DronePlugin, EnemyPlugin, EscortPlugin, PlayerPlugin, WingmanPlugin};
use crate::systems::{
    AbilityPlugin, BossPlugin, CampaignPlugin, ManeuverPlugin, ScoringPlugin, SpawningPlugin,
};

use combat_outcomes::{enemy_death_outcomes, player_damage_outcomes, player_death_outcome};

pub mod combat_outcomes;

/// Plugin that registers all shared gameplay systems.
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameProgress>()
            .init_resource::<Difficulty>()
            .init_resource::<SelectedShip>()
            .init_resource::<CurrentStage>()
            .init_resource::<ShipUnlocks>()
            .init_resource::<CampaignState>()
            .init_resource::<GameSession>()
            .init_resource::<EndlessMode>()
            .init_resource::<ItchMode>()
            .add_plugins((
                PlayerPlugin,
                EnemyPlugin,
                WingmanPlugin,
                DronePlugin,
                EscortPlugin,
                AbilityPlugin,
                SpawningPlugin,
                BossPlugin,
                ManeuverPlugin,
                CampaignPlugin,
                ScoringPlugin,
            ))
            .add_systems(
                FixedUpdate,
                (
                    enemy_death_outcomes,
                    player_damage_outcomes,
                    player_death_outcome,
                )
                    .after(crate::simulation::CollisionPhase::Resolution)
                    .run_if(
                        in_state(crate::core::GameState::Playing)
                            .or(in_state(crate::core::GameState::BossFight)),
                    ),
            );
    }
}
