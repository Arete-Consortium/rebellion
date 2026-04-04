//! Boss health bar display system

use super::common::*;
use crate::entities::{Boss, BossData, BossState};
use bevy::prelude::*;

/// Update boss health bar
pub fn update_boss_health_bar(
    boss_query: Query<(&BossData, &BossState), With<Boss>>,
    mut container_query: Query<&mut Node, With<BossHealthContainer>>,
    mut fill_query: Query<&mut Node, (With<BossHealthFill>, Without<BossHealthContainer>)>,
    mut name_query: Query<&mut Text, With<BossNameText>>,
) {
    let has_boss = boss_query.get_single().is_ok();

    // Show/hide boss health bar
    for mut node in container_query.iter_mut() {
        node.display = if has_boss {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Ok((data, state)) = boss_query.get_single() {
        // Update health bar fill
        for mut node in fill_query.iter_mut() {
            let health_percent = (data.health / data.max_health * 100.0).max(0.0);
            node.width = Val::Percent(health_percent);
        }

        // Update boss name
        for mut text in name_query.iter_mut() {
            let phase_info = if data.total_phases > 1 {
                format!(" (Phase {}/{})", data.current_phase, data.total_phases)
            } else {
                String::new()
            };

            match *state {
                BossState::Intro => {
                    **text = format!("{} - {}", data.name, data.title);
                }
                BossState::Battle | BossState::PhaseTransition => {
                    **text = format!("{}{}", data.name, phase_info);
                }
                BossState::Defeated => {
                    **text = format!("{} DEFEATED!", data.name);
                }
            }
        }
    }
}
