//! Read-only attack tells: no damage, targeting, or collision changes here.
use bevy::prelude::*;
use crate::entities::{Enemy, EnemyAI, EnemyBehavior, EnemyStats, EnemyWeapon, DisintegratorRamp, Player};

pub fn draw_enemy_threats(
    mut gizmos: Gizmos,
    player: Query<&Transform, With<Player>>,
    enemies: Query<(&Transform, &EnemyAI, &EnemyStats, Option<&EnemyWeapon>, Option<&DisintegratorRamp>), With<Enemy>>,
) {
    let Ok(player) = player.get_single() else { return; };
    let target = player.translation.truncate();
    let amber = Color::srgb(1.0, 0.66, 0.28);
    for (transform, ai, stats, weapon, beam) in &enemies {
        if !ai.active || stats.health <= 0.0 { continue; }
        let pos = transform.translation.truncate();
        // Damaged enemies reveal a short integrity bar, not a permanent label.
        if stats.health < stats.max_health {
            let start = pos + Vec2::new(-16.0, 30.0);
            gizmos.line_2d(start, start + Vec2::new(32.0, 0.0), Color::srgb(0.22, 0.25, 0.30));
            gizmos.line_2d(start, start + Vec2::new(32.0 * (stats.health / stats.max_health.max(1.0)).clamp(0.0, 1.0), 0.0), amber);
        }
        if let Some(beam) = beam {
            if beam.time_on_target > 0.0 && !beam.beam_active {
                let progress = (beam.time_on_target / crate::entities::DISINTEGRATOR_LOCK_SECS).clamp(0.0, 1.0);
                gizmos.circle_2d(pos, 32.0 - progress * 10.0, amber);
                for i in 0..8 {
                    gizmos.line_2d(pos.lerp(target, i as f32 / 8.0),
                        pos.lerp(target, (i as f32 + 0.45) / 8.0), amber.with_alpha(0.6));
                }
            }
        }
        if ai.behavior == EnemyBehavior::Kamikaze {
            // Inward chevrons identify a contact threat without rapid flashing.
            let dir = (target - pos).normalize_or_zero();
            let side = Vec2::new(-dir.y, dir.x);
            let tip = pos + dir * 31.0;
            gizmos.line_2d(tip - dir * 9.0 + side * 7.0, tip, amber);
            gizmos.line_2d(tip - dir * 9.0 - side * 7.0, tip, amber);
        } else if let Some(weapon) = weapon {
            if weapon.cooldown > 0.0 && weapon.cooldown < 0.3 && weapon.damage > 0.0 {
                gizmos.circle_2d(pos, 12.0 + weapon.cooldown * 30.0, amber.with_alpha(0.85));
            }
        }
    }
}
