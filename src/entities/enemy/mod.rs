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
pub use faction::{get_player_weapon_type, get_ship_rotation_correction, ship_sprite_tint};
pub use spawn::*;
pub use types::*;

use ai::*;
use systems::*;

use crate::core::*;
use bevy::prelude::*;

/// Marker on enemies that should wrap to the opposite edge of the play area
/// instead of despawning when they exit. Used for formation patrols so they
/// loop over the battlefield continuously.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CycleOnExit;

/// One-shot marker — pending Endless-mode stat scaling. Consumed on the next
/// tick by `apply_endless_scale_system` which multiplies health + damage by
/// the stored factor, then removes the component. Prevents double-scaling.
#[derive(Component, Debug, Clone, Copy)]
pub struct EndlessScale(pub f32);

/// Enemy plugin
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerTracker>().add_systems(
            FixedUpdate,
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
                apply_endless_scale_system,
            )
                .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
        );
    }
}

/// Consume `EndlessScale` markers by multiplying enemy max HP.
/// Runs once per spawn — marker is removed to prevent compounding.
fn apply_endless_scale_system(
    mut commands: Commands,
    mut q: Query<(Entity, &EndlessScale, &mut types::EnemyStats)>,
) {
    for (entity, scale, mut stats) in q.iter_mut() {
        stats.health *= scale.0;
        stats.max_health *= scale.0;
        commands.entity(entity).remove::<EndlessScale>();
    }
}
