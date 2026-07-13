//! Enemy Systems
//!
//! Movement, shooting, disintegrator beam, bounds checking, and ship rotation systems.

use super::types::*;
use crate::assets::ShipModelRotation;
use crate::core::*;
use bevy::prelude::*;

use super::ai::PlayerTracker;

/// Enemy movement based on AI behavior + spatial awareness dodge impulse
pub(super) fn enemy_movement(
    time: Res<Time>,
    player_tracker: Res<PlayerTracker>,
    mut query: Query<
        (&mut Transform, &EnemyStats, &mut EnemyAI),
        (With<Enemy>, Without<crate::entities::Player>),
    >,
) {
    let dt = time.delta_secs();
    let player_pos = player_tracker.position;

    for (mut transform, stats, mut ai) in query.iter_mut() {
        ai.timer += dt;
        let pos = transform.translation.truncate();

        let velocity = match ai.behavior {
            EnemyBehavior::Linear => Vec2::new(0.0, -1.0) * stats.speed,
            EnemyBehavior::Zigzag => {
                let x = (ai.timer * 3.0 + ai.phase).sin() * stats.speed;
                Vec2::new(x, -stats.speed * 0.5)
            }
            EnemyBehavior::Homing => {
                let dir = (player_pos - pos).normalize_or_zero();
                dir * stats.speed
            }
            EnemyBehavior::Orbital => {
                let angle = ai.timer * 2.0 + ai.phase;
                let orbit_center = Vec2::new(0.0, 100.0);
                let target = orbit_center + Vec2::new(angle.cos(), angle.sin()) * 150.0;
                (target - pos).normalize_or_zero() * stats.speed
            }
            EnemyBehavior::Sniper => {
                // Stay at top, strafe
                let target_y = SCREEN_HEIGHT / 2.0 - 100.0;
                let y_diff = target_y - pos.y;
                let x = (ai.timer * 1.5 + ai.phase).sin() * stats.speed;
                Vec2::new(x, y_diff.signum() * stats.speed.min(y_diff.abs()))
            }
            EnemyBehavior::Kamikaze => {
                // Suicide rush toward player at 2x speed
                let dir = (player_pos - pos).normalize_or_zero();
                dir * stats.speed * 2.0
            }
            EnemyBehavior::Weaver => {
                // Fast sine-wave, wide amplitude, harassing movement
                let amplitude = 200.0;
                let frequency = 4.0;
                let x = (ai.timer * frequency + ai.phase).sin() * amplitude * dt * 2.0;
                Vec2::new(x, -stats.speed * 0.7)
            }
            EnemyBehavior::Spawner => {
                // Slow descent, stays in upper area
                let target_y = SCREEN_HEIGHT / 2.0 - 150.0;
                if pos.y > target_y {
                    Vec2::new(0.0, -stats.speed * 0.3)
                } else {
                    // Slow side-to-side drift once in position
                    let x = (ai.timer * 0.5).sin() * stats.speed * 0.3;
                    Vec2::new(x, 0.0)
                }
            }
            EnemyBehavior::Tank => {
                // Slow but relentless advance toward player
                let dir = (player_pos - pos).normalize_or_zero();
                // Mostly moves down, slight homing
                Vec2::new(dir.x * stats.speed * 0.3, -stats.speed * 0.4)
            }
            EnemyBehavior::Disintegrator => {
                // Triglavian: Maintains distance while tracking player
                // Optimal range: 150-250 units from player
                let to_player = player_pos - pos;
                let distance = to_player.length();
                let dir = to_player.normalize_or_zero();

                let optimal_range = 200.0;
                let approach_speed = if distance > optimal_range + 50.0 {
                    stats.speed * 0.8 // Close in
                } else if distance < optimal_range - 50.0 {
                    -stats.speed * 0.5 // Back off
                } else {
                    0.0 // At optimal range
                };

                // Strafe perpendicular to player direction
                let strafe = Vec2::new(-dir.y, dir.x) * (ai.timer * 2.0).sin() * stats.speed * 0.4;

                dir * approach_speed + strafe + Vec2::new(0.0, -stats.speed * 0.2)
            }
        };

        // Combine behavior velocity with spatial awareness (dodge + separation + edge avoidance)
        let total_velocity = velocity + ai.dodge_impulse;

        transform.translation.x += total_velocity.x * dt;
        transform.translation.y += total_velocity.y * dt;

        // Slight tilt based on horizontal movement (visual effect only)
        let tilt = (total_velocity.x / stats.speed.max(1.0)).clamp(-1.0, 1.0) * 0.2;
        transform.rotation = Quat::from_rotation_z(tilt);
    }
}

/// Enemy shooting system with predictive aiming
/// Enemies lead their shots based on player velocity — accuracy depends on behavior type
pub(super) fn enemy_shooting(
    mut commands: Commands,
    time: Res<Time>,
    player_tracker: Res<PlayerTracker>,
    mut query: Query<(&Transform, &mut EnemyWeapon, &EnemyAI), With<Enemy>>,
) {
    let dt = time.delta_secs();
    let player_pos = player_tracker.position;
    let player_vel = player_tracker.velocity;

    for (transform, mut weapon, ai) in query.iter_mut() {
        if !ai.active {
            continue;
        }

        weapon.cooldown -= dt;
        if weapon.cooldown <= 0.0 {
            weapon.cooldown = 1.0 / weapon.fire_rate;

            let pos = transform.translation.truncate();

            // Predictive aiming: lead the shot based on player velocity
            let accuracy = ai.behavior.aim_accuracy();
            let distance = (player_pos - pos).length();
            let flight_time = distance / weapon.bullet_speed.max(1.0);
            let predicted_pos = player_pos + player_vel * flight_time * accuracy;

            let dir = (predicted_pos - pos).normalize_or_zero();

            // Per-hull fire pattern: autocannon twin-tracer, drone tri-spread,
            // everything else stays single-shot so the weapon family reads.
            let (shots, spread_rad) = match weapon.weapon_type {
                WeaponType::Autocannon => (2, 6_f32.to_radians()),
                WeaponType::Drone => (3, 18_f32.to_radians()),
                _ => (1, 0.0),
            };
            let base_angle = dir.y.atan2(dir.x);
            for i in 0..shots {
                let offset = if shots > 1 {
                    -spread_rad / 2.0 + spread_rad * (i as f32 / (shots - 1) as f32)
                } else {
                    0.0
                };
                let angle = base_angle + offset;
                let shot_dir = Vec2::new(angle.cos(), angle.sin());
                crate::entities::projectile::spawn_enemy_projectile_typed(
                    &mut commands,
                    pos,
                    shot_dir,
                    weapon.damage,
                    weapon.bullet_speed,
                    weapon.weapon_type,
                );
            }
        }
    }
}

/// Triglavian disintegrator beam system
/// Handles continuous beam damage with ramping multiplier
pub(super) fn disintegrator_update(
    time: Res<Time>,
    mut player_query: Query<
        (
            &Transform,
            &mut crate::entities::ShipStats,
            &crate::entities::PowerupEffects,
            &crate::systems::ManeuverState,
        ),
        With<crate::entities::Player>,
    >,
    mut enemy_query: Query<(&Transform, &mut DisintegratorRamp, &EnemyAI), With<Enemy>>,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let dt = time.delta_secs();

    let Ok((player_transform, mut player_stats, powerups, maneuver)) =
        player_query.get_single_mut()
    else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    // Check invulnerability
    let player_invulnerable = powerups.is_invulnerable() || maneuver.invincible;

    for (enemy_transform, mut disintegrator, ai) in enemy_query.iter_mut() {
        if !ai.active {
            disintegrator.update(dt, false);
            continue;
        }

        let enemy_pos = enemy_transform.translation.truncate();
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();

        // Check if player is within beam range (350 units max)
        let in_range = distance < 350.0;

        // Update ramping state
        disintegrator.update(dt, in_range);

        // Apply damage if beam is active
        if disintegrator.beam_active && !player_invulnerable {
            // Damage per second = base * mult, convert to per-frame damage
            let damage_per_frame = disintegrator.current_damage() * dt;

            // Apply damage directly to player
            let damage_result =
                player_stats.take_damage_detailed(damage_per_frame, DamageType::Thermal);

            // Send damage event for other systems to react
            damage_events.send(PlayerDamagedEvent {
                damage: damage_per_frame,
                damage_type: DamageType::Thermal,
                source_position: enemy_pos,
                shield_damage: damage_result.shield_damage,
                armor_damage: damage_result.armor_damage,
                hull_damage: damage_result.hull_damage,
                destroyed: damage_result.destroyed,
            });

            if damage_result.destroyed {
                info!("Player destroyed by disintegrator beam!");
                next_state.set(GameState::GameOver);
            }
        }
    }
}

/// Remove enemies that go off screen, OR wrap them back in if they carry
/// the `CycleOnExit` marker — used by formation patrols so they loop across
/// the battlefield instead of one-shot vanishing.
pub(super) fn enemy_bounds_check(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, Option<&super::CycleOnExit>), With<Enemy>>,
    mut sim_rng: ResMut<crate::simulation::SimulationRng>,
) {
    let margin = 100.0;
    let half_w = SCREEN_WIDTH / 2.0 + margin;
    let half_h = SCREEN_HEIGHT / 2.0 + margin;
    for (entity, mut transform, cycle) in query.iter_mut() {
        let pos = transform.translation;
        let out_bottom = pos.y < -half_h;
        let out_top = pos.y > half_h;
        let out_x = pos.x.abs() > half_w;
        if !(out_bottom || out_top || out_x) {
            continue;
        }
        if cycle.is_some() {
            // Wrap to the opposite side so patrols reappear and keep pressure
            if out_bottom {
                transform.translation.y = half_h - margin;
                // randomise X a bit so they don't all loop on the same column
                transform.translation.x = (sim_rng.f32() - 0.5) * (SCREEN_WIDTH - 120.0);
            } else if out_top {
                transform.translation.y = -half_h + margin;
            } else if out_x {
                transform.translation.x = -transform.translation.x.signum() * (half_w - margin);
            }
        } else {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Update 3D enemy rotation based on movement (banking/tilting)
pub(super) fn update_enemy_ship_rotation(
    time: Res<Time>,
    mut query: Query<(&EnemyStats, &EnemyAI, &mut Transform, &ShipModelRotation), With<Enemy>>,
) {
    let dt = time.delta_secs();

    for (stats, ai, mut transform, model_rot) in query.iter_mut() {
        // Estimate velocity from AI behavior
        let velocity = match ai.behavior {
            EnemyBehavior::Linear => Vec2::new(0.0, -stats.speed),
            EnemyBehavior::Zigzag => {
                let x = (ai.timer * 3.0 + ai.phase).sin() * stats.speed;
                Vec2::new(x, -stats.speed * 0.5)
            }
            EnemyBehavior::Homing | EnemyBehavior::Kamikaze => {
                // These move toward player, estimate based on target
                let dir = (ai.target - transform.translation.truncate()).normalize_or_zero();
                dir * stats.speed
            }
            EnemyBehavior::Orbital => {
                let angle = ai.timer * 2.0 + ai.phase;
                Vec2::new(-angle.sin(), angle.cos()) * stats.speed * 0.5
            }
            EnemyBehavior::Sniper => {
                let x = (ai.timer * 1.5 + ai.phase).sin() * stats.speed;
                Vec2::new(x, 0.0)
            }
            EnemyBehavior::Weaver => {
                let x = (ai.timer * 4.0 + ai.phase).cos() * stats.speed;
                Vec2::new(x, -stats.speed * 0.7)
            }
            EnemyBehavior::Spawner => {
                let x = (ai.timer * 0.5).cos() * stats.speed * 0.3;
                Vec2::new(x, 0.0)
            }
            EnemyBehavior::Tank => Vec2::new(0.0, -stats.speed * 0.4),
            EnemyBehavior::Disintegrator => {
                // Triglavian ships strafe while tracking
                let strafe = (ai.timer * 2.0).sin() * stats.speed * 0.4;
                Vec2::new(strafe, -stats.speed * 0.2)
            }
        };

        let target_rotation = model_rot.calculate_rotation(velocity, stats.speed);
        transform.rotation = transform
            .rotation
            .slerp(target_rotation, (model_rot.smoothing * dt).min(1.0));
    }
}
