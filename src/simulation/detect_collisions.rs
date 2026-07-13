//! Collision Detection Systems
//!
//! Emits `ContactRaw` events for downstream resolution.
//! Does NOT mutate health, spawn FX, or mutate score.

use crate::core::{ContactRaw, RawContactType};
use crate::entities::*;
use crate::systems::collision::SpatialGrid;
use bevy::prelude::*;

/// Update spatial grid with current enemy positions
pub fn update_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    grid.clear();
    for (entity, transform) in enemy_query.iter() {
        grid.insert_enemy(entity, transform.translation.truncate());
    }
}

/// Detect player projectiles colliding with enemies using the spatial grid.
/// Emits `ContactRaw::PlayerProjectileEnemy` for each contact.
pub fn detect_player_projectile_hits(
    grid: Res<SpatialGrid>,
    projectile_query: Query<(Entity, &Transform), With<PlayerProjectile>>,
    mut contact_events: EventWriter<ContactRaw>,
) {
    const COLLISION_RADIUS_SQ: f32 = 25.0 * 25.0;

    for (proj_entity, proj_transform) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();

        for &(enemy_entity, enemy_pos) in grid.get_nearby_enemies(proj_pos) {
            let dist_sq = (proj_pos - enemy_pos).length_squared();
            if dist_sq < COLLISION_RADIUS_SQ {
                contact_events.send(ContactRaw {
                    contact_type: RawContactType::PlayerProjectileEnemy {
                        projectile: proj_entity,
                        enemy: enemy_entity,
                        projectile_pos: proj_pos,
                        enemy_pos,
                    },
                });
                break; // Projectile can only hit one enemy per frame
            }
        }
    }
}

/// Detect enemy projectiles colliding with the player.
/// Emits `ContactRaw::EnemyProjectilePlayer` for each contact.
pub fn detect_enemy_projectile_hits(
    projectile_query: Query<(Entity, &Transform), With<EnemyProjectile>>,
    player_query: Query<(Entity, &Transform, &Hitbox), With<Player>>,
    mut contact_events: EventWriter<ContactRaw>,
) {
    let Ok((player_entity, player_transform, hitbox)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let hit_radius_sq = (hitbox.radius + 4.0) * (hitbox.radius + 4.0);

    for (proj_entity, proj_transform) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();
        let dist_sq = (proj_pos - player_pos).length_squared();

        if dist_sq < hit_radius_sq {
            contact_events.send(ContactRaw {
                contact_type: RawContactType::EnemyProjectilePlayer {
                    projectile: proj_entity,
                    player: player_entity,
                    projectile_pos: proj_pos,
                    player_pos,
                },
            });
        }
    }
}
