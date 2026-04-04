//! Heat and Salt Miner meter update systems

use super::common::*;
use crate::core::*;
use crate::systems::ComboHeatSystem;
use bevy::prelude::*;

pub fn update_salt_miner_meter(
    salt_miner: Res<SaltMinerSystem>,
    mut query: Query<(&mut Node, &mut BackgroundColor), With<SaltMinerBar>>,
) {
    for (mut node, mut bg) in query.iter_mut() {
        if salt_miner.is_active {
            // Pulsing effect when active - show remaining time
            let pulse = (salt_miner.timer * 10.0).sin().abs();
            node.width = Val::Percent(salt_miner.progress() * 100.0);
            bg.0 = Color::srgb(0.8 + pulse * 0.2, 0.2, 0.8 + pulse * 0.2);
        } else {
            // Show proximity kills progress toward salt miner
            node.width = Val::Percent(salt_miner.progress() * 100.0);
            bg.0 = Color::srgb(0.8, 0.2, 0.8);
        }
    }
}

/// Update heat display bar
pub fn update_heat_display(
    heat_system: Res<ComboHeatSystem>,
    mut query: Query<(&mut Node, &mut BackgroundColor), With<HeatBar>>,
) {
    for (mut node, mut bg) in query.iter_mut() {
        node.width = Val::Percent(heat_system.heat);
        // Color changes with heat level
        bg.0 = heat_system.heat_level.color();
    }
}
