//! Combat-related HUD systems: wingman gauge, drone status, ability indicator, ammo display

use super::common::*;
use crate::core::*;
use crate::entities::{Drone, DroneStats, Player, Wingman, WingmanTracker};
use crate::systems::{Ability, AbilityType};
use bevy::prelude::*;

/// Update wingman gauge (Rifter only)
pub fn update_wingman_gauge(
    tracker: Res<WingmanTracker>,
    selected_ship: Res<SelectedShip>,
    wingmen_query: Query<Entity, With<Wingman>>,
    mut gauge_query: Query<&mut Node, With<WingmanGauge>>,
    mut fill_query: Query<&mut Node, (With<WingmanGaugeFill>, Without<WingmanGauge>)>,
    mut count_query: Query<&mut Text, With<WingmanCountText>>,
) {
    let is_rifter = selected_ship.ship == MinmatarShip::Rifter;

    // Show/hide wingman gauge
    for mut node in gauge_query.iter_mut() {
        node.display = if is_rifter {
            Display::Flex
        } else {
            Display::None
        };
    }

    if !is_rifter {
        return;
    }

    // Update fill bar
    let progress = tracker.progress() * 100.0;
    for mut node in fill_query.iter_mut() {
        node.width = Val::Percent(progress);
    }

    // Update count text
    let wingman_count = wingmen_query.iter().count();
    for mut text in count_query.iter_mut() {
        **text = format!(
            "{}/{} | Active: {}",
            tracker.kill_count, tracker.kills_per_wingman, wingman_count
        );
    }
}

/// Update drone status indicator
pub fn update_drone_status(
    drone_query: Query<&DroneStats, With<Drone>>,
    mut container_query: Query<&mut Node, With<DroneStatusContainer>>,
    mut text_query: Query<&mut Text, With<DroneStatusText>>,
) {
    let drone_count = drone_query.iter().count();
    let has_drones = drone_count > 0;

    // Show/hide container
    for mut node in container_query.iter_mut() {
        node.display = if has_drones {
            Display::Flex
        } else {
            Display::None
        };
    }

    if !has_drones {
        return;
    }

    // Find shortest remaining lifetime
    let min_lifetime = drone_query
        .iter()
        .map(|s| s.lifetime)
        .fold(f32::MAX, f32::min);

    for mut text in text_query.iter_mut() {
        **text = format!("{} active | {:.1}s", drone_count, min_lifetime);
    }
}

/// Update ability indicator display based on player's ability state
pub fn update_ability_indicator(
    player_query: Query<&Ability, With<Player>>,
    mut container_query: Query<&mut Node, With<AbilityIndicatorContainer>>,
    mut fill_query: Query<
        (&mut Node, &mut BackgroundColor),
        (
            With<AbilityIndicatorFill>,
            Without<AbilityIndicatorContainer>,
        ),
    >,
    mut text_query: Query<&mut Text, With<AbilityIndicatorText>>,
) {
    let Ok(ability) = player_query.get_single() else {
        return;
    };

    // Hide if no ability
    for mut node in container_query.iter_mut() {
        node.display = if ability.ability_type == AbilityType::None {
            Display::None
        } else {
            Display::Flex
        };
    }

    if ability.ability_type == AbilityType::None {
        return;
    }

    // Update ability name
    for mut text in text_query.iter_mut() {
        **text = ability.ability_type.name().to_string();
    }

    // Update cooldown bar
    let progress = ability.cooldown_progress();
    for (mut node, mut bg_color) in fill_query.iter_mut() {
        node.width = Val::Percent(progress * 100.0);

        // Color changes: cyan when ready, dark blue when on cooldown, pulsing when active
        if ability.is_active {
            // Pulsing white/cyan when active
            bg_color.0 = Color::srgb(0.8, 0.95, 1.0);
        } else if progress >= 1.0 {
            // Ready - bright cyan
            bg_color.0 = Color::srgb(0.3, 0.9, 1.0);
        } else {
            // Cooldown - darker blue
            bg_color.0 = Color::srgb(0.2, 0.4, 0.6);
        }
    }
}

/// Update ammo type display based on player's current ammo
pub fn update_ammo_display(
    player_query: Query<&crate::entities::Weapon, With<Player>>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<AmmoTypeText>>,
) {
    let Ok(weapon) = player_query.get_single() else {
        return;
    };

    // Only show for autocannons
    if weapon.weapon_type != crate::core::WeaponType::Autocannon {
        return;
    }

    for (mut text, mut color) in text_query.iter_mut() {
        **text = weapon.ammo_type.name().to_string();
        color.0 = weapon.ammo_type.color();
    }
}
