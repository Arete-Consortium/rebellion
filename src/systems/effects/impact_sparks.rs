//! Bullet-impact spark burst — short radial particles in weapon-family color.

use crate::core::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct ImpactSpark {
    pub velocity: Vec2,
    pub life: f32,
    pub max: f32,
}

/// Impact spark color based on damage type (EM/Kinetic/Thermal/Explosive).
pub fn spark_color_from_damage(damage_type: DamageType) -> Color {
    match damage_type {
        DamageType::EM => Color::srgba(0.7, 0.95, 1.0, 1.0),       // EDENCOM/Laser
        DamageType::Thermal => Color::srgba(1.0, 0.45, 0.15, 1.0), // Disintegrator/drone
        DamageType::Kinetic => Color::srgba(1.0, 0.85, 0.35, 1.0), // Autocannon/rail
        DamageType::Explosive => Color::srgba(1.0, 0.55, 0.25, 1.0), // Missile
    }
}

/// Spawn 6 radial sparks at `pos`. Sparks shrink and fade over ~0.25s.
pub fn spawn_impact_sparks(commands: &mut Commands, pos: Vec2, damage_type: DamageType) {
    const COUNT: usize = 6;
    let color = spark_color_from_damage(damage_type);
    let base_speed = 180.0;
    for i in 0..COUNT {
        let angle = (i as f32) / COUNT as f32 * std::f32::consts::TAU
            + (fastrand::f32() - 0.5) * 0.4;
        let speed = base_speed * (0.65 + fastrand::f32() * 0.5);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
        let life = 0.18 + fastrand::f32() * 0.12;
        commands.spawn((
            ImpactSpark {
                velocity,
                life,
                max: life,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::new(3.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, LAYER_EFFECTS + 1.0)
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

/// Tick impact sparks: drift with velocity, shrink + fade.
pub fn tick_impact_sparks(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut ImpactSpark, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut spark, mut tf, mut sprite) in q.iter_mut() {
        spark.life -= dt;
        if spark.life <= 0.0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        tf.translation.x += spark.velocity.x * dt;
        tf.translation.y += spark.velocity.y * dt;
        // Decelerate — arcade-y "flicker" feel.
        spark.velocity *= (1.0 - dt * 3.0).max(0.0);
        let t = spark.life / spark.max;
        sprite.color.set_alpha(t);
        if let Some(ref mut size) = sprite.custom_size {
            let s = 3.0 * (0.4 + 0.6 * t);
            *size = Vec2::splat(s);
        }
    }
}
