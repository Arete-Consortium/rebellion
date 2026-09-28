//! Enemy Spatial Awareness & Predictive AI
//!
//! Projectile dodge, enemy separation, edge avoidance, and coordinated leader-escort tactics.

use super::types::*;
use crate::core::*;
use crate::entities::projectile::{PlayerProjectile, ProjectilePhysics};
use bevy::prelude::*;

// =============================================================================
// SPATIAL AWARENESS & PREDICTIVE AI
// =============================================================================

/// How far enemies detect incoming player projectiles
pub(super) const DODGE_DETECTION_RADIUS: f32 = 100.0;
/// Base dodge force applied when evading projectiles
pub(super) const DODGE_STRENGTH: f32 = 150.0;
/// Distance at which enemies repel from each other
pub(super) const SEPARATION_RADIUS: f32 = 55.0;
/// Force applied to keep enemies apart
pub(super) const SEPARATION_STRENGTH: f32 = 80.0;
/// Screen edge margin for soft boundary avoidance
pub(super) const EDGE_AVOIDANCE_MARGIN: f32 = 40.0;
/// Force pushing enemies away from screen edges
pub(super) const EDGE_PUSH_STRENGTH: f32 = 120.0;
/// Maximum total dodge impulse magnitude (prevents runaway forces)
pub(super) const MAX_DODGE_IMPULSE: f32 = 300.0;
/// Radius within which enemies rally around leader units (Spawner/Tank)
pub(super) const LEADER_RALLY_RADIUS: f32 = 150.0;
/// Cohesion force pulling escort enemies toward their leader
pub(super) const LEADER_COHESION_STRENGTH: f32 = 40.0;

/// Tracks player position and velocity so enemies can predict movement
#[derive(Resource, Default)]
pub struct PlayerTracker {
    /// Current player position
    pub position: Vec2,
    /// Estimated player velocity (units/sec)
    pub velocity: Vec2,
    /// Previous frame position for velocity calculation
    prev_position: Vec2,
    /// Whether we've seen at least one frame
    initialized: bool,
}

/// Updates the player tracker with current position and derived velocity
pub(super) fn update_player_tracker(
    time: Res<Time>,
    player_query: Query<&Transform, With<crate::entities::Player>>,
    mut tracker: ResMut<PlayerTracker>,
) {
    let dt = time.delta_secs();
    if let Ok(transform) = player_query.get_single() {
        let pos = transform.translation.truncate();
        if tracker.initialized && dt > 0.0 {
            tracker.velocity = (pos - tracker.prev_position) / dt;
        }
        tracker.prev_position = pos;
        tracker.position = pos;
        tracker.initialized = true;
    }
}

/// Computes spatial awareness for each enemy: projectile dodge, enemy separation,
/// edge avoidance, and coordinated leader-escort tactics.
/// Stores result in `EnemyAI.dodge_impulse` for the movement system to apply.
pub(super) fn enemy_spatial_awareness(
    projectile_query: Query<(&Transform, &ProjectilePhysics), With<PlayerProjectile>>,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyAI), With<Enemy>>,
) {
    // Collect enemy positions and behaviors first (immutable pass)
    let enemy_data: Vec<(Entity, Vec2, EnemyBehavior)> = enemy_query
        .iter()
        .map(|(e, t, ai)| (e, t.translation.truncate(), ai.behavior))
        .collect();

    // Sorting makes escort slots stable across ECS iteration-order changes.
    let mut enemy_data = enemy_data;
    enemy_data.sort_by_key(|(entity, _, _)| entity.to_bits());
    // Identify leader positions (Spawner and Tank enemies act as squad leaders)
    let leaders: Vec<Vec2> = enemy_data
        .iter()
        .filter(|(_, _, b)| matches!(b, EnemyBehavior::Spawner | EnemyBehavior::Tank))
        .map(|(_, pos, _)| *pos)
        .collect();

    // Collect projectile data
    let projectiles: Vec<(Vec2, Vec2)> = projectile_query
        .iter()
        .map(|(t, p)| (t.translation.truncate(), p.velocity))
        .collect();

    let half_w = SCREEN_WIDTH / 2.0;
    let half_h = SCREEN_HEIGHT / 2.0;

    for (entity, transform, mut ai) in enemy_query.iter_mut() {
        let pos = transform.translation.truncate();
        let sensitivity = ai.behavior.dodge_sensitivity();
        let mut impulse = Vec2::ZERO;

        if sensitivity > 0.0 {
            // 1. Projectile dodge — evade incoming player bullets
            for &(proj_pos, proj_vel) in &projectiles {
                let to_enemy = pos - proj_pos;
                let dist = to_enemy.length();

                if dist < DODGE_DETECTION_RADIUS && dist > 1.0 {
                    let proj_dir = proj_vel.normalize_or_zero();
                    let approach = proj_dir.dot(to_enemy.normalize_or_zero());

                    if approach > 0.2 {
                        let perpendicular = Vec2::new(-proj_dir.y, proj_dir.x);
                        let side = perpendicular.dot(to_enemy).signum();
                        let urgency = 1.0 - (dist / DODGE_DETECTION_RADIUS);
                        impulse += perpendicular * side * urgency * DODGE_STRENGTH * sensitivity;
                    }
                }
            }

            // 2. Separation — avoid stacking on top of other enemies
            for &(other_entity, other_pos, _) in &enemy_data {
                if other_entity == entity {
                    continue;
                }
                let diff = pos - other_pos;
                let dist = diff.length();
                if dist < SEPARATION_RADIUS && dist > 1.0 {
                    let push = diff.normalize_or_zero()
                        * (1.0 - dist / SEPARATION_RADIUS)
                        * SEPARATION_STRENGTH;
                    impulse += push;
                }
            }

            // 3. Screen edge avoidance
            if pos.x < -half_w + EDGE_AVOIDANCE_MARGIN {
                impulse.x += (1.0 - (pos.x + half_w) / EDGE_AVOIDANCE_MARGIN) * EDGE_PUSH_STRENGTH;
            }
            if pos.x > half_w - EDGE_AVOIDANCE_MARGIN {
                impulse.x -= (1.0 - (half_w - pos.x) / EDGE_AVOIDANCE_MARGIN) * EDGE_PUSH_STRENGTH;
            }
            if pos.y > half_h - EDGE_AVOIDANCE_MARGIN {
                impulse.y -= (1.0 - (half_h - pos.y) / EDGE_AVOIDANCE_MARGIN) * EDGE_PUSH_STRENGTH;
            }
        }

        // 4. Coordinated tactics — escort enemies rally near leaders
        // Non-leader enemies are gently pulled toward the nearest Spawner or Tank
        let is_leader = matches!(ai.behavior, EnemyBehavior::Spawner | EnemyBehavior::Tank);
        if !is_leader && ai.behavior != EnemyBehavior::Kamikaze && !leaders.is_empty() {
            let mut nearest_leader: Option<Vec2> = None;
            let mut nearest_dist = LEADER_RALLY_RADIUS;
            for &leader_pos in &leaders {
                let dist = (leader_pos - pos).length();
                if dist < nearest_dist {
                    nearest_dist = dist;
                    nearest_leader = Some(leader_pos);
                }
            }
            if let Some(leader_pos) = nearest_leader {
                let slot = enemy_data.iter()
                    .filter(|(_, _, behavior)| !matches!(behavior,
                        EnemyBehavior::Spawner | EnemyBehavior::Tank | EnemyBehavior::Kamikaze))
                    .position(|(other, _, _)| *other == entity).unwrap_or(0);
                let to_station = leader_pos + escort_offset(slot) - pos;
                // Alternating wings create crossfire lanes around the leader.
                impulse += (to_station * 0.8).clamp_length_max(LEADER_COHESION_STRENGTH);

            }
        }

        ai.dodge_impulse = impulse.clamp_length_max(MAX_DODGE_IMPULSE);
    }
}

/// Alternating wings, with further rows trailing their leader.
fn escort_offset(slot: usize) -> Vec2 {
    let side = if slot % 2 == 0 { -1.0 } else { 1.0 };
    let row = (slot / 2) as f32;
    Vec2::new(side * (65.0 + row * 18.0), -25.0 + row * 28.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escorts_take_separate_wings_and_rows() {
        assert!(escort_offset(0).x < 0.0 && escort_offset(1).x > 0.0);
        for a in 0..8 { for b in a+1..8 {
            assert!(escort_offset(a).distance(escort_offset(b)) > 25.0);
        }}
    }
}
