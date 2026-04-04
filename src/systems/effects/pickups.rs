//! Pickup visual effects

use bevy::prelude::*;
use crate::core::*;
use crate::entities::collectible::Rarity;
use super::screen_effects::{ScreenShake, ScreenFlash};

/// Maximum pickup effect particles at once
const MAX_PICKUP_PARTICLES: usize = 100;

/// Pickup flash - instant bright burst at collection point
#[derive(Component)]
pub struct PickupFlash {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: Color,
}

/// Pickup shockwave - expanding ring
#[derive(Component)]
pub struct PickupShockwave {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub max_radius: f32,
    pub color: Color,
}

/// Pickup particle - color-matched burst particle
#[derive(Component)]
pub struct PickupParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub color: Color,
}

/// Handle pickup effect events
pub fn handle_pickup_effect_events(
    mut commands: Commands,
    mut events: EventReader<PickupEffectEvent>,
    particle_query: Query<&PickupParticle>,
    mut screen_shake: ResMut<ScreenShake>,
    mut screen_flash: ResMut<ScreenFlash>,
) {
    let current_particles = particle_query.iter().count();

    for event in events.read() {
        let rarity = Rarity::for_collectible(event.collectible_type);

        // Only spawn particles if under cap (always spawn flash/shockwave for feedback)
        if current_particles < MAX_PICKUP_PARTICLES {
            spawn_pickup_effects(&mut commands, event.position, event.color, rarity);
        }

        // Screen effects scale with rarity (always play these for feedback)
        match rarity {
            Rarity::Epic => {
                screen_shake.trigger(5.0, 0.12);
                screen_flash.colored(event.color, 0.3);
            }
            Rarity::Rare => {
                screen_shake.trigger(3.0, 0.08);
                screen_flash.colored(event.color, 0.2);
            }
            Rarity::Uncommon => {
                screen_shake.trigger(1.5, 0.05);
            }
            Rarity::Common => {
                // No screen shake for common pickups
            }
        }
    }
}

/// Spawn all pickup visual effects
fn spawn_pickup_effects(commands: &mut Commands, position: Vec2, color: Color, rarity: Rarity) {
    let intensity = rarity.glow_mult();
    let particle_count = rarity.orbital_count();

    // Phase 1: Instant flash
    let flash_size = 40.0 * intensity;
    let flash_lifetime = 0.1 + 0.05 * intensity;

    commands.spawn((
        PickupFlash {
            lifetime: flash_lifetime,
            max_lifetime: flash_lifetime,
            color,
        },
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.9), // Start white
            custom_size: Some(Vec2::splat(flash_size)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 2.0),
    ));

    // Phase 2: Shockwave ring
    let shockwave_radius = 60.0 * intensity;
    let shockwave_lifetime = 0.3 + 0.1 * intensity;

    commands.spawn((
        PickupShockwave {
            lifetime: shockwave_lifetime,
            max_lifetime: shockwave_lifetime,
            max_radius: shockwave_radius,
            color,
        },
        Sprite {
            color: color.with_alpha(0.8),
            custom_size: Some(Vec2::splat(10.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.5),
    ));

    // Secondary inner shockwave for rare/epic
    if matches!(rarity, Rarity::Rare | Rarity::Epic) {
        commands.spawn((
            PickupShockwave {
                lifetime: shockwave_lifetime * 0.7,
                max_lifetime: shockwave_lifetime * 0.7,
                max_radius: shockwave_radius * 0.6,
                color,
            },
            Sprite {
                color: Color::WHITE.with_alpha(0.6),
                custom_size: Some(Vec2::splat(8.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.6),
        ));
    }

    // Phase 3: Particle burst
    let base_speed = 100.0 * intensity;
    let particle_lifetime = 0.4 + 0.2 * intensity;

    for i in 0..particle_count {
        let angle =
            (i as f32 / particle_count as f32) * std::f32::consts::TAU + fastrand::f32() * 0.3;
        let speed = base_speed * (0.7 + fastrand::f32() * 0.6);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        let lifetime = particle_lifetime * (0.7 + fastrand::f32() * 0.6);
        let size = 4.0 + 3.0 * intensity * fastrand::f32();

        // Color variation - mix with white for sparkle
        let sparkle = fastrand::f32() * 0.4;
        let c = color.to_srgba();
        let particle_color = Color::srgba(
            (c.red + sparkle).min(1.0),
            (c.green + sparkle).min(1.0),
            (c.blue + sparkle).min(1.0),
            0.9,
        );

        commands.spawn((
            PickupParticle {
                velocity,
                lifetime,
                max_lifetime: lifetime,
                color: particle_color,
            },
            Sprite {
                color: particle_color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.0),
        ));
    }

    // Extra sparkle particles for epic rarity
    if rarity == Rarity::Epic {
        for _ in 0..6 {
            let angle = fastrand::f32() * std::f32::consts::TAU;
            let speed = base_speed * 1.5 * (0.8 + fastrand::f32() * 0.4);
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

            commands.spawn((
                PickupParticle {
                    velocity,
                    lifetime: particle_lifetime * 1.2,
                    max_lifetime: particle_lifetime * 1.2,
                    color: Color::WHITE,
                },
                Sprite {
                    color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::splat(6.0)),
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.1),
            ));
        }
    }
}

/// Update pickup flash effects
pub fn update_pickup_flashes(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut PickupFlash, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut flash, mut sprite, mut transform) in flashes.iter_mut() {
        flash.lifetime -= dt;

        if flash.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = flash.lifetime / flash.max_lifetime;

        // Expand rapidly then shrink
        let scale = if progress > 0.5 {
            // First half: expand
            1.0 + (1.0 - progress) * 2.0
        } else {
            // Second half: shrink
            progress * 2.0
        };
        transform.scale = Vec3::splat(scale);

        // Fade from white to color to transparent
        let c = flash.color.to_srgba();
        let white_blend = progress;
        sprite.color = Color::srgba(
            c.red + (1.0 - c.red) * white_blend,
            c.green + (1.0 - c.green) * white_blend,
            c.blue + (1.0 - c.blue) * white_blend,
            progress * 0.9,
        );
    }
}

/// Update pickup shockwave effects
pub fn update_pickup_shockwaves(
    mut commands: Commands,
    time: Res<Time>,
    mut shockwaves: Query<(Entity, &mut PickupShockwave, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut wave, mut sprite) in shockwaves.iter_mut() {
        wave.lifetime -= dt;

        if wave.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = 1.0 - (wave.lifetime / wave.max_lifetime);

        // Expand outward (ease-out curve)
        let ease_progress = 1.0 - (1.0 - progress).powi(2);
        let current_radius = wave.max_radius * ease_progress;
        sprite.custom_size = Some(Vec2::splat(current_radius * 2.0));

        // Fade out
        let alpha = (1.0 - progress) * 0.8;
        let c = wave.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

/// Update pickup particle effects
pub fn update_pickup_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut PickupParticle, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut sprite, mut transform) in particles.iter_mut() {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Move with slight deceleration
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;
        particle.velocity *= 0.96; // Slow down

        let progress = particle.lifetime / particle.max_lifetime;

        // Fade and shrink
        let c = particle.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, progress * 0.9);

        let current_size = sprite.custom_size.unwrap_or(Vec2::splat(4.0));
        sprite.custom_size = Some(current_size * (0.98 + progress * 0.02));
    }
}
