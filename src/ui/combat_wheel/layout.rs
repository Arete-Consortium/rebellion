use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashMap;

use super::components::*;
use super::resources::HudSettings;

/// Computes screen-space layout for the Combat Wheel.
///
/// Fix vs. prototype: the prototype placed Ability and Deployable at the same
/// `(center.x, center.y + 60.0 * scale)` coords in compact mode (visible
/// stacking bug). For the Triglavian Invasion the player has no Deployable
/// capability (see `src/games/triglavian_invasion/ships.rs`), so we drop
/// Deployable from the slot set entirely. The 5 remaining slots
/// (Primary, Secondary, Propulsion, Defense, Ability) get unique angles in
/// both layouts.
pub fn resolve_layout(
    window_query: Query<&Window, With<PrimaryWindow>>,
    settings: Res<HudSettings>,
    mut wheel_query: Query<&mut Transform, With<CombatWheel>>,
    mut module_query: Query<(&ModuleSlot, &mut Transform), Without<CombatWheel>>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let screen = Vec2::new(window.width(), window.height());

    let compact = screen.x < settings.compact_mode_threshold.x
        || screen.y < settings.compact_mode_threshold.y;

    for mut transform in wheel_query.iter_mut() {
        let wheel_pos = Vec3::new(screen.x * 0.5, screen.y * 0.18, 100.0);
        transform.translation = wheel_pos;
        transform.scale = Vec3::splat(settings.scale * if compact { 0.6 } else { 1.0 });
    }

    let positions = if compact {
        compact_module_positions(screen, settings.scale)
    } else {
        standard_module_positions(screen, settings.scale)
    };

    for (slot, mut transform) in module_query.iter_mut() {
        if let Some(pos) = positions.get(&slot.slot_id) {
            transform.translation = *pos;
        }
    }
}

/// Standard 6-slot (or 5 with Deployable dropped) layout for desktop /
/// large screens. Six evenly-spaced positions starting at the top
/// (`-PI/2`) and walking clockwise.
fn standard_module_positions(screen: Vec2, scale: f32) -> HashMap<ModuleSlotId, Vec3> {
    let center = Vec2::new(screen.x * 0.5, screen.y * 0.18);
    let radius = 140.0 * scale;
    let mut map = HashMap::new();

    // Six cardinal/intercardinal positions starting from top, clockwise.
    // -PI/2, -PI/6, PI/6, PI/2, 5*PI/6, 7*PI/6 (60° apart, primary at top)
    let angles = [
        (-std::f32::consts::FRAC_PI_2, ModuleSlotId::PrimaryWeapon),
        (-std::f32::consts::FRAC_PI_6, ModuleSlotId::SecondaryWeapon),
        (std::f32::consts::FRAC_PI_6, ModuleSlotId::Propulsion),
        (std::f32::consts::FRAC_PI_2, ModuleSlotId::Defense),
        (5.0 * std::f32::consts::FRAC_PI_6, ModuleSlotId::Ability),
    ];

    for (angle, slot_id) in angles {
        let pos = center + Vec2::from_angle(angle) * radius;
        map.insert(slot_id, Vec3::new(pos.x, pos.y, 101.0));
    }

    map
}

/// Compact 5-slot layout for small screens. Tighter ring (110px radius) and
/// fewer slots — Deployable is dropped for the Triglavian campaign because
/// player ships have no deployable capability. Each of the 5 slots gets a
/// unique angle (60° apart starting at top), no stacking.
fn compact_module_positions(screen: Vec2, scale: f32) -> HashMap<ModuleSlotId, Vec3> {
    let center = Vec2::new(screen.x * 0.5, screen.y * 0.15);
    let radius = 110.0 * scale;
    let mut map = HashMap::new();

    // 5 positions, 72° apart, starting from top.
    // -PI/2, -PI/2 + 2*PI/5, -PI/2 + 4*PI/5, ...
    let start = -std::f32::consts::FRAC_PI_2;
    let step = 2.0 * std::f32::consts::PI / 5.0;
    let slot_ids = [
        ModuleSlotId::PrimaryWeapon,
        ModuleSlotId::SecondaryWeapon,
        ModuleSlotId::Propulsion,
        ModuleSlotId::Defense,
        ModuleSlotId::Ability,
    ];

    for (i, slot_id) in slot_ids.iter().enumerate() {
        let angle = start + step * i as f32;
        let pos = center + Vec2::from_angle(angle) * radius;
        map.insert(*slot_id, Vec3::new(pos.x, pos.y, 101.0));
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_positions_has_all_five_slots() {
        let pos = standard_module_positions(Vec2::new(1920.0, 1080.0), 1.0);
        // Deployable is intentionally dropped for the Triglavian campaign.
        assert!(pos.contains_key(&ModuleSlotId::PrimaryWeapon));
        assert!(pos.contains_key(&ModuleSlotId::SecondaryWeapon));
        assert!(pos.contains_key(&ModuleSlotId::Propulsion));
        assert!(pos.contains_key(&ModuleSlotId::Defense));
        assert!(pos.contains_key(&ModuleSlotId::Ability));
        assert!(!pos.contains_key(&ModuleSlotId::Deployable));
    }

    #[test]
    fn compact_positions_has_all_five_slots() {
        let pos = compact_module_positions(Vec2::new(1280.0, 720.0), 0.8);
        assert!(pos.contains_key(&ModuleSlotId::PrimaryWeapon));
        assert!(pos.contains_key(&ModuleSlotId::SecondaryWeapon));
        assert!(pos.contains_key(&ModuleSlotId::Propulsion));
        assert!(pos.contains_key(&ModuleSlotId::Defense));
        assert!(pos.contains_key(&ModuleSlotId::Ability));
        assert!(!pos.contains_key(&ModuleSlotId::Deployable));
    }

    #[test]
    fn compact_layout_has_unique_slot_positions() {
        // Regression test for the prototype's stacking bug
        // (layout.rs:90-97 in the prototype: Ability and Deployable at the same
        // coords). Even though Deployable is now dropped, we verify the 5
        // remaining slots are all distinct.
        let pos = compact_module_positions(Vec2::new(1280.0, 720.0), 1.0);
        let pts: Vec<Vec3> = pos.values().copied().collect();
        for (i, a) in pts.iter().enumerate() {
            for (j, b) in pts.iter().enumerate().skip(i + 1) {
                let dist = (a - b).length();
                assert!(
                    dist > 5.0,
                    "slots {i} and {j} are within 5px of each other ({dist:.2}px): {a:?} vs {b:?}"
                );
            }
        }
    }

    #[test]
    fn standard_layout_has_unique_slot_positions() {
        let pos = standard_module_positions(Vec2::new(1920.0, 1080.0), 1.0);
        let pts: Vec<Vec3> = pos.values().copied().collect();
        for (i, a) in pts.iter().enumerate() {
            for (j, b) in pts.iter().enumerate().skip(i + 1) {
                let dist = (a - b).length();
                assert!(
                    dist > 5.0,
                    "slots {i} and {j} are within 5px of each other ({dist:.2}px): {a:?} vs {b:?}"
                );
            }
        }
    }

    #[test]
    fn compact_positions_scale_down() {
        let pos1 = compact_module_positions(Vec2::new(1280.0, 720.0), 1.0);
        let pos2 = compact_module_positions(Vec2::new(1280.0, 720.0), 0.5);
        // SecondaryWeapon is at angle -PI/2 + 2*PI/5 (~-18°), so it has a
        // horizontal component. Halving scale halves the offset from center,
        // so x-distance from center must shrink.
        let center = 1280.0 * 0.5;
        let p1 = pos1[&ModuleSlotId::SecondaryWeapon];
        let p2 = pos2[&ModuleSlotId::SecondaryWeapon];
        let d1 = (p1.x - center).abs();
        let d2 = (p2.x - center).abs();
        assert!(
            d2 < d1,
            "scale=0.5 should bring slot closer to center x: d1={d1:.2} d2={d2:.2}"
        );

        // Total distance from center (in xy) should also be smaller.
        let center2 = Vec2::new(center, 720.0 * 0.15);
        let r1 = Vec2::new(p1.x, p1.y).distance(center2);
        let r2 = Vec2::new(p2.x, p2.y).distance(center2);
        assert!(
            r2 < r1,
            "scale=0.5 should reduce ring radius: r1={r1:.2} r2={r2:.2}"
        );
    }
}
