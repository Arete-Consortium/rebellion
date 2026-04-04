//! Scoring System
//!
//! Handles score, multipliers, chain combos, and salt miner meter.

use crate::core::*;
use bevy::prelude::*;

/// Scoring plugin
pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_score_system, update_salt_miner_system).run_if(in_state(GameState::Playing)),
        );
    }
}

/// Update score chain timer
fn update_score_system(time: Res<Time>, mut score: ResMut<ScoreSystem>) {
    score.update(time.delta_secs());
}

/// Update salt miner meter and handle activation input
fn update_salt_miner_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<crate::systems::JoystickState>,
    mut salt_miner: ResMut<SaltMinerSystem>,
    mut end_events: EventWriter<SaltMinerEndedEvent>,
    mut screen_flash: ResMut<crate::systems::ScreenFlash>,
    mut screen_shake: ResMut<crate::systems::ScreenShake>,
    mut dialogue_events: EventWriter<super::DialogueEvent>,
    mut rumble_events: EventWriter<super::RumbleRequest>,
) {
    let was_active = salt_miner.is_active;
    salt_miner.update(time.delta_secs());

    // Check if salt miner just ended
    if was_active && !salt_miner.is_active {
        end_events.send(SaltMinerEndedEvent);
        info!("Salt Miner mode ended!");
    }

    // B key or gamepad Y button to activate salt miner when meter is full
    let activate_pressed = keyboard.just_pressed(KeyCode::KeyB) || joystick.salt_miner();

    if activate_pressed && salt_miner.can_activate() && salt_miner.try_activate() {
        info!("SALT MINER MODE ACTIVATED! 5x score for 8 seconds!");
        screen_shake.medium(); // Screen shake on activation
        screen_flash.colored(Color::srgba(1.0, 0.85, 0.2, 0.7), 0.6); // Gold flash on activation
        screen_flash.fade_speed = 3.0;
        rumble_events.send(super::RumbleRequest::salt_miner()); // Controller rumble
        dialogue_events.send(super::DialogueEvent::combat_callout(
            super::CombatCalloutType::SaltMinerActive,
        ));
    }
}

// Salt miner meter fills from proximity kills
// See collision.rs: player_projectile_enemy_collision
