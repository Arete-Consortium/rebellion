//! Damage Resolution Systems
//!
//! Consumes `ContactDetected` events and applies damage to entities.
//! Does NOT spawn FX, mutate score, or trigger dialogue.

use crate::core::{
    ContactDetected, ContactType, DamageLayer, DamageLayerEvent, EnemyDamageAppliedEvent,
    PlayerDamagedEvent,
};
use crate::entities::{BurnStatus, Enemy, EnemyStats, Pierce, Player, PowerupEffects, ShipStats};
use crate::systems::collision::SpatialGrid;
use crate::systems::ManeuverState;
use bevy::prelude::*;

/// Resolve player projectiles hitting enemies.
/// Applies damage, burn DoT, pierce, and chain lightning.
/// Emits `EnemyDamageAppliedEvent` for each hit.
pub fn resolve_player_projectile_damage(
    mut commands: Commands,
    grid: Res<SpatialGrid>,
    mut sim_rng: ResMut<crate::simulation::SimulationRng>,
    mut contact_events: EventReader<ContactDetected>,
    mut enemy_query: Query<&mut EnemyStats, With<Enemy>>,
    mut damage_applied_events: EventWriter<EnemyDamageAppliedEvent>,
    mut chain_bolt_events: EventWriter<crate::core::ChainBoltSpawnEvent>,
) {
    for contact in contact_events.read() {
        let ContactType::PlayerProjectileEnemy {
            projectile: proj_entity,
            enemy: enemy_entity,
            projectile_pos: _proj_pos,
            enemy_pos,
            damage,
            damage_type,
            crit_chance,
            crit_multiplier,
            ammo_type,
            pierce_remaining,
            burn_dps,
            chain_targets,
        } = contact.contact_type
        else {
            continue;
        };

        let Ok(mut enemy_stats) = enemy_query.get_mut(enemy_entity) else {
            continue;
        };

        // Roll for critical hit
        let is_crit = sim_rng.f32() < crit_chance;
        let crit_mult = if is_crit { crit_multiplier } else { 1.0 };
        let ammo_mult = ammo_type.armor_mult();
        let final_damage = damage * crit_mult * ammo_mult;

        enemy_stats.health -= final_damage;

        // Emit damage-applied event for presentation
        damage_applied_events.send(EnemyDamageAppliedEvent {
            enemy: enemy_entity,
            damage: final_damage,
            is_crit,
            enemy_pos,
            damage_type,
        });

        // Apply burn DoT if projectile carries one
        if let Some(dps) = burn_dps {
            commands.entity(enemy_entity).insert(BurnStatus {
                dps,
                remaining: 3.0,
            });
        }

        // Pre-plan chain targets (read-only grid)
        let chain_plan: Vec<(Entity, Vec2)> = if let Some(max_chains) = chain_targets {
            let mut visited: Vec<Entity> = vec![enemy_entity];
            let mut origin = enemy_pos;
            let mut plan = Vec::new();
            for _ in 0..max_chains {
                let mut best: Option<(Entity, Vec2)> = None;
                let mut best_d = 320.0_f32;
                for &(oe, opos) in grid.get_nearby_enemies(origin) {
                    if visited.contains(&oe) {
                        continue;
                    }
                    let d = origin.distance(opos);
                    if d < best_d {
                        best_d = d;
                        best = Some((oe, opos));
                    }
                }
                let Some((tgt, tpos)) = best else { break };
                plan.push((tgt, tpos));
                visited.push(tgt);
                origin = tpos;
            }
            plan
        } else {
            Vec::new()
        };

        // Pierce: decrement and keep projectile alive; else despawn
        match pierce_remaining {
            Some(n) if n > 0 => {
                commands.entity(proj_entity).insert(Pierce(n - 1));
            }
            _ => {
                commands.entity(proj_entity).despawn_recursive();
            }
        }

        // Execute chain-lightning plan
        let chain_dmg = damage * 0.9;
        let mut prev = enemy_pos;
        for (tgt, tpos) in chain_plan {
            if let Ok(mut tstats) = enemy_query.get_mut(tgt) {
                tstats.health -= chain_dmg;
                damage_applied_events.send(EnemyDamageAppliedEvent {
                    enemy: tgt,
                    damage: chain_dmg,
                    is_crit: false,
                    enemy_pos: tpos,
                    damage_type,
                });
            }
            chain_bolt_events.send(crate::core::ChainBoltSpawnEvent {
                from: prev,
                to: tpos,
            });
            prev = tpos;
        }
    }
}

/// Resolve enemy projectiles hitting the player.
/// Applies damage with layer breakdown and emits `PlayerDamagedEvent`.
pub fn resolve_enemy_projectile_damage(
    mut commands: Commands,
    mut contact_events: EventReader<ContactDetected>,
    mut player_query: Query<
        (&Transform, &mut ShipStats, &PowerupEffects, &ManeuverState),
        With<Player>,
    >,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut damage_layer_events: EventWriter<DamageLayerEvent>,
) {
    let Ok((player_transform, mut player_stats, powerups, maneuver)) =
        player_query.get_single_mut()
    else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for contact in contact_events.read() {
        let ContactType::EnemyProjectilePlayer {
            projectile,
            player: _,
            projectile_pos,
            damage,
            damage_type,
        } = contact.contact_type
        else {
            continue;
        };

        // Despawn projectile regardless
        commands.entity(projectile).despawn_recursive();

        // Check invulnerability (powerups OR barrel roll i-frames)
        if powerups.is_invulnerable() || maneuver.invincible {
            continue;
        }

        // Apply damage with layer tracking
        let damage_result = player_stats.take_damage_detailed(damage, damage_type);
        let direction = (player_pos - projectile_pos).normalize_or_zero();

        // Send damage layer events for visual effects
        if damage_result.shield_damage > 0.0 {
            damage_layer_events.send(DamageLayerEvent {
                position: player_pos,
                layer: DamageLayer::Shield,
                damage: damage_result.shield_damage,
                direction,
            });
        }
        if damage_result.armor_damage > 0.0 {
            damage_layer_events.send(DamageLayerEvent {
                position: player_pos,
                layer: DamageLayer::Armor,
                damage: damage_result.armor_damage,
                direction,
            });
        }
        if damage_result.hull_damage > 0.0 {
            damage_layer_events.send(DamageLayerEvent {
                position: player_pos,
                layer: DamageLayer::Hull,
                damage: damage_result.hull_damage,
                direction,
            });
        }

        // Send player damaged event
        damage_events.send(PlayerDamagedEvent {
            damage,
            damage_type,
            source_position: projectile_pos,
            shield_damage: damage_result.shield_damage,
            armor_damage: damage_result.armor_damage,
            hull_damage: damage_result.hull_damage,
            destroyed: damage_result.destroyed,
        });
    }
}
