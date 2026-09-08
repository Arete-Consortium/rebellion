//! Read-only cues for the Hedion objective: boarding radius, progress, hull
//! integrity, and a short targeting warning from designated convoy raiders.

use crate::entities::{escort::EscortAttacker, EnemyAI, EnemyWeapon, EscortData};
use crate::games::elder_fleet::transport::{
    MissionTransport, TransportObjective, TransportPhase, TRANSFER_RADIUS,
};
use bevy::prelude::*;

pub fn draw_transport_objective(
    mut gizmos: Gizmos,
    objective: Option<Res<TransportObjective>>,
    transports: Query<(&Transform, &EscortData), With<MissionTransport>>,
    attackers: Query<(&Transform, &EnemyWeapon, &EnemyAI, &EscortAttacker)>,
) {
    let Some(objective) = objective else {
        return;
    };
    if objective.phase != TransportPhase::Active {
        return;
    }
    let Ok((transform, hull)) = transports.get_single() else {
        return;
    };
    let pos = transform.translation.truncate();
    let cyan = Color::srgb(0.45, 0.9, 1.0);
    let amber = Color::srgb(1.0, 0.6, 0.2);
    gizmos.circle_2d(
        pos,
        TRANSFER_RADIUS,
        if objective.in_range {
            cyan
        } else {
            cyan.with_alpha(0.35)
        },
    );
    // Segment geometry stays bounded and conveys progress without color alone.
    let segments = 48;
    for i in 0..segments {
        if (i as f32) / (segments as f32) >= objective.progress() {
            break;
        }
        let a = std::f32::consts::FRAC_PI_2 - (i as f32 / segments as f32) * std::f32::consts::TAU;
        let b = a - std::f32::consts::TAU / segments as f32;
        gizmos.line_2d(
            pos + Vec2::from_angle(a) * (TRANSFER_RADIUS + 5.0),
            pos + Vec2::from_angle(b) * (TRANSFER_RADIUS + 5.0),
            cyan,
        );
    }
    let bar_start = pos + Vec2::new(-40.0, 84.0);
    gizmos.line_2d(
        bar_start,
        bar_start + Vec2::new(80.0, 0.0),
        Color::srgb(0.25, 0.3, 0.35),
    );
    gizmos.line_2d(
        bar_start,
        bar_start + Vec2::new(80.0 * hull.health_fraction(), 0.0),
        if hull.health_fraction() < 0.3 {
            amber
        } else {
            cyan
        },
    );
    for (transform, weapon, ai, target) in &attackers {
        if !ai.active || weapon.cooldown > 0.65 || transports.get(target.0).is_err() {
            continue;
        }
        let source = transform.translation.truncate();
        gizmos.circle_2d(source, 26.0, amber);
        for i in 0..8 {
            gizmos.line_2d(
                source.lerp(pos, i as f32 / 8.0),
                source.lerp(pos, (i as f32 + 0.5) / 8.0),
                amber.with_alpha(0.7),
            );
        }
    }
}
