//! Combat Presentation Reactions
//!
//! Visual and audio feedback for combat events.
//! Consumes simulation/gameplay outcome events but never mutates authoritative state.

use crate::core::{
    ChainBoltSpawnEvent, ContactDetected, EnemyDamageAppliedEvent, EnemyDestroyedEvent,
    ExplosionEvent, ExplosionSize, PlayerDamagedEvent,
};
use crate::entities::{Enemy, EnemyStats, Player, ShipStats};
use crate::systems::effects::{
    spawn_damage_number, spawn_impact_sparks, CameraZoom, HitFlash, HitStop, ScreenFlash,
    ScreenShake,
};
use crate::systems::{
    dialogue::{CombatCalloutType, DialogueEvent},
    joystick::RumbleRequest,
};
use bevy::prelude::*;

/// Marker to prevent duplicate boss low-health callouts per boss entity.
#[derive(Component)]
pub struct BossCalloutSent;

// =============================================================================
// Enemy Hit Reactions
// =============================================================================

/// Spawn impact sparks, damage numbers, hit flash, and screen effects when
/// an enemy is damaged.
pub fn enemy_hit_reactions(
    mut commands: Commands,
    mut damage_events: EventReader<EnemyDamageAppliedEvent>,
    enemy_query: Query<(Entity, Option<&Sprite>), With<Enemy>>,
    mut screen_shake: ResMut<ScreenShake>,
    mut screen_flash: ResMut<ScreenFlash>,
) {
    for event in damage_events.read() {
        let Ok((enemy_entity, sprite)) = enemy_query.get(event.enemy) else {
            continue;
        };

        // Hit flash effect (white flash when damaged)
        let original_color = sprite.map(|s| s.color).unwrap_or(Color::WHITE);
        commands
            .entity(enemy_entity)
            .insert(HitFlash::new(original_color));

        // Impact sparks — radial burst in damage-type color.
        spawn_impact_sparks(&mut commands, event.enemy_pos, event.damage_type);

        // Crit punch — subtle screen flash + extra shake on crit hits.
        if event.is_crit {
            screen_flash.brief();
            screen_shake.trigger(4.0, 0.05);
        }

        // Spawn floating damage number
        spawn_damage_number(&mut commands, event.enemy_pos, event.damage, event.is_crit);
    }
}

/// Emit boss low-health dialogue callouts once per boss entity.
pub fn boss_health_callouts(
    mut commands: Commands,
    boss_query: Query<(Entity, &EnemyStats), (With<Enemy>, Without<BossCalloutSent>)>,
    mut dialogue_events: EventWriter<DialogueEvent>,
) {
    for (entity, stats) in boss_query.iter() {
        if !stats.is_boss {
            continue;
        }
        let health_pct = stats.health / stats.max_health;
        if health_pct > 0.0 && health_pct < 0.25 {
            dialogue_events.send(DialogueEvent::combat_callout(
                CombatCalloutType::BossLowHealth,
            ));
            commands.entity(entity).insert(BossCalloutSent);
        }
    }
}

// =============================================================================
// Enemy Death Reactions
// =============================================================================

/// Spawn explosions, screen shake/zoom/hitstop on enemy destruction.
pub fn enemy_death_reactions(
    mut destroy_events: EventReader<EnemyDestroyedEvent>,
    mut screen_shake: ResMut<ScreenShake>,
    mut screen_flash: ResMut<ScreenFlash>,
    mut camera_zoom: ResMut<CameraZoom>,
    mut hit_stop: ResMut<HitStop>,
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    for event in destroy_events.read() {
        // Faction-colored explosions based on enemy type
        let explosion_color = match event.type_id {
            // Amarr enemies — golden
            597 | 589 | 591 | 16236 | 624 | 625 | 24690 => Color::srgb(1.0, 0.85, 0.3),
            // Triglavian enemies — red
            47269..=47273 => Color::srgb(0.9, 0.2, 0.3),
            // Default — orange
            _ => Color::srgb(1.0, 0.5, 0.2),
        };

        explosion_events.send(ExplosionEvent {
            position: event.position,
            size: if event.was_boss {
                ExplosionSize::Massive
            } else {
                ExplosionSize::Small
            },
            color: explosion_color,
        });

        // Screen shake, flash, zoom, and hitstop on kill
        if event.was_boss {
            screen_shake.massive();
            screen_flash.massive();
            camera_zoom.boss_kill();
            hit_stop.trigger(0.08);
        } else {
            screen_shake.trigger(3.0, 0.1);
            hit_stop.trigger(0.02);
        }
    }
}

// =============================================================================
// Player Hit Reactions
// =============================================================================

/// Apply hit flash, rumble, screen shake, and health callouts when the player
/// is damaged by an enemy projectile.
pub fn player_hit_reactions(
    mut commands: Commands,
    mut contact_events: EventReader<ContactDetected>,
    mut player_damaged: EventReader<PlayerDamagedEvent>,
    player_query: Query<(Entity, &Transform, &ShipStats, Option<&Sprite>), With<Player>>,
    mut dialogue_events: EventWriter<DialogueEvent>,
    mut rumble_events: EventWriter<RumbleRequest>,
    mut screen_shake: ResMut<ScreenShake>,
    mut last_callout: Local<f32>,
    time: Res<Time>,
) {
    // Advance callout cooldown timer
    *last_callout += time.delta_secs();

    let Ok((player_entity, player_transform, player_stats, sprite)) = player_query.get_single()
    else {
        return;
    };
    // Process each player-damaged event (one per projectile that hit)
    for _event in player_damaged.read() {
        // Add hit flash effect to player (red-white flash when hit)
        let original_color = sprite.map(|s| s.color).unwrap_or(Color::WHITE);
        commands
            .entity(player_entity)
            .insert(HitFlash::with_duration(original_color, 0.15));

        // Controller rumble on hit
        rumble_events.send(RumbleRequest::player_hit());

        // Screen shake on hit
        screen_shake.small();

        // Health callouts (with 8 second cooldown)
        if *last_callout > 8.0 {
            let total_hp = player_stats.shield + player_stats.armor + player_stats.hull;
            let max_hp = player_stats.max_shield + player_stats.max_armor + player_stats.max_hull;
            let health_pct = total_hp / max_hp;
            if health_pct < 0.2 {
                dialogue_events.send(DialogueEvent::combat_callout(CombatCalloutType::NearDeath));
                *last_callout = 0.0;
            } else if health_pct < 0.4 {
                dialogue_events.send(DialogueEvent::combat_callout(CombatCalloutType::LowHealth));
                *last_callout = 0.0;
            }
        }
    }

    // We also read ContactDetected to get projectile positions for any future
    // directional effects, but currently the main reactions are driven by
    // PlayerDamagedEvent above. Mark as read so events don't accumulate.
    for _contact in contact_events.read() {
        // No-op: ContactDetected reactions are handled by damage-layer systems
        // and the PlayerDamagedEvent loop above.
    }
}

// =============================================================================
// Player Death Reactions
// =============================================================================

/// Trigger dramatic death effects when the player is destroyed.
pub fn player_death_reactions(
    mut player_damaged: EventReader<PlayerDamagedEvent>,
    player_query: Query<&Transform, With<Player>>,
    mut screen_shake: ResMut<ScreenShake>,
    mut screen_flash: ResMut<ScreenFlash>,
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for event in player_damaged.read() {
        if event.destroyed {
            screen_shake.massive();
            screen_flash.colored(Color::srgb(1.0, 0.2, 0.2), 0.9);
            explosion_events.send(ExplosionEvent {
                position: player_pos,
                size: ExplosionSize::Massive,
                color: Color::srgb(1.0, 0.4, 0.2),
            });
        }
    }
}

// =============================================================================
// Chain Lightning
// =============================================================================

/// Spawn chain-lightning bolt sprites from simulation events.
pub fn spawn_chain_bolts(mut commands: Commands, mut events: EventReader<ChainBoltSpawnEvent>) {
    for event in events.read() {
        spawn_chain_bolt(&mut commands, event.from, event.to);
    }
}

/// Fade and despawn chain-lightning bolts.
pub fn tick_chain_bolts(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(
        Entity,
        &mut crate::entities::projectile::ChainBolt,
        &mut Sprite,
    )>,
) {
    let dt = time.delta_secs();
    for (e, mut b, mut s) in q.iter_mut() {
        b.life -= dt;
        if b.life <= 0.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        s.color.set_alpha(b.life / b.max);
    }
}

// ---------------------------------------------------------------------------
// Helper: spawn a jagged chain-lightning arc between two points.
// ---------------------------------------------------------------------------
fn spawn_chain_bolt(commands: &mut Commands, from: Vec2, to: Vec2) {
    use crate::entities::projectile::ChainBolt;
    const SEGMENTS: usize = 5;
    let diff = to - from;
    let total_len = diff.length().max(1.0);
    let dir = diff / total_len;
    let perp = Vec2::new(-dir.y, dir.x);
    let jitter_amp = (total_len * 0.09).min(28.0);

    let core_color = Color::srgba(1.0, 1.0, 1.0, 1.0);
    let halo_color = Color::srgba(0.55, 0.85, 1.0, 0.85);

    let mut points = Vec::with_capacity(SEGMENTS + 1);
    points.push(from);
    for i in 1..SEGMENTS {
        let t = i as f32 / SEGMENTS as f32;
        let base = from + diff * t;
        let offset = (fastrand::f32() - 0.5) * 2.0 * jitter_amp;
        points.push(base + perp * offset);
    }
    points.push(to);

    for pair in points.windows(2) {
        let a = pair[0];
        let b = pair[1];
        let seg = b - a;
        let seg_len = seg.length().max(1.0);
        let seg_mid = a + seg * 0.5;
        let seg_angle = seg.y.atan2(seg.x);
        // Halo (wider, semi-transparent)
        commands.spawn((
            ChainBolt {
                life: 0.22,
                max: 0.22,
            },
            Sprite {
                color: halo_color,
                custom_size: Some(Vec2::new(seg_len, 7.0)),
                ..default()
            },
            Transform::from_xyz(
                seg_mid.x,
                seg_mid.y,
                crate::core::LAYER_PLAYER_BULLETS + 0.09,
            )
            .with_rotation(Quat::from_rotation_z(seg_angle)),
        ));
        // Bright core
        commands.spawn((
            ChainBolt {
                life: 0.20,
                max: 0.20,
            },
            Sprite {
                color: core_color,
                custom_size: Some(Vec2::new(seg_len, 2.0)),
                ..default()
            },
            Transform::from_xyz(
                seg_mid.x,
                seg_mid.y,
                crate::core::LAYER_PLAYER_BULLETS + 0.12,
            )
            .with_rotation(Quat::from_rotation_z(seg_angle)),
        ));
    }
}
