//! Collision Detection Systems
//!
//! Emits `ContactRaw` events for downstream resolution.
//! Does NOT mutate health, spawn FX, or mutate score.

use crate::core::{ContactRaw, RawContactType};
use crate::entities::*;
use crate::entities::environment::{
    circle_contact, EnvironmentCollider, EnvironmentObject, PlayerEnvironmentContact,
    ProjectileEnvironmentContact,
};
use crate::systems::collision::SpatialGrid;
use bevy::prelude::*;

/// Update spatial grid with current enemy and environment positions
pub fn update_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    env_query: Query<(Entity, &Transform, &EnvironmentCollider), With<EnvironmentObject>>,
) {
    grid.clear();
    for (entity, transform) in enemy_query.iter() {
        grid.insert_enemy(entity, transform.translation.truncate());
    }
    for (entity, transform, collider) in env_query.iter() {
        grid.insert_environment(entity, transform.translation.truncate(), collider.radius);
    }
}

/// Detect player colliding with environment objects using the spatial grid.
/// Emits `PlayerEnvironmentContact` for each overlapping contact.
pub fn detect_player_environment_contacts(
    grid: Res<SpatialGrid>,
    player_query: Query<(Entity, &Transform, &Hitbox), With<Player>>,
    mut contact_events: EventWriter<PlayerEnvironmentContact>,
) {
    let Ok((player_entity, player_transform, hitbox)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (env_entity, env_pos, env_radius) in grid.get_nearby_environments(player_pos) {
        if let Some(contact) = circle_contact(player_pos, hitbox.radius, env_pos, env_radius) {
            contact_events.send(PlayerEnvironmentContact {
                player: player_entity,
                environment: env_entity,
                player_position: player_pos,
                environment_position: env_pos,
                normal: contact.normal,
                penetration: contact.penetration,
            });
        }
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

/// Detect player projectiles colliding with environment objects.
/// Emits `ProjectileEnvironmentContact` for each contact.
pub fn detect_player_projectile_environment_hits(
    grid: Res<SpatialGrid>,
    projectile_query: Query<(Entity, &Transform, &ProjectileDamage, Option<&Pierce>), With<PlayerProjectile>>,
    mut contact_events: EventWriter<ProjectileEnvironmentContact>,
) {
    for (proj_entity, proj_transform, proj_damage, pierce) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();

        for (env_entity, env_pos, env_radius) in grid.get_nearby_environments(proj_pos) {
            let hit_radius = env_radius + 4.0;
            let dist_sq = (proj_pos - env_pos).length_squared();
            if dist_sq < hit_radius * hit_radius {
                contact_events.send(ProjectileEnvironmentContact {
                    projectile: proj_entity,
                    environment: env_entity,
                    projectile_pos: proj_pos,
                    environment_pos: env_pos,
                    damage: proj_damage.damage,
                    damage_type: proj_damage.damage_type,
                    pierce_remaining: pierce.map(|p| p.0),
                    is_player_projectile: true,
                });
                break; // Projectile can only hit one environment object per frame
            }
        }
    }
}

/// Detect enemy projectiles colliding with environment objects.
/// Emits `ProjectileEnvironmentContact` for each contact.
pub fn detect_enemy_projectile_environment_hits(
    grid: Res<SpatialGrid>,
    projectile_query: Query<(Entity, &Transform, &ProjectileDamage), With<EnemyProjectile>>,
    mut contact_events: EventWriter<ProjectileEnvironmentContact>,
) {
    for (proj_entity, proj_transform, proj_damage) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();

        for (env_entity, env_pos, env_radius) in grid.get_nearby_environments(proj_pos) {
            let hit_radius = env_radius + 4.0;
            let dist_sq = (proj_pos - env_pos).length_squared();
            if dist_sq < hit_radius * hit_radius {
                contact_events.send(ProjectileEnvironmentContact {
                    projectile: proj_entity,
                    environment: env_entity,
                    projectile_pos: proj_pos,
                    environment_pos: env_pos,
                    damage: proj_damage.damage,
                    damage_type: proj_damage.damage_type,
                    pierce_remaining: None,
                    is_player_projectile: false,
                });
                break; // Projectile can only hit one environment object per frame
            }
        }
    }
}
