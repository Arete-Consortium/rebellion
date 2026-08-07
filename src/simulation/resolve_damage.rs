//! Damage Resolution Systems
//!
//! Consumes `ContactRaw` events, enriches them into `ContactDetected`,
//! then applies damage to entities.
//! Does NOT spawn FX, mutate score, or trigger dialogue.

use crate::core::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::core::{
    ContactDetected, ContactRaw, ContactType, DamageLayer, DamageLayerEvent,
    EnemyDamageAppliedEvent, PlayerDamagedEvent, RawContactType,
};
use crate::entities::environment::{
    resolve_boundary_pin, ContactCooldown, EnvironmentCollider, EnvironmentContactDamage,
    EnvironmentDamageAppliedEvent, EnvironmentDestroyedEvent, EnvironmentHealth, EnvironmentObject,
    EnvironmentScoreValue, PlayerEnvironmentContact, ProjectileEnvironmentContact,
    ProjectileInteraction,
};
use crate::entities::{
    BurnOnHit, BurnStatus, ChainOnHit, Enemy, EnemyProjectile, EnemyStats, Hitbox, Movement,
    Pierce, Player, PlayerProjectile, PowerupEffects, ProjectileDamage, ShipStats,
};
use crate::systems::collision::SpatialGrid;
use crate::systems::ManeuverState;
use bevy::prelude::*;

// =============================================================================
// Stage 1: Enrich raw contacts into resolved contacts
// =============================================================================

/// Consume `ContactRaw` events and emit `ContactDetected` with full projectile
/// stats looked up from the ECS.
pub fn enrich_contacts(
    mut raw_events: EventReader<ContactRaw>,
    player_proj_query: Query<
        (
            &ProjectileDamage,
            Option<&Pierce>,
            Option<&BurnOnHit>,
            Option<&ChainOnHit>,
        ),
        With<PlayerProjectile>,
    >,
    enemy_proj_query: Query<&ProjectileDamage, With<EnemyProjectile>>,
    mut resolved_events: EventWriter<ContactDetected>,
) {
    for raw in raw_events.read() {
        match raw.contact_type {
            RawContactType::PlayerProjectileEnemy {
                projectile,
                enemy,
                projectile_pos,
                enemy_pos,
            } => {
                let Ok((proj_damage, pierce, burn, chain)) = player_proj_query.get(projectile)
                else {
                    continue;
                };
                let pierce_remaining: Option<u32> = pierce.map(|p| p.0);
                let burn_dps: Option<f32> = burn.map(|b| b.0);
                let chain_targets: Option<u32> = chain.map(|c| c.0);
                resolved_events.send(ContactDetected {
                    contact_type: ContactType::PlayerProjectileEnemy {
                        projectile,
                        enemy,
                        projectile_pos,
                        enemy_pos,
                        damage: proj_damage.damage,
                        damage_type: proj_damage.damage_type,
                        crit_chance: proj_damage.crit_chance,
                        crit_multiplier: proj_damage.crit_multiplier,
                        ammo_type: proj_damage.ammo_type,
                        pierce_remaining,
                        burn_dps,
                        chain_targets,
                    },
                });
            }
            RawContactType::EnemyProjectilePlayer {
                projectile,
                player,
                projectile_pos,
                player_pos: _,
            } => {
                let Ok(proj_damage) = enemy_proj_query.get(projectile) else {
                    continue;
                };
                resolved_events.send(ContactDetected {
                    contact_type: ContactType::EnemyProjectilePlayer {
                        projectile,
                        player,
                        projectile_pos,
                        damage: proj_damage.damage,
                        damage_type: proj_damage.damage_type,
                    },
                });
            }
        }
    }
}

// =============================================================================
// Stage 2: Apply damage from resolved contacts
// =============================================================================

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

// =============================================================================
// Stage 3: Resolve player / environment contacts
// =============================================================================

/// Resolve player overlapping with environment objects.
/// Applies boundary-pin separation, deflects velocity along normal, and
/// applies contact damage with cooldown.
pub fn resolve_player_environment_contacts(
    mut commands: Commands,
    mut contact_events: EventReader<PlayerEnvironmentContact>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut Movement,
            &mut ShipStats,
            &ManeuverState,
            &Hitbox,
        ),
        (With<Player>, Without<EnvironmentObject>),
    >,
    mut env_query: Query<
        (
            &Transform,
            &EnvironmentCollider,
            Option<&EnvironmentContactDamage>,
            Option<&mut ContactCooldown>,
        ),
        (With<EnvironmentObject>, Without<Player>),
    >,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut damage_layer_events: EventWriter<DamageLayerEvent>,
) {
    let Ok((mut player_transform, mut movement, mut player_stats, maneuver, hitbox)) =
        player_query.get_single_mut()
    else {
        return;
    };

    for contact in contact_events.read() {
        let Ok((env_transform, env_collider, contact_damage, mut cooldown)) =
            env_query.get_mut(contact.environment)
        else {
            continue;
        };

        let env_pos = env_transform.translation.truncate();
        let player_pos = player_transform.translation.truncate();

        // ── Separation ──
        let slop = 2.0; // small buffer to prevent jitter
        let corrected = resolve_boundary_pin(
            player_pos,
            hitbox.radius,
            env_pos,
            env_collider.radius,
            &crate::entities::environment::CircleContact {
                normal: contact.normal,
                penetration: contact.penetration,
            },
            slop,
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        );
        player_transform.translation.x = corrected.x;
        player_transform.translation.y = corrected.y;

        // ── Deflection ──
        // Nudge velocity along contact normal to prevent sliding into the object again
        let deflection = contact.normal
            * contact.penetration
            * ((1.0 / crate::simulation::FIXED_TIMESTEP_SECS) as f32);
        movement.velocity += deflection;

        // ── Contact Damage ──
        if let Some(dmg) = contact_damage {
            // Check cooldown
            let can_damage = if let Some(ref mut cd) = cooldown {
                if cd.remaining_ticks == 0 {
                    cd.remaining_ticks = dmg.cooldown_ticks;
                    true
                } else {
                    false
                }
            } else {
                // No cooldown component yet — insert one and apply damage this tick
                commands
                    .entity(contact.environment)
                    .insert(ContactCooldown {
                        remaining_ticks: dmg.cooldown_ticks,
                    });
                true
            };

            if can_damage && !maneuver.invincible {
                let damage_result = player_stats.take_damage_detailed(dmg.amount, dmg.damage_type);
                let direction = contact.normal;

                if damage_result.shield_damage > 0.0 {
                    damage_layer_events.send(DamageLayerEvent {
                        position: corrected,
                        layer: DamageLayer::Shield,
                        damage: damage_result.shield_damage,
                        direction,
                    });
                }
                if damage_result.armor_damage > 0.0 {
                    damage_layer_events.send(DamageLayerEvent {
                        position: corrected,
                        layer: DamageLayer::Armor,
                        damage: damage_result.armor_damage,
                        direction,
                    });
                }
                if damage_result.hull_damage > 0.0 {
                    damage_layer_events.send(DamageLayerEvent {
                        position: corrected,
                        layer: DamageLayer::Hull,
                        damage: damage_result.hull_damage,
                        direction,
                    });
                }

                damage_events.send(PlayerDamagedEvent {
                    damage: dmg.amount,
                    damage_type: dmg.damage_type,
                    source_position: env_pos,
                    shield_damage: damage_result.shield_damage,
                    armor_damage: damage_result.armor_damage,
                    hull_damage: damage_result.hull_damage,
                    destroyed: damage_result.destroyed,
                });
            }
        }
    }
}

// =============================================================================
// Stage 4: Resolve projectile / environment contacts
// =============================================================================

/// Resolve projectiles hitting environment objects.
/// Applies damage to destructible objects, despawns projectiles on absorb,
/// and ignores decorative objects.
pub fn resolve_projectile_environment_contacts(
    mut commands: Commands,
    mut contact_events: EventReader<ProjectileEnvironmentContact>,
    mut env_query: Query<
        (
            &mut EnvironmentHealth,
            &EnvironmentScoreValue,
            &ProjectileInteraction,
            &Transform,
        ),
        With<EnvironmentObject>,
    >,
    mut damage_applied_events: EventWriter<EnvironmentDamageAppliedEvent>,
    mut destroyed_events: EventWriter<EnvironmentDestroyedEvent>,
) {
    for contact in contact_events.read() {
        let Ok((mut health, score_value, interaction, env_transform)) =
            env_query.get_mut(contact.environment)
        else {
            continue;
        };

        match *interaction {
            ProjectileInteraction::Ignore => {
                // Decorative — nothing happens
                continue;
            }
            ProjectileInteraction::Absorb => {
                // Hard terrain — projectile is destroyed, no damage
                commands.entity(contact.projectile).despawn_recursive();
            }
            ProjectileInteraction::Damageable => {
                // Soft hazard / asteroid — apply damage; respect pierce
                health.current -= contact.damage;
                let destroyed = health.current <= 0.0;

                damage_applied_events.send(EnvironmentDamageAppliedEvent {
                    environment: contact.environment,
                    position: env_transform.translation.truncate(),
                    damage: contact.damage,
                    damage_type: contact.damage_type,
                    destroyed,
                });

                if destroyed {
                    destroyed_events.send(EnvironmentDestroyedEvent {
                        environment: contact.environment,
                        definition_id: String::new(), // populated by spawner if needed
                        position: env_transform.translation.truncate(),
                        score_value: score_value.0,
                    });
                    commands.entity(contact.environment).despawn_recursive();
                }

                // Pierce: decrement and keep projectile alive; else despawn
                match contact.pierce_remaining {
                    Some(n) if n > 0 => {
                        commands
                            .entity(contact.projectile)
                            .insert(crate::entities::Pierce(n - 1));
                    }
                    _ => {
                        commands.entity(contact.projectile).despawn_recursive();
                    }
                }
            }
        }
    }
}
