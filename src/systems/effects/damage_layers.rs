//! Damage layer visual effects: shield ripples, armor sparks, hull fire

use bevy::prelude::*;
use crate::core::*;
use super::screen_effects::ScreenShake;

/// Maximum damage layer particles
const MAX_DAMAGE_LAYER_PARTICLES: usize = 100;

/// Maximum shield ripple effects at once
const MAX_SHIELD_RIPPLES: usize = 15;

/// Shield impact ripple - hexagonal expanding ring
#[derive(Component)]
pub struct ShieldRipple {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub max_radius: f32,
    pub angle: f32, // Direction of impact
}

/// Armor spark particle - directional spark spray
#[derive(Component)]
pub struct ArmorSpark {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Hull fire/smoke particle - persistent damage indicator
#[derive(Component)]
pub struct HullFireParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub is_smoke: bool,
}

/// Handle damage layer events and spawn appropriate effects
pub fn handle_damage_layer_events(
    mut commands: Commands,
    mut events: EventReader<DamageLayerEvent>,
    spark_query: Query<&ArmorSpark>,
    ripple_query: Query<&ShieldRipple>,
    mut screen_shake: ResMut<ScreenShake>,
) {
    let current_sparks = spark_query.iter().count();
    let current_ripples = ripple_query.iter().count();

    for event in events.read() {
        match event.layer {
            DamageLayer::Shield => {
                // Cap shield ripples to prevent lag during sustained fire
                if current_ripples < MAX_SHIELD_RIPPLES {
                    spawn_shield_ripple(&mut commands, event.position, event.direction);
                }
            }
            DamageLayer::Armor => {
                if current_sparks < MAX_DAMAGE_LAYER_PARTICLES {
                    spawn_armor_sparks(
                        &mut commands,
                        event.position,
                        event.direction,
                        event.damage,
                    );
                }
            }
            DamageLayer::Hull => {
                spawn_hull_fire(&mut commands, event.position, event.damage);
                // Hull damage causes stronger screen shake
                screen_shake.trigger(6.0, 0.15);
            }
        }
    }
}

/// Spawn a shield impact ripple effect
fn spawn_shield_ripple(commands: &mut Commands, position: Vec2, direction: Vec2) {
    let angle = direction.y.atan2(direction.x);
    let lifetime = 0.4;

    // Main ripple ring
    commands.spawn((
        ShieldRipple {
            lifetime,
            max_lifetime: lifetime,
            max_radius: 60.0,
            angle,
        },
        Sprite {
            color: Color::srgba(0.3, 0.7, 1.0, 0.8), // Blue shield color
            custom_size: Some(Vec2::splat(10.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.0),
    ));

    // Secondary inner ripple
    commands.spawn((
        ShieldRipple {
            lifetime: lifetime * 0.8,
            max_lifetime: lifetime * 0.8,
            max_radius: 40.0,
            angle,
        },
        Sprite {
            color: Color::srgba(0.5, 0.8, 1.0, 0.6),
            custom_size: Some(Vec2::splat(8.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 0.9),
    ));

    // Spawn hex grid particles at impact point
    let num_particles = 6;
    for i in 0..num_particles {
        let particle_angle =
            angle + (i as f32 / num_particles as f32 - 0.5) * std::f32::consts::PI * 0.5;
        let offset = Vec2::new(particle_angle.cos(), particle_angle.sin()) * 15.0;

        commands.spawn((
            ShieldRipple {
                lifetime: 0.3,
                max_lifetime: 0.3,
                max_radius: 20.0,
                angle: particle_angle,
            },
            Sprite {
                color: Color::srgba(0.4, 0.8, 1.0, 0.7),
                custom_size: Some(Vec2::splat(6.0)),
                ..default()
            },
            Transform::from_xyz(
                position.x + offset.x,
                position.y + offset.y,
                LAYER_EFFECTS + 0.8,
            ),
        ));
    }
}

/// Update shield ripple effects
pub fn update_shield_ripples(
    mut commands: Commands,
    time: Res<Time>,
    mut ripples: Query<(Entity, &mut ShieldRipple, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut ripple, mut sprite, mut transform) in ripples.iter_mut() {
        ripple.lifetime -= dt;

        if ripple.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Expand outward
        let progress = 1.0 - (ripple.lifetime / ripple.max_lifetime);
        let current_radius = ripple.max_radius * progress;
        sprite.custom_size = Some(Vec2::splat(current_radius * 2.0));

        // Fade out
        let alpha = (ripple.lifetime / ripple.max_lifetime) * 0.8;
        let current = sprite.color.to_srgba();
        sprite.color = Color::srgba(current.red, current.green, current.blue, alpha);

        // Slight rotation for visual interest
        transform.rotation = Quat::from_rotation_z(ripple.angle + progress * 0.5);
    }
}

/// Spawn armor spark particles
fn spawn_armor_sparks(commands: &mut Commands, position: Vec2, direction: Vec2, damage: f32) {
    let spark_count = (damage / 5.0).clamp(3.0, 12.0) as u32;
    let base_angle = direction.y.atan2(direction.x);

    for i in 0..spark_count {
        // Spread sparks in a cone opposite to damage direction
        let spread = (i as f32 / spark_count as f32 - 0.5) * std::f32::consts::PI * 0.6;
        let angle = base_angle + std::f32::consts::PI + spread + (fastrand::f32() - 0.5) * 0.3;
        let speed = 80.0 + fastrand::f32() * 120.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        let lifetime = 0.2 + fastrand::f32() * 0.3;

        commands.spawn((
            ArmorSpark {
                velocity,
                lifetime,
                max_lifetime: lifetime,
            },
            Sprite {
                color: Color::srgba(1.0, 0.7, 0.3, 1.0), // Orange/gold
                custom_size: Some(Vec2::new(4.0, 2.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 0.5)
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

/// Update armor spark particles
pub fn update_armor_sparks(
    mut commands: Commands,
    time: Res<Time>,
    mut sparks: Query<(Entity, &mut ArmorSpark, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut spark, mut sprite, mut transform) in sparks.iter_mut() {
        spark.lifetime -= dt;

        if spark.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Move with gravity
        transform.translation.x += spark.velocity.x * dt;
        transform.translation.y += spark.velocity.y * dt;
        spark.velocity.y -= 200.0 * dt; // Gravity

        // Fade and shrink
        let progress = spark.lifetime / spark.max_lifetime;
        let alpha = progress;
        let current = sprite.color.to_srgba();
        sprite.color = Color::srgba(
            current.red,
            current.green * progress, // Orange -> red as it fades
            current.blue * progress * 0.5,
            alpha,
        );

        // Shrink
        sprite.custom_size = Some(Vec2::new(4.0 * progress, 2.0 * progress));
    }
}

/// Spawn hull fire and smoke particles
fn spawn_hull_fire(commands: &mut Commands, position: Vec2, damage: f32) {
    let particle_count = (damage / 3.0).clamp(4.0, 15.0) as u32;

    for i in 0..particle_count {
        let is_smoke = i % 3 == 0; // Every 3rd particle is smoke

        let angle = fastrand::f32() * std::f32::consts::TAU;
        let speed = 20.0 + fastrand::f32() * 40.0;
        let velocity = Vec2::new(
            angle.cos() * speed * 0.3,
            speed + fastrand::f32() * 20.0, // Mostly upward
        );

        let lifetime = if is_smoke {
            0.6 + fastrand::f32() * 0.4
        } else {
            0.3 + fastrand::f32() * 0.3
        };

        let color = if is_smoke {
            Color::srgba(0.2, 0.2, 0.2, 0.7) // Dark smoke
        } else {
            Color::srgba(1.0, 0.4, 0.1, 0.9) // Orange fire
        };

        let size = if is_smoke {
            8.0 + fastrand::f32() * 6.0
        } else {
            4.0 + fastrand::f32() * 4.0
        };

        let offset = Vec2::new(
            (fastrand::f32() - 0.5) * 20.0,
            (fastrand::f32() - 0.5) * 20.0,
        );

        commands.spawn((
            HullFireParticle {
                velocity,
                lifetime,
                max_lifetime: lifetime,
                is_smoke,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(
                position.x + offset.x,
                position.y + offset.y,
                if is_smoke {
                    LAYER_EFFECTS + 0.3
                } else {
                    LAYER_EFFECTS + 0.4
                },
            ),
        ));
    }
}

/// Update hull fire and smoke particles
pub fn update_hull_fire(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut HullFireParticle, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut sprite, mut transform) in particles.iter_mut() {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Move upward (fire/smoke rises)
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Slow down horizontal movement
        particle.velocity.x *= 0.95;

        let progress = particle.lifetime / particle.max_lifetime;

        if particle.is_smoke {
            // Smoke expands and fades
            let current_size = sprite.custom_size.unwrap_or(Vec2::splat(8.0));
            sprite.custom_size = Some(current_size * (1.0 + dt * 2.0)); // Expand
            sprite.color = Color::srgba(0.2, 0.2, 0.2, progress * 0.7);
        } else {
            // Fire shrinks and changes color (orange -> red -> dark)
            let current = sprite.color.to_srgba();
            sprite.color = Color::srgba(
                current.red,
                current.green * 0.95, // Yellow -> red
                current.blue * 0.9,
                progress * 0.9,
            );
            sprite.custom_size = Some(Vec2::splat(4.0 + 4.0 * progress));
        }
    }
}
