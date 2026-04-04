//! Enemy Entities
//!
//! All enemy ship types, AI behaviors, and wave spawning.

mod ai;
mod faction;
mod spawn;
mod systems;
mod types;

// Re-export all public types
pub use ai::PlayerTracker;
pub use faction::get_ship_rotation_correction;
pub use spawn::*;
pub use types::*;

use ai::*;
use systems::*;

use crate::core::*;
use bevy::prelude::*;

/// Enemy plugin
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerTracker>().add_systems(
            Update,
            (
                // Ordered pipeline: track player -> compute awareness -> move -> shoot
                (
                    update_player_tracker,
                    enemy_spatial_awareness,
                    enemy_movement,
                    enemy_shooting,
                )
                    .chain(),
                // These can run in parallel
                update_enemy_ship_rotation,
                disintegrator_update,
                spawn::spawner_update,
                enemy_bounds_check,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
