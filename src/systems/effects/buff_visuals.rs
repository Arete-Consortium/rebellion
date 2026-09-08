//! Active buff visual effects on player

use crate::core::*;
use crate::entities::{Player, PowerupEffects};
use crate::systems::perf_profile::PerfProfile;
use bevy::prelude::*;

/// Golden hexagonal shield bubble for invulnerability
#[derive(Component)]
pub struct InvulnShieldBubble {
    /// Animation phase for pulsing
    pub phase: f32,
    /// Rotation angle for hex pattern
    pub rotation: f32,
}

/// Speed line particle trailing behind player during overdrive
#[derive(Component)]
pub struct OverdriveSpeedLine {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub alpha: f32,
}

/// Damage aura particle orbiting player during damage boost
#[derive(Component)]
pub struct DamageBoostAura {
    /// Orbital angle around player
    pub angle: f32,
    /// Distance from player center
    pub radius: f32,
    /// Particle lifetime
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Spawn and update active buff visuals on player
pub fn update_active_buff_visuals(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<(Entity, &Transform, &PowerupEffects), With<Player>>,
    mut shield_query: Query<(Entity, &mut InvulnShieldBubble, &mut Transform), Without<Player>>,
    speed_line_query: Query<&OverdriveSpeedLine>,
    aura_query: Query<&DamageBoostAura>,
    profile: Res<PerfProfile>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    let Ok((player_entity, player_transform, effects)) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    // Exactly one marked root owns six unmarked outline edges. Marking every
    // edge as a bubble made get_single_mut fail and spawn seven more sprites
    // every frame, obscuring the ship and growing the effect without a cap.
    if effects.is_invulnerable() {
        if let Ok((_, mut bubble, mut transform)) = shield_query.get_single_mut() {
            bubble.phase += dt * 3.0;
            bubble.rotation += dt * 0.5;
            transform.translation.x = player_pos.x;
            transform.translation.y = player_pos.y;
            transform.rotation = Quat::from_rotation_z(bubble.rotation);
            transform.scale = Vec3::splat(bubble.phase.sin() * 0.06 + 1.0);
        } else if shield_query.is_empty() {
            commands
                .spawn((
                    InvulnShieldBubble {
                        phase: 0.0,
                        rotation: 0.0,
                    },
                    Transform::from_xyz(player_pos.x, player_pos.y, LAYER_EFFECTS + 3.0),
                    Visibility::Visible,
                ))
                .with_children(|parent| {
                    let radius = 42.0;
                    for i in 0..6 {
                        let a = Vec2::from_angle(i as f32 * std::f32::consts::TAU / 6.0) * radius;
                        let b =
                            Vec2::from_angle((i + 1) as f32 * std::f32::consts::TAU / 6.0) * radius;
                        let edge = b - a;
                        parent.spawn((
                            Sprite {
                                color: Color::srgba(1.0, 0.9, 0.4, 0.8),
                                custom_size: Some(Vec2::new(edge.length(), 2.0)),
                                ..default()
                            },
                            Transform::from_translation(((a + b) * 0.5).extend(0.0))
                                .with_rotation(Quat::from_rotation_z(edge.y.atan2(edge.x))),
                        ));
                    }
                });
        }
    } else {
        for (entity, _, _) in &shield_query {
            commands.entity(entity).despawn_recursive();
        }
    }

    // === OVERDRIVE SPEED LINES ===
    if effects.is_overdrive() {
        // Spawn speed lines behind player
        let current_lines = speed_line_query.iter().count();

        if current_lines < profile.speed_lines && fastrand::f32() < 0.6 {
            // Spawn at random position behind player
            let offset_x = (fastrand::f32() - 0.5) * 40.0;
            let offset_y = -30.0 - fastrand::f32() * 20.0;
            let lifetime = 0.2 + fastrand::f32() * 0.15;

            commands.spawn((
                OverdriveSpeedLine {
                    lifetime,
                    max_lifetime: lifetime,
                    alpha: 0.7,
                },
                Sprite {
                    color: Color::srgba(0.3, 0.9, 1.0, 0.7), // Cyan
                    custom_size: Some(Vec2::new(3.0, 15.0 + fastrand::f32() * 20.0)),
                    ..default()
                },
                Transform::from_xyz(
                    player_pos.x + offset_x,
                    player_pos.y + offset_y,
                    LAYER_EFFECTS + 2.5,
                ),
            ));
        }
    }

    // === DAMAGE BOOST AURA ===
    if effects.is_damage_boosted() {
        // Spawn orbiting particles
        let current_aura = aura_query.iter().count();

        if current_aura < profile.aura_particles && fastrand::f32() < 0.3 {
            let angle = fastrand::f32() * std::f32::consts::TAU;
            let radius = 35.0 + fastrand::f32() * 15.0;
            let lifetime = 0.5 + fastrand::f32() * 0.3;

            commands.spawn((
                DamageBoostAura {
                    angle,
                    radius,
                    lifetime,
                    max_lifetime: lifetime,
                },
                Sprite {
                    color: Color::srgba(1.0, 0.3, 0.2, 0.8),
                    custom_size: Some(Vec2::splat(6.0 + fastrand::f32() * 4.0)),
                    ..default()
                },
                Transform::from_xyz(
                    player_pos.x + angle.cos() * radius,
                    player_pos.y + angle.sin() * radius,
                    LAYER_EFFECTS + 2.8,
                ),
            ));
        }
    }

    // Suppress unused variable warning
    let _ = (player_entity, elapsed);
}

/// Update overdrive speed lines
pub fn update_overdrive_speed_lines(
    mut commands: Commands,
    time: Res<Time>,
    mut lines: Query<(Entity, &mut OverdriveSpeedLine, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut line, mut sprite, mut transform) in lines.iter_mut() {
        line.lifetime -= dt;

        if line.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Move downward (ship is moving up)
        transform.translation.y -= 400.0 * dt;

        // Fade and stretch
        let progress = line.lifetime / line.max_lifetime;
        sprite.color = sprite.color.with_alpha(progress * line.alpha);

        // Stretch as they fade
        if let Some(size) = sprite.custom_size {
            sprite.custom_size = Some(Vec2::new(size.x * 0.98, size.y * 1.02));
        }
    }
}

/// Update damage boost aura particles
pub fn update_damage_boost_aura(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<(&Transform, &PowerupEffects), With<Player>>,
    mut aura: Query<(Entity, &mut DamageBoostAura, &mut Sprite, &mut Transform), Without<Player>>,
) {
    let dt = time.delta_secs();

    let Ok((player_transform, effects)) = player_query.get_single() else {
        // Despawn all aura particles if no player
        for (entity, _, _, _) in aura.iter() {
            commands.entity(entity).despawn();
        }
        return;
    };

    // If damage boost ended, despawn all particles
    if !effects.is_damage_boosted() {
        for (entity, _, _, _) in aura.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let player_pos = player_transform.translation.truncate();

    for (entity, mut particle, mut sprite, mut transform) in aura.iter_mut() {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Orbit around player
        particle.angle += dt * 4.0; // Angular velocity
        particle.radius -= dt * 10.0; // Slowly spiral inward

        // Update position to follow player
        transform.translation.x = player_pos.x + particle.angle.cos() * particle.radius;
        transform.translation.y = player_pos.y + particle.angle.sin() * particle.radius;

        // Fade out
        let progress = particle.lifetime / particle.max_lifetime;
        let base = sprite.color.to_srgba();
        sprite.color = Color::srgba(base.red, base.green, base.blue, progress * 0.8);

        // Shrink as they approach center
        let size_factor = (particle.radius / 50.0).clamp(0.3, 1.0);
        sprite.custom_size = Some(Vec2::splat(6.0 * size_factor));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn sustained_invulnerability_keeps_one_outline_and_removes_all_edges() {
        let mut app = App::new();
        app.init_resource::<Time>().init_resource::<PerfProfile>();
        let player = app
            .world_mut()
            .spawn((
                Player,
                Transform::default(),
                PowerupEffects {
                    invuln_timer: 3.0,
                    ..default()
                },
            ))
            .id();
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(1.0 / 60.0));
        for _ in 0..180 {
            app.world_mut()
                .run_system_once(update_active_buff_visuals)
                .unwrap();
        }
        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<InvulnShieldBubble>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1, "shield must not grow each frame");
        assert!(
            app.world().get::<Sprite>(roots[0]).is_none(),
            "no filled quad over the hull"
        );
        assert_eq!(app.world().get::<Children>(roots[0]).unwrap().len(), 6);
        assert_eq!(
            app.world().entities().len(),
            8,
            "one player, one root, six outline edges"
        );
        app.world_mut()
            .get_mut::<PowerupEffects>(player)
            .unwrap()
            .invuln_timer = 0.0;
        app.world_mut()
            .run_system_once(update_active_buff_visuals)
            .unwrap();
        assert_eq!(
            app.world().entities().len(),
            1,
            "expired shield leaves no orphan geometry"
        );
    }
}
