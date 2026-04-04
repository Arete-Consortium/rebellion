//! Overlay effects: salt miner tint, low health vignette, disintegrator beams

use crate::core::*;
use crate::entities::{Player, ShipStats};
use bevy::prelude::*;

// =============================================================================
// SALT MINER SCREEN TINT
// =============================================================================

/// Marker component for salt miner tint overlay
#[derive(Component)]
pub struct SaltMinerTintOverlay;

/// Salt Miner screen tint effect - red tint while salt miner is active
pub fn update_salt_miner_tint(
    mut commands: Commands,
    salt_miner: Res<SaltMinerSystem>,
    mut overlay_query: Query<(Entity, &mut Sprite), With<SaltMinerTintOverlay>>,
) {
    if salt_miner.is_active {
        // Pulse the tint based on remaining time
        let pulse = (salt_miner.timer * 8.0).sin().abs() * 0.1;
        let alpha = 0.15 + pulse;

        if let Ok((_, mut sprite)) = overlay_query.get_single_mut() {
            sprite.color = Color::srgba(1.0, 0.1, 0.1, alpha);
        } else {
            // Spawn tint overlay
            commands.spawn((
                SaltMinerTintOverlay,
                Sprite {
                    color: Color::srgba(1.0, 0.1, 0.1, alpha),
                    custom_size: Some(Vec2::new(SCREEN_WIDTH + 100.0, SCREEN_HEIGHT + 100.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, LAYER_HUD + 5.0), // Below flash, above game
            ));
        }
    } else {
        // Remove tint when salt miner ends
        for (entity, _) in overlay_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// LOW HEALTH WARNING VIGNETTE
// =============================================================================

/// Marker component for low health vignette overlay
#[derive(Component)]
pub struct LowHealthVignette;

/// Low health warning vignette - pulsing red edges when health is critical
pub fn update_low_health_vignette(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&ShipStats, With<Player>>,
    mut vignette_query: Query<(Entity, &mut Sprite), With<LowHealthVignette>>,
) {
    let Ok(stats) = player_query.get_single() else {
        // Remove vignette if no player
        for (entity, _) in vignette_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    };

    // Calculate health percentage (all layers combined)
    let total_max = stats.max_shield + stats.max_armor + stats.max_hull;
    let total_current = stats.shield + stats.armor + stats.hull;
    let health_pct = total_current / total_max;

    // Show vignette below 30% health, intensity increases as health drops
    const VIGNETTE_THRESHOLD: f32 = 0.30;

    if health_pct < VIGNETTE_THRESHOLD {
        let elapsed = time.elapsed_secs();

        // Urgency increases as health drops (0 = threshold, 1 = near death)
        let urgency = 1.0 - (health_pct / VIGNETTE_THRESHOLD);

        // Pulse speed increases with urgency (2-6 Hz)
        let pulse_speed = 2.0 + urgency * 4.0;
        let pulse = (elapsed * pulse_speed * std::f32::consts::TAU).sin() * 0.5 + 0.5;

        // Alpha based on urgency and pulse
        let base_alpha = 0.1 + urgency * 0.25;
        let alpha = base_alpha + pulse * urgency * 0.15;

        if let Ok((_, mut sprite)) = vignette_query.get_single_mut() {
            sprite.color = Color::srgba(0.8, 0.0, 0.0, alpha);
        } else {
            // Spawn vignette overlay
            commands.spawn((
                LowHealthVignette,
                Sprite {
                    color: Color::srgba(0.8, 0.0, 0.0, alpha),
                    custom_size: Some(Vec2::new(SCREEN_WIDTH + 100.0, SCREEN_HEIGHT + 100.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, LAYER_HUD + 4.0), // Below salt miner tint
            ));
        }
    } else {
        // Remove vignette when health is OK
        for (entity, _) in vignette_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// DISINTEGRATOR BEAM VISUALS
// =============================================================================

/// Visual component for the Triglavian disintegrator beam line
#[derive(Component)]
pub struct DisintegratorBeamVisual {
    /// The enemy entity this beam belongs to
    pub source_entity: Entity,
}

/// Renders/updates disintegrator beam lines from Triglavian enemies to the player.
/// The beam appears as a stretched sprite between the enemy and player, scaling
/// in thickness and color intensity based on `DisintegratorRamp.beam_intensity`.
pub fn update_disintegrator_beams(
    mut commands: Commands,
    player_query: Query<&Transform, With<crate::entities::Player>>,
    enemy_query: Query<
        (Entity, &Transform, &crate::entities::DisintegratorRamp),
        With<crate::entities::Enemy>,
    >,
    mut beam_query: Query<
        (
            Entity,
            &DisintegratorBeamVisual,
            &mut Transform,
            &mut Sprite,
        ),
        (
            Without<crate::entities::Enemy>,
            Without<crate::entities::Player>,
        ),
    >,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    // Track which enemies still have active beams
    let mut active_sources: Vec<Entity> = Vec::new();

    for (enemy_entity, enemy_transform, ramp) in enemy_query.iter() {
        if !ramp.beam_active {
            continue;
        }

        active_sources.push(enemy_entity);
        let enemy_pos = enemy_transform.translation.truncate();
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();

        if distance < 1.0 {
            continue;
        }

        let midpoint = (enemy_pos + player_pos) / 2.0;
        let angle = to_player.y.atan2(to_player.x) - std::f32::consts::FRAC_PI_2;

        // Beam thickness scales with intensity (1-6 pixels)
        let thickness = 1.0 + ramp.beam_intensity * 5.0;
        // Color shifts from orange-red to bright red-white as ramp increases
        let r = 0.9 + ramp.beam_intensity * 0.1;
        let g = 0.2 + ramp.beam_intensity * 0.3;
        let b = 0.0 + ramp.beam_intensity * 0.2;
        let a = 0.5 + ramp.beam_intensity * 0.5;
        let beam_color = Color::srgba(r, g, b, a);

        // Try to find existing beam visual for this enemy
        let mut found = false;
        for (_beam_entity, beam_visual, mut transform, mut sprite) in beam_query.iter_mut() {
            if beam_visual.source_entity == enemy_entity {
                // Update existing beam
                transform.translation = Vec3::new(midpoint.x, midpoint.y, LAYER_EFFECTS - 1.0);
                transform.rotation = Quat::from_rotation_z(angle);
                sprite.custom_size = Some(Vec2::new(thickness, distance));
                sprite.color = beam_color;
                found = true;
                break;
            }
        }

        if !found {
            // Spawn new beam visual
            commands.spawn((
                DisintegratorBeamVisual {
                    source_entity: enemy_entity,
                },
                Sprite {
                    color: beam_color,
                    custom_size: Some(Vec2::new(thickness, distance)),
                    ..default()
                },
                Transform::from_xyz(midpoint.x, midpoint.y, LAYER_EFFECTS - 1.0)
                    .with_rotation(Quat::from_rotation_z(angle)),
            ));
        }
    }

    // Despawn beam visuals for enemies that stopped firing
    for (beam_entity, beam_visual, _, _) in beam_query.iter() {
        if !active_sources.contains(&beam_visual.source_entity) {
            commands.entity(beam_entity).despawn_recursive();
        }
    }
}
