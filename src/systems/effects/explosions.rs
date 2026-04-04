//! Explosion particle effects

use super::MAX_EXPLOSION_PARTICLES;
use crate::core::*;
use bevy::prelude::*;

/// Explosion particle
#[derive(Component)]
pub struct ExplosionParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Shockwave ring - expanding circular ring effect
#[derive(Component)]
pub struct ShockwaveRing {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub max_radius: f32,
}

/// Explosion flash - bright center glow
#[derive(Component)]
pub struct ExplosionFlash {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Explosion ember - slow-moving sparks/debris
#[derive(Component)]
pub struct ExplosionEmber {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Handle explosion events with particle cap
pub fn handle_explosion_events(
    mut commands: Commands,
    mut events: EventReader<ExplosionEvent>,
    particle_query: Query<&ExplosionParticle>,
    mut rumble_events: EventWriter<crate::systems::joystick::RumbleRequest>,
) {
    let current_count = particle_query.iter().count();
    let mut spawned = 0;

    for event in events.read() {
        // Check particle cap before spawning
        if current_count + spawned < MAX_EXPLOSION_PARTICLES {
            let new_count =
                spawn_explosion_capped(&mut commands, event.position, &event.size, event.color);
            spawned += new_count;

            // Trigger rumble based on explosion size (only for large+ explosions to avoid spam)
            match event.size {
                ExplosionSize::Large => {
                    rumble_events.send(crate::systems::joystick::RumbleRequest::explosion());
                }
                ExplosionSize::Massive => {
                    rumble_events.send(crate::systems::joystick::RumbleRequest::big_explosion());
                }
                _ => {} // No rumble for tiny/small/medium (too spammy)
            }
        }
    }
}

/// Spawn explosion particles (returns count spawned)
fn spawn_explosion_capped(
    commands: &mut Commands,
    position: Vec2,
    size: &ExplosionSize,
    color: Color,
) -> usize {
    let (count, speed, lifetime, particle_size) = match size {
        ExplosionSize::Tiny => (5, 50.0, 0.2, 3.0),
        ExplosionSize::Small => (12, 100.0, 0.4, 5.0),
        ExplosionSize::Medium => (20, 150.0, 0.5, 7.0),
        ExplosionSize::Large => (30, 200.0, 0.6, 10.0),
        ExplosionSize::Massive => (50, 300.0, 0.8, 15.0),
    };

    let mut rng = fastrand::Rng::new();

    // Main explosion particles - hot colors in center, cooler at edges
    for i in 0..count {
        let angle = rng.f32() * std::f32::consts::TAU;
        let speed_var = speed * (0.5 + rng.f32() * 0.5);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed_var;

        // Color gradient: inner particles are brighter/hotter
        let inner_factor = 1.0 - (i as f32 / count as f32);
        let color_var = if inner_factor > 0.5 {
            // Inner particles: white/yellow hot core
            Color::srgba(1.0, 0.9 + rng.f32() * 0.1, 0.5 + rng.f32() * 0.3, 1.0)
        } else {
            // Outer particles: orange/red
            Color::srgba(
                color.to_srgba().red * (0.8 + rng.f32() * 0.4),
                color.to_srgba().green * (0.5 + rng.f32() * 0.3),
                color.to_srgba().blue * (0.2 + rng.f32() * 0.2),
                1.0,
            )
        };

        commands.spawn((
            ExplosionParticle {
                velocity,
                lifetime,
                max_lifetime: lifetime,
            },
            Sprite {
                color: color_var,
                custom_size: Some(Vec2::splat(particle_size * (0.5 + rng.f32() * 0.5))),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS),
        ));
    }

    // Spawn shockwave ring for medium+ explosions
    if matches!(
        size,
        ExplosionSize::Medium | ExplosionSize::Large | ExplosionSize::Massive
    ) {
        let ring_lifetime = match size {
            ExplosionSize::Medium => 0.3,
            ExplosionSize::Large => 0.4,
            ExplosionSize::Massive => 0.5,
            _ => 0.3,
        };
        let ring_radius = match size {
            ExplosionSize::Medium => 60.0,
            ExplosionSize::Large => 100.0,
            ExplosionSize::Massive => 150.0,
            _ => 60.0,
        };

        commands.spawn((
            ShockwaveRing {
                lifetime: ring_lifetime,
                max_lifetime: ring_lifetime,
                max_radius: ring_radius,
            },
            Sprite {
                color: Color::srgba(1.0, 0.8, 0.4, 0.8),
                custom_size: Some(Vec2::splat(10.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 0.1),
        ));
    }

    // Spawn center flash for small+ explosions
    if !matches!(size, ExplosionSize::Tiny) {
        let flash_lifetime = match size {
            ExplosionSize::Small => 0.1,
            ExplosionSize::Medium => 0.15,
            ExplosionSize::Large => 0.2,
            ExplosionSize::Massive => 0.25,
            _ => 0.1,
        };
        let flash_size = match size {
            ExplosionSize::Small => 20.0,
            ExplosionSize::Medium => 35.0,
            ExplosionSize::Large => 50.0,
            ExplosionSize::Massive => 80.0,
            _ => 20.0,
        };

        commands.spawn((
            ExplosionFlash {
                lifetime: flash_lifetime,
                max_lifetime: flash_lifetime,
            },
            Sprite {
                color: Color::srgba(1.0, 1.0, 0.9, 1.0), // Bright white-yellow
                custom_size: Some(Vec2::splat(flash_size)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 0.2),
        ));
    }

    // Spawn embers/sparks for medium+ explosions
    if matches!(
        size,
        ExplosionSize::Medium | ExplosionSize::Large | ExplosionSize::Massive
    ) {
        let ember_count = match size {
            ExplosionSize::Medium => 5,
            ExplosionSize::Large => 8,
            ExplosionSize::Massive => 12,
            _ => 5,
        };

        for _ in 0..ember_count {
            let angle = rng.f32() * std::f32::consts::TAU;
            let ember_speed = speed * 0.3 * (0.5 + rng.f32() * 0.5);
            let velocity = Vec2::new(angle.cos(), angle.sin()) * ember_speed;

            commands.spawn((
                ExplosionEmber {
                    velocity,
                    lifetime: lifetime * 2.0, // Embers last longer
                    max_lifetime: lifetime * 2.0,
                },
                Sprite {
                    color: Color::srgba(1.0, 0.6 + rng.f32() * 0.3, 0.1, 1.0), // Orange-yellow sparks
                    custom_size: Some(Vec2::splat(2.0 + rng.f32() * 2.0)),
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, LAYER_EFFECTS - 0.1),
            ));
        }
    }

    count
}

/// Spawn explosion particles (legacy public API)
pub fn spawn_explosion(
    commands: &mut Commands,
    position: Vec2,
    size: &ExplosionSize,
    color: Color,
) {
    spawn_explosion_capped(commands, position, size, color);
}

/// Update explosion particles
pub fn update_explosions(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ExplosionParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        // Move
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Slow down
        particle.velocity *= 1.0 - 3.0 * dt;

        // Fade out
        particle.lifetime -= dt;
        let alpha = (particle.lifetime / particle.max_lifetime).max(0.0);
        sprite.color = sprite.color.with_alpha(alpha);

        // Shrink
        if let Some(size) = sprite.custom_size {
            sprite.custom_size = Some(size * (1.0 - 0.5 * dt));
        }

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Update shockwave rings - expand and fade
pub fn update_shockwave_rings(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShockwaveRing, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut ring, mut sprite, mut transform) in query.iter_mut() {
        ring.lifetime -= dt;

        let progress = 1.0 - (ring.lifetime / ring.max_lifetime);
        let current_radius = ring.max_radius * progress;

        // Expand the ring
        sprite.custom_size = Some(Vec2::splat(current_radius * 2.0));

        // Make it hollow by reducing alpha and using scale
        // Ring gets thinner as it expands
        let alpha = (1.0 - progress) * 0.6;
        sprite.color = sprite.color.with_alpha(alpha);

        // Slight upward drift for visual interest
        transform.translation.y += 10.0 * dt;

        if ring.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Update explosion flashes - quick bright flash that fades
pub fn update_explosion_flashes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionFlash, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut flash, mut sprite) in query.iter_mut() {
        flash.lifetime -= dt;

        // Quick fade out with size pulse
        let progress = flash.lifetime / flash.max_lifetime;
        let alpha = progress * progress; // Quadratic fade for snappy feel
        sprite.color = sprite.color.with_alpha(alpha);

        // Pulse size slightly larger then shrink
        if let Some(size) = sprite.custom_size {
            let scale = 1.0 + (1.0 - progress) * 0.5; // Grows slightly as it fades
            sprite.custom_size = Some(size * (1.0 + 0.5 * dt));
            if progress < 0.5 {
                sprite.custom_size = Some(Vec2::splat(size.x * (0.8 + progress * 0.4)));
            }
            let _ = scale; // suppress warning
        }

        if flash.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Update explosion embers - slow drifting sparks
pub fn update_explosion_embers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionEmber, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut ember, mut sprite, mut transform) in query.iter_mut() {
        // Move slowly
        transform.translation.x += ember.velocity.x * dt;
        transform.translation.y += ember.velocity.y * dt;

        // Gentle gravity (embers drift down slightly)
        ember.velocity.y -= 20.0 * dt;

        // Very slow deceleration
        ember.velocity *= 1.0 - 0.5 * dt;

        // Fade out
        ember.lifetime -= dt;
        let alpha = (ember.lifetime / ember.max_lifetime).max(0.0);
        sprite.color = sprite.color.with_alpha(alpha);

        // Flicker effect - random brightness variation
        let flicker = 0.7 + fastrand::f32() * 0.3;
        let base = sprite.color.to_srgba();
        sprite.color = Color::srgba(base.red * flicker, base.green * flicker, base.blue, alpha);

        if ember.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
