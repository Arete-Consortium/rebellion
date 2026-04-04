//! Bullet trails and engine trails

use bevy::prelude::*;
use crate::core::*;

// =============================================================================
// BULLET TRAILS
// =============================================================================

/// Component for projectiles that emit trails
#[derive(Component)]
pub struct BulletTrail {
    /// Trail color
    pub color: Color,
    /// Spawn rate (particles per second)
    pub spawn_rate: f32,
    /// Timer for spawning
    pub spawn_timer: f32,
}

impl BulletTrail {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            spawn_rate: 40.0,
            spawn_timer: 0.0,
        }
    }
}

/// Bullet trail particle
#[derive(Component)]
pub struct BulletTrailParticle {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Spawn bullet trail particles from projectiles
pub fn spawn_bullet_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut BulletTrail)>,
    particle_count: Query<&BulletTrailParticle>,
) {
    // Cap trail particles to avoid performance issues
    const MAX_TRAIL_PARTICLES: usize = 300;
    if particle_count.iter().count() >= MAX_TRAIL_PARTICLES {
        return;
    }

    let dt = time.delta_secs();

    for (transform, mut trail) in query.iter_mut() {
        trail.spawn_timer += dt;
        let spawn_interval = 1.0 / trail.spawn_rate;

        while trail.spawn_timer >= spawn_interval {
            trail.spawn_timer -= spawn_interval;

            let pos = transform.translation.truncate();
            let lifetime = 0.15;

            // Spawn fading particle
            commands.spawn((
                BulletTrailParticle {
                    lifetime,
                    max_lifetime: lifetime,
                },
                Sprite {
                    color: trail.color.with_alpha(0.6),
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, LAYER_EFFECTS - 2.0),
            ));
        }
    }
}

/// Update bullet trail particles (fade and despawn)
pub fn update_bullet_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BulletTrailParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Fade out and shrink
            let alpha = (particle.lifetime / particle.max_lifetime) * 0.6;
            sprite.color = sprite.color.with_alpha(alpha);

            if let Some(size) = sprite.custom_size {
                sprite.custom_size = Some(size * (1.0 - dt * 4.0));
            }
        }
    }
}

// =============================================================================
// ENGINE TRAILS
// =============================================================================

/// Component for entities that emit engine trails
#[derive(Component)]
pub struct EngineTrail {
    /// Trail color (faction-based)
    pub color: Color,
    /// Spawn rate (particles per second)
    pub spawn_rate: f32,
    /// Timer for spawning
    pub spawn_timer: f32,
    /// Offset from entity center (engine position)
    pub offset: Vec2,
    /// Whether trail is active (moving)
    pub active: bool,
}

impl Default for EngineTrail {
    fn default() -> Self {
        Self {
            color: Color::srgba(0.4, 0.7, 1.0, 0.9), // Blue engine glow
            spawn_rate: 60.0,
            spawn_timer: 0.0,
            offset: Vec2::new(0.0, -25.0), // Behind ship
            active: true,
        }
    }
}

impl EngineTrail {
    /// Minmatar rust-orange engine
    pub fn minmatar() -> Self {
        Self {
            color: Color::srgba(1.0, 0.5, 0.2, 0.9),
            ..default()
        }
    }

    /// Amarr golden engine
    pub fn amarr() -> Self {
        Self {
            color: Color::srgba(1.0, 0.85, 0.3, 0.9),
            ..default()
        }
    }

    /// Caldari blue engine
    pub fn caldari() -> Self {
        Self {
            color: Color::srgba(0.3, 0.6, 1.0, 0.9),
            ..default()
        }
    }

    /// Gallente green engine
    pub fn gallente() -> Self {
        Self {
            color: Color::srgba(0.3, 0.9, 0.5, 0.9),
            ..default()
        }
    }

    /// Create engine trail from faction
    pub fn from_faction(faction: crate::core::Faction) -> Self {
        Self {
            color: faction.engine_color(),
            ..default()
        }
    }
}

/// Engine trail particle
#[derive(Component)]
pub struct EngineParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub base_color: Color,
    pub is_core: bool, // Core particles are brighter/smaller
}

/// Spawn engine trail particles from entities with EngineTrail
pub fn spawn_engine_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut EngineTrail)>,
) {
    let dt = time.delta_secs();

    for (transform, mut trail) in query.iter_mut() {
        if !trail.active {
            continue;
        }

        trail.spawn_timer += dt;
        let spawn_interval = 1.0 / trail.spawn_rate;

        while trail.spawn_timer >= spawn_interval {
            trail.spawn_timer -= spawn_interval;

            // Calculate spawn position with offset
            let rotation = transform.rotation.to_euler(EulerRot::ZYX).0;
            let rotated_offset = Vec2::new(
                trail.offset.x * rotation.cos() - trail.offset.y * rotation.sin(),
                trail.offset.x * rotation.sin() + trail.offset.y * rotation.cos(),
            );
            let spawn_pos = transform.translation.truncate() + rotated_offset;

            // Exhaust direction (opposite of ship facing)
            let exhaust_dir = Vec2::new(-rotation.sin(), -rotation.cos());

            // Spawn core particle (bright, small, short-lived)
            let core_spread = 2.0;
            let core_offset = Vec2::new(
                (fastrand::f32() - 0.5) * core_spread,
                (fastrand::f32() - 0.5) * core_spread,
            );
            let core_vel = exhaust_dir * (60.0 + fastrand::f32() * 30.0);
            let core_lifetime = 0.08 + fastrand::f32() * 0.06;

            commands.spawn((
                EngineParticle {
                    velocity: core_vel,
                    lifetime: core_lifetime,
                    max_lifetime: core_lifetime,
                    base_color: trail.color,
                    is_core: true,
                },
                Sprite {
                    color: Color::srgba(1.0, 1.0, 0.95, 1.0), // Hot white core
                    custom_size: Some(Vec2::new(3.0, 5.0)),   // Elongated
                    ..default()
                },
                Transform::from_xyz(
                    spawn_pos.x + core_offset.x,
                    spawn_pos.y + core_offset.y,
                    LAYER_EFFECTS - 0.5,
                )
                .with_rotation(Quat::from_rotation_z(rotation)),
            ));

            // Spawn outer glow particle (faction color, larger, longer-lived)
            let glow_spread = 6.0;
            let glow_offset = Vec2::new(
                (fastrand::f32() - 0.5) * glow_spread,
                (fastrand::f32() - 0.5) * glow_spread,
            );
            let glow_vel = exhaust_dir * (40.0 + fastrand::f32() * 40.0)
                + Vec2::new(
                    (fastrand::f32() - 0.5) * 20.0,
                    (fastrand::f32() - 0.5) * 20.0,
                );
            let glow_lifetime = 0.15 + fastrand::f32() * 0.15;
            let glow_size = 4.0 + fastrand::f32() * 4.0;

            commands.spawn((
                EngineParticle {
                    velocity: glow_vel,
                    lifetime: glow_lifetime,
                    max_lifetime: glow_lifetime,
                    base_color: trail.color,
                    is_core: false,
                },
                Sprite {
                    color: trail.color.with_alpha(0.7),
                    custom_size: Some(Vec2::splat(glow_size)),
                    ..default()
                },
                Transform::from_xyz(
                    spawn_pos.x + glow_offset.x,
                    spawn_pos.y + glow_offset.y,
                    LAYER_EFFECTS - 1.0,
                ),
            ));
        }
    }
}

/// Update engine trail particles
pub fn update_engine_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut EngineParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        // Move
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Slow down (core slows faster)
        let drag = if particle.is_core { 8.0 } else { 4.0 };
        particle.velocity *= 1.0 - drag * dt;

        // Update lifetime
        particle.lifetime -= dt;
        let progress = particle.lifetime / particle.max_lifetime;

        if particle.is_core {
            // Core: fade from white to faction color, then fade out
            let base = particle.base_color.to_srgba();
            let r = 1.0 * progress + base.red * (1.0 - progress);
            let g = 1.0 * progress + base.green * (1.0 - progress);
            let b = 0.9 * progress + base.blue * (1.0 - progress);
            sprite.color = Color::srgba(r, g, b, progress);
        } else {
            // Glow: just fade out
            sprite.color = particle.base_color.with_alpha(progress * 0.7);
        }

        // Shrink
        if let Some(size) = sprite.custom_size {
            let shrink_rate = if particle.is_core { 3.0 } else { 1.5 };
            sprite.custom_size = Some(size * (1.0 - shrink_rate * dt));
        }

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
