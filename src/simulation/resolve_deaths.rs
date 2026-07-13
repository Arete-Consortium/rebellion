//! Death Resolution Systems
//!
//! Checks entity health and emits destruction events.
//! Does NOT spawn FX, mutate score, or spawn drops.

use crate::core::EnemyDestroyedEvent;
use crate::entities::{Enemy, EnemyStats};
use bevy::prelude::*;

/// Scan all enemies and despawn any whose health has dropped to zero or below.
/// Emits `EnemyDestroyedEvent` with full contextual data before despawning.
pub fn resolve_enemy_deaths(
    mut commands: Commands,
    enemy_query: Query<(Entity, &EnemyStats, &Transform), With<Enemy>>,
    mut destroy_events: EventWriter<EnemyDestroyedEvent>,
) {
    for (entity, stats, transform) in enemy_query.iter() {
        if stats.health <= 0.0 {
            destroy_events.send(EnemyDestroyedEvent {
                enemy: entity,
                position: transform.translation.truncate(),
                enemy_type: stats.name.clone(),
                score_value: stats.score_value,
                was_boss: stats.is_boss,
                liberation_value: stats.liberation_value,
                type_id: stats.type_id,
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}
