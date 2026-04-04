//! Combat feedback: damage numbers and hit flash

use bevy::prelude::*;
use bevy::text::{Text2d, TextColor, TextFont};
use crate::core::*;

// =============================================================================
// DAMAGE NUMBERS
// =============================================================================

/// Floating damage number that rises and fades
#[derive(Component)]
pub struct DamageNumber {
    /// Upward velocity
    pub velocity: Vec2,
    /// Time remaining
    pub lifetime: f32,
    /// Max lifetime for fade calculation
    pub max_lifetime: f32,
}

impl DamageNumber {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::new(
                (fastrand::f32() - 0.5) * 30.0, // Random horizontal drift
                80.0,                           // Rise upward
            ),
            lifetime: 0.8,
            max_lifetime: 0.8,
        }
    }
}

impl Default for DamageNumber {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn a floating damage number at position
pub fn spawn_damage_number(commands: &mut Commands, position: Vec2, damage: f32, is_crit: bool) {
    let text = format!("{:.0}", damage);
    let (color, size) = if is_crit {
        (Color::srgb(1.0, 0.9, 0.2), 18.0) // Yellow, larger for crits
    } else if damage >= 20.0 {
        (Color::srgb(1.0, 0.5, 0.2), 16.0) // Orange for heavy hits
    } else {
        (Color::srgb(1.0, 1.0, 1.0), 14.0) // White for normal
    };

    commands.spawn((
        DamageNumber::new(),
        Text2d::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(position.x, position.y + 20.0, LAYER_EFFECTS + 5.0),
    ));
}

/// Update damage number positions and fade
pub fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber, &mut TextColor)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut dmg, mut color) in query.iter_mut() {
        // Move upward
        transform.translation.x += dmg.velocity.x * dt;
        transform.translation.y += dmg.velocity.y * dt;

        // Slow down horizontal drift
        dmg.velocity.x *= 1.0 - 3.0 * dt;

        // Update lifetime
        dmg.lifetime -= dt;
        let alpha = (dmg.lifetime / dmg.max_lifetime).max(0.0);

        // Fade out
        color.0 = color.0.with_alpha(alpha);

        // Scale up slightly as it rises
        let scale = 1.0 + (1.0 - alpha) * 0.3;
        transform.scale = Vec3::splat(scale);

        if dmg.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// HIT FLASH
// =============================================================================

/// Component that makes a sprite flash white when damaged
#[derive(Component)]
pub struct HitFlash {
    /// Time remaining for flash effect
    pub timer: f32,
    /// Total duration of flash
    pub duration: f32,
    /// Original sprite color (to restore after flash)
    pub original_color: Color,
}

impl HitFlash {
    /// Create a new hit flash effect
    pub fn new(original_color: Color) -> Self {
        Self {
            timer: 0.1,
            duration: 0.1,
            original_color,
        }
    }

    /// Create a hit flash with custom duration
    pub fn with_duration(original_color: Color, duration: f32) -> Self {
        Self {
            timer: duration,
            duration,
            original_color,
        }
    }
}

/// Update hit flash effects on sprites
pub fn update_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut HitFlash)>,
) {
    let dt = time.delta_secs();

    for (entity, mut sprite, mut flash) in query.iter_mut() {
        flash.timer -= dt;

        if flash.timer > 0.0 {
            // Lerp from white to original color
            let progress = 1.0 - (flash.timer / flash.duration);
            let white = Color::WHITE;
            let original = flash.original_color;

            // Simple lerp between white and original
            let r = white.to_srgba().red * (1.0 - progress) + original.to_srgba().red * progress;
            let g =
                white.to_srgba().green * (1.0 - progress) + original.to_srgba().green * progress;
            let b = white.to_srgba().blue * (1.0 - progress) + original.to_srgba().blue * progress;
            let a = original.to_srgba().alpha;

            sprite.color = Color::srgba(r, g, b, a);
        } else {
            // Flash complete, restore original and remove component
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}
