//! Starfield background effect

use crate::core::*;
use bevy::prelude::*;

/// Marker for star entities
#[derive(Component)]
pub struct Star {
    pub speed: f32,
    pub layer: u8,
}

/// Spawn scrolling starfield background
pub fn spawn_starfield(mut commands: Commands) {
    let mut rng = fastrand::Rng::new();

    // Spawn stars in 3 layers (parallax)
    for layer in 0..3 {
        let count = match layer {
            0 => 30, // Far stars (dim, slow)
            1 => 50, // Mid stars
            _ => 70, // Near stars (bright, fast)
        };

        let (speed, size, alpha) = match layer {
            0 => (20.0, 1.0, 0.3),
            1 => (40.0, 1.5, 0.5),
            _ => (80.0, 2.5, 0.8),
        };

        for _ in 0..count {
            let x = rng.f32() * SCREEN_WIDTH - SCREEN_WIDTH / 2.0;
            let y = rng.f32() * SCREEN_HEIGHT - SCREEN_HEIGHT / 2.0;

            commands.spawn((
                Star { speed, layer },
                Sprite {
                    color: Color::srgba(0.8, 0.85, 1.0, alpha),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz(x, y, layer as f32),
            ));
        }
    }
}

/// Scroll stars downward
pub fn update_starfield(time: Res<Time>, mut query: Query<(&mut Transform, &Star)>) {
    let dt = time.delta_secs();

    for (mut transform, star) in query.iter_mut() {
        transform.translation.y -= star.speed * dt;

        // Wrap around
        if transform.translation.y < -SCREEN_HEIGHT / 2.0 - 10.0 {
            transform.translation.y = SCREEN_HEIGHT / 2.0 + 10.0;
            transform.translation.x = fastrand::f32() * SCREEN_WIDTH - SCREEN_WIDTH / 2.0;
        }
    }
}
