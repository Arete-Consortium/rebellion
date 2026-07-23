//! Combat Outcome Systems
//!
//! Score mutation, salt miner updates, powerup drops, liberation pods,
//! and game-state transitions. Consumes simulation events but never
//! performs collision detection or spawns visual effects directly.

use crate::core::{
    EnemyDestroyedEvent, GameState, PlayerDamagedEvent, SaltMinerSystem, ScoreSystem,
};
use crate::entities::{
    collectible::{spawn_liberation_pods, spawn_smart_powerup, PlayerHealthState},
    Player, ShipStats,
};
use bevy::prelude::*;

// =============================================================================
// Enemy Death Outcomes
// =============================================================================

/// Update score, salt miner, and spawn drops when enemies are destroyed.
pub fn enemy_death_outcomes(
    mut commands: Commands,
    mut destroy_events: EventReader<EnemyDestroyedEvent>,
    mut score: ResMut<ScoreSystem>,
    mut salt_miner: ResMut<SaltMinerSystem>,
    player_query: Query<(&Transform, &ShipStats), With<Player>>,
    icon_cache: Res<crate::assets::PowerupIconCache>,
    mut sim_rng: ResMut<crate::simulation::SimulationRng>,
) {
    let Ok((player_transform, player_stats)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let player_health = Some(PlayerHealthState::from_stats(player_stats));

    for event in destroy_events.read() {
        // Calculate distance from player to enemy for salt miner
        let player_distance = (player_pos - event.position).length();

        // Update score (with salt miner multiplier)
        let base_score = event.score_value;
        let final_score = (base_score as f32 * salt_miner.score_mult()) as u64;
        score.on_kill(final_score);

        // Fill salt miner meter based on proximity (closer = more meter)
        let meter_gained = salt_miner.on_kill_at_distance(player_distance);
        if meter_gained > 0.0 && salt_miner.can_activate() {
            info!(
                "SALT MINER READY! Press B to activate! (meter: {:.0}%)",
                salt_miner.meter
            );
        }

        // Spawn liberation pods
        spawn_liberation_pods(&mut commands, event.position, event.liberation_value);

        // 30% chance to drop powerup (100% for bosses)
        let drop_chance = if event.was_boss { 1.0 } else { 0.30 };
        if sim_rng.f32() < drop_chance {
            spawn_smart_powerup(
                &mut commands,
                event.position,
                Some(&icon_cache),
                player_health,
            );
        }
    }
}

// =============================================================================
// Player Damage Outcomes
// =============================================================================

/// Mark score and campaign as having lost the no-damage bonus when the player is hit.
pub fn player_damage_outcomes(
    mut player_damaged: EventReader<PlayerDamagedEvent>,
    mut score: ResMut<ScoreSystem>,
    mut campaign: ResMut<crate::core::CampaignState>,
) {
    for _event in player_damaged.read() {
        score.no_damage_bonus = false;
        campaign.no_damage_taken = false;
    }
}

/// Transition to GameOver when the player is destroyed.
pub fn player_death_outcome(
    mut player_damaged: EventReader<PlayerDamagedEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in player_damaged.read() {
        if event.destroyed {
            info!("Player destroyed!");
            next_state.set(GameState::GameOver);
        }
    }
}
