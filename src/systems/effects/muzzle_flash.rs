//! Muzzle flash VFX — short-lived, weapon-family-colored sprite spawned at
//! the player's firing origin to punch up the "fire" moment.

use crate::core::*;
use bevy::prelude::*;

/// Short-lived flash sprite. Tick system shrinks + fades over `max` seconds.
#[derive(Component)]
pub struct MuzzleFlash {
    pub life: f32,
    pub max: f32,
    pub start_size: f32,
}

/// Color palette per weapon family. Muzzle flashes read as instant bursts of
/// the weapon's signature color so each hull's fire feels distinct.
pub fn muzzle_flash_color(weapon_type: WeaponType) -> Color {
    match weapon_type {
        WeaponType::Laser => Color::srgba(1.0, 0.95, 0.55, 0.95),
        WeaponType::Autocannon | WeaponType::Artillery => Color::srgba(1.0, 0.75, 0.35, 0.95),
        WeaponType::Railgun => Color::srgba(0.55, 0.85, 1.0, 0.95),
        WeaponType::MissileLauncher => Color::srgba(1.0, 0.55, 0.25, 0.95),
        WeaponType::Drone => Color::srgba(0.55, 1.0, 0.5, 0.95),
        WeaponType::Disintegrator => Color::srgba(1.0, 0.4, 0.15, 0.95),
        WeaponType::Vorton => Color::srgba(0.75, 0.95, 1.0, 0.95),
    }
}

/// Spawn a muzzle flash sprite at `pos` oriented along `direction`.
pub fn spawn_muzzle_flash(
    commands: &mut Commands,
    pos: Vec2,
    direction: Vec2,
    weapon_type: WeaponType,
) {
    let color = muzzle_flash_color(weapon_type);
    // Size tuned per weapon so big guns (disintegrator/vorton) feel weightier.
    let size = match weapon_type {
        WeaponType::Disintegrator | WeaponType::Vorton => 28.0,
        WeaponType::MissileLauncher => 22.0,
        WeaponType::Railgun => 24.0,
        _ => 18.0,
    };
    let angle = direction.y.atan2(direction.x);
    commands.spawn((
        MuzzleFlash {
            life: 0.09,
            max: 0.09,
            start_size: size,
        },
        Sprite {
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, LAYER_PLAYER_BULLETS + 0.2)
            .with_rotation(Quat::from_rotation_z(angle)),
    ));
}

/// Tick muzzle flashes: shrink + fade, despawn when life expires.
pub fn tick_muzzle_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut MuzzleFlash, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut flash, mut sprite, mut tf) in q.iter_mut() {
        flash.life -= dt;
        if flash.life <= 0.0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        let t = flash.life / flash.max;
        // Shrink from 1.0 → 0.4 as flash dies; alpha tracks life linearly.
        let scale_factor = 0.4 + 0.6 * t;
        if let Some(ref mut size) = sprite.custom_size {
            let s = flash.start_size * scale_factor;
            *size = Vec2::splat(s);
        }
        sprite.color.set_alpha(t);
        tf.scale = Vec3::splat(1.0);
    }
}
