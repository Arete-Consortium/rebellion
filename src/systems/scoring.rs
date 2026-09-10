//! Scoring System
//!
//! Handles score, multipliers, chain combos, salt miner meter, combo/heat.

use crate::core::*;
use bevy::prelude::*;
use bevy::state::state::StateTransitionSteps;

// Combo/heat system lives in scoring_v2.rs; this plugin owns all scoring resources.
use super::scoring_v2::{update_combo_heat_system, ComboHeatSystem};

/// Scoring plugin — owns all score-mutating systems and resources.
pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScoreSystem>()
            .init_resource::<RunResult>()
            .init_resource::<SaltMinerSystem>()
            .init_resource::<ComboHeatSystem>()
            .add_systems(
                StateTransition,
                track_run_transition
                    .after(crate::core::game_state::track_pause_transition)
                    .before(StateTransitionSteps::ExitSchedules),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_score_system,
                    update_salt_miner_system,
                    update_combo_heat_system,
                )
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
            );
    }
}

/// Update score chain timer
fn update_score_system(time: Res<Time>, mut score: ResMut<ScoreSystem>) {
    score.update(time.delta_secs());
    score.run.combat_seconds += time.delta_secs_f64();
}

/// Finalize before exit hooks can clear special-mode identity, and before UI
/// entry hooks read the comparison. Reset only at an attempt boundary. Campaign
/// continuation and boss transitions preserve totals, while an explicit restart
/// begins a fresh attempt on the currently selected mission.
#[allow(clippy::too_many_arguments)]
fn track_run_transition(
    mut transitions: EventReader<StateTransitionEvent<GameState>>,
    mut score: ResMut<ScoreSystem>,
    mut result: ResMut<RunResult>,
    mut salt: ResMut<SaltMinerSystem>,
    mut combo: ResMut<ComboHeatSystem>,
    active: Res<crate::games::ActiveModule>,
    session: Res<GameSession>,
    cg: Option<Res<crate::games::caldari_gallente::CGCampaignState>>,
    ef: Option<Res<crate::games::elder_fleet::ElderFleetCampaignState>>,
    mut save: ResMut<SaveData>,
    pause: Res<PauseContext>,
    nightmare: Option<Res<crate::games::caldari_gallente::ShiigeruNightmare>>,
    last_stand: Option<Res<crate::games::caldari_gallente::LastStandState>>,
    endless: Option<Res<EndlessMode>>,
) {
    let resuming = !not_resuming_gameplay(pause);
    for transition in transitions.read() {
        if resuming && transition.exited == Some(GameState::Paused) {
            continue;
        }
        let Some(entered) = transition.entered else {
            continue;
        };
        let reset = matches!(entered, GameState::MainMenu | GameState::ShipSelect)
            || (entered == GameState::Playing
                && matches!(
                    transition.exited,
                    Some(GameState::Paused | GameState::GameOver)
                ));
        if reset {
            score.reset_game();
            *result = RunResult::default();
            *salt = SaltMinerSystem::default();
            *combo = ComboHeatSystem::default();
        } else if entered == GameState::Playing
            && matches!(
                transition.exited,
                Some(GameState::StageComplete | GameState::MissionBriefing)
            )
        {
            score.reset_stage();
        }

        if !matches!(
            entered,
            GameState::GameOver | GameState::SliceComplete | GameState::Victory
        ) || result.recorded
            || nightmare.as_deref().is_some_and(|mode| mode.active)
            || last_stand.as_deref().is_some_and(|mode| mode.active)
            || endless.as_deref().is_some_and(|mode| mode.active)
        {
            continue;
        }
        // The exposed chapters retain their existing victory-save keys. Hidden
        // modes keep their existing persistence until separately qualified.
        let (faction, enemy, stage) = if active.is_caldari_gallente() {
            (
                format!("cg_{}", session.player_faction.short_name()),
                format!("cg_{}", session.enemy_faction.short_name()),
                cg.as_deref()
                    .map(|c| c.mission_number() as u32)
                    .unwrap_or(1),
            )
        } else if active.is_elder_fleet() {
            (
                session.player_faction.name().to_string(),
                session.enemy_faction.name().to_string(),
                ef.as_deref()
                    .map(|c| {
                        if entered == GameState::Victory {
                            c.current_mission
                        } else {
                            c.current_mission + 1
                        }
                    })
                    .unwrap_or(1),
            )
        } else {
            continue;
        };
        *result = RunResult {
            score: score.score,
            previous_best: save.get_high_score(&faction, &enemy),
            recorded: true,
        };
        save.record_run_score(&faction, &enemy, score.score, stage, score.run);
    }
}

/// Update salt miner meter and handle activation input
fn update_salt_miner_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<crate::systems::JoystickState>,
    bindings: Res<KeyBindings>,
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

    // Keep charged combat utility reachable without leaving the aim stick.
    let activate_pressed = if bindings.controller_only {
        joystick.just_pressed(4)
    } else {
        keyboard.just_pressed(KeyCode::KeyB) || joystick.salt_miner()
    };

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
