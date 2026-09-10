//! Attempt statistics and result persistence through the production score lifecycle.
//!
//! The harness deliberately omits SavePlugin: every test owns an in-memory
//! SaveData and cannot read or change a player's saved profile.

use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use rebellion::core::{
    CampaignState, EnemyDestroyedEvent, Faction, GameSession, GameState, HighScore, KeyBindings,
    PauseContext, PauseLifecyclePlugin, RunResult, RunStatistics, SaltMinerEndedEvent,
    SaltMinerSystem, SaveData, ScoreSystem,
};
use rebellion::entities::{Enemy, EnemyStats, Player, ShipStats};
use rebellion::gameplay::combat_outcomes::enemy_death_outcomes;
use rebellion::games::caldari_gallente::{CGCampaignState, LastStandState};
use rebellion::games::elder_fleet::ElderFleetCampaignState;
use rebellion::games::ActiveModule;
use rebellion::simulation::resolve_deaths::resolve_enemy_deaths;
use rebellion::simulation::SimulationRng;
use rebellion::systems::{
    ComboHeatSystem, DialogueEvent, JoystickState, RumbleRequest, ScoringPlugin, ScreenFlash,
    ScreenShake,
};
use rebellion::PowerupIconCache;

#[derive(Resource, Default)]
struct ResultsAtEntry(Vec<RunResult>);

fn capture_result_on_entry(result: Res<RunResult>, mut entries: ResMut<ResultsAtEntry>) {
    entries.0.push(result.clone());
}

fn scoring_app(module: &str, player: Faction) -> App {
    let mut active = ActiveModule::default();
    active.set_module(module);
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .init_state::<GameState>()
        .insert_resource(active)
        .insert_resource(GameSession::new(player, player.rival()))
        .init_resource::<SaveData>()
        .init_resource::<CGCampaignState>()
        .init_resource::<ElderFleetCampaignState>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<KeyBindings>()
        .init_resource::<JoystickState>()
        .init_resource::<ScreenFlash>()
        .init_resource::<ScreenShake>()
        .init_resource::<CampaignState>()
        .init_resource::<PowerupIconCache>()
        .insert_resource(SimulationRng::from_seed(42))
        .init_resource::<ResultsAtEntry>()
        .add_event::<SaltMinerEndedEvent>()
        .add_event::<DialogueEvent>()
        .add_event::<RumbleRequest>()
        .add_event::<EnemyDestroyedEvent>()
        .add_plugins((PauseLifecyclePlugin, ScoringPlugin))
        .add_systems(OnEnter(GameState::GameOver), capture_result_on_entry)
        .add_systems(OnEnter(GameState::SliceComplete), capture_result_on_entry)
        .add_systems(OnEnter(GameState::Victory), capture_result_on_entry);
    app.world_mut().run_schedule(StateTransition);
    app
}

fn transition(app: &mut App, destination: GameState) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(destination);
    app.world_mut().run_schedule(StateTransition);
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        destination
    );
}

fn resume(app: &mut App, expected: GameState) {
    app.world_mut()
        .resource_scope(|world, pause: Mut<PauseContext>| {
            pause.resume(&mut world.resource_mut::<NextState<GameState>>());
        });
    app.world_mut().run_schedule(StateTransition);
    assert_eq!(*app.world().resource::<State<GameState>>().get(), expected);
}

fn fixed_tick(app: &mut App, seconds: f64) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f64(seconds));
    app.world_mut().run_schedule(FixedUpdate);
}

fn statistics(chain: u32, seconds: f64) -> RunStatistics {
    RunStatistics {
        max_chain: chain,
        max_multiplier: 1.0 + chain as f32 * 0.1,
        combat_seconds: seconds,
    }
}

fn set_run(app: &mut App, points: u64, run: RunStatistics) {
    let mut score = app.world_mut().resource_mut::<ScoreSystem>();
    score.score = points;
    score.run = run;
}

#[test]
fn enemy_deaths_record_peaks_that_survive_chain_expiry() {
    let mut app = scoring_app("caldari_gallente", Faction::Caldari);
    transition(&mut app, GameState::Playing);
    app.world_mut()
        .spawn((Player, ShipStats::default(), Transform::default()));
    let enemies: Vec<_> = (0..3)
        .map(|_| {
            app.world_mut()
                .spawn((
                    Enemy,
                    EnemyStats {
                        health: 0.0,
                        score_value: 100,
                        liberation_value: 0,
                        ..default()
                    },
                    Transform::from_xyz(0.0, 400.0, 0.0),
                ))
                .id()
        })
        .collect();
    let mut deaths = Schedule::default();
    deaths.add_systems((resolve_enemy_deaths, enemy_death_outcomes).chain());
    deaths.run(app.world_mut());

    assert!(enemies
        .iter()
        .all(|enemy| app.world().get_entity(*enemy).is_err()));
    let score = app.world().resource::<ScoreSystem>();
    assert_eq!(score.chain, 3);
    assert_eq!(score.run.max_chain, 3);
    assert!((score.run.max_multiplier - 1.3).abs() < f32::EPSILON);
    assert!(
        score.score >= 350,
        "death outcomes must award actual kill points"
    );
    let awarded_points = score.score;
    let expiry = f64::from(score.max_chain_time) + 0.25;
    fixed_tick(&mut app, expiry);
    let score = app.world().resource::<ScoreSystem>();
    assert_eq!(score.chain, 0);
    assert_eq!(score.multiplier, 1.0);
    assert_eq!(score.run.max_chain, 3);
    assert!((score.run.max_multiplier - 1.3).abs() < f32::EPSILON);
    assert_eq!(score.score, awarded_points);
    assert_eq!(score.run.combat_seconds, expiry);

    deaths.run(app.world_mut());
    assert_eq!(app.world().resource::<ScoreSystem>().score, awarded_points);
}

#[test]
fn combat_time_excludes_pause_briefing_boss_intro_and_result_screens() {
    let mut app = scoring_app("caldari_gallente", Faction::Gallente);
    fixed_tick(&mut app, 20.0); // Loading.
    transition(&mut app, GameState::Playing);
    fixed_tick(&mut app, 1.25);
    transition(&mut app, GameState::Paused);
    fixed_tick(&mut app, 9.0);
    resume(&mut app, GameState::Playing);
    transition(&mut app, GameState::BossIntro);
    fixed_tick(&mut app, 7.0);
    transition(&mut app, GameState::BossFight);
    fixed_tick(&mut app, 2.0);
    transition(&mut app, GameState::Paused);
    fixed_tick(&mut app, 10.0);
    resume(&mut app, GameState::BossFight);
    transition(&mut app, GameState::StageComplete);
    fixed_tick(&mut app, 3.0);
    transition(&mut app, GameState::MissionBriefing);
    fixed_tick(&mut app, 4.0);
    transition(&mut app, GameState::Playing);
    fixed_tick(&mut app, 1.25);
    assert_eq!(
        app.world().resource::<ScoreSystem>().run.combat_seconds,
        4.5
    );
    transition(&mut app, GameState::GameOver);
    fixed_tick(&mut app, 15.0);
    assert_eq!(
        app.world().resource::<ScoreSystem>().run.combat_seconds,
        4.5
    );
}

#[test]
fn resume_preserves_attempt_but_explicit_restart_resets_it() {
    for combat_state in [GameState::Playing, GameState::BossFight] {
        let mut app = scoring_app("caldari_gallente", Faction::Caldari);
        transition(&mut app, combat_state);
        app.world_mut().resource_mut::<ScoreSystem>().on_kill(100);
        fixed_tick(&mut app, 0.25);
        app.world_mut().resource_mut::<SaltMinerSystem>().meter = 73.0;
        app.world_mut().resource_mut::<ComboHeatSystem>().on_kill();
        let before = app.world().resource::<ScoreSystem>().clone();

        transition(&mut app, GameState::Paused);
        resume(&mut app, combat_state);
        let after = app.world().resource::<ScoreSystem>();
        assert_eq!(after.score, before.score);
        assert_eq!(after.run, before.run);
        assert_eq!(after.chain, before.chain);
        assert_eq!(after.chain_timer, before.chain_timer);
        assert_eq!(app.world().resource::<SaltMinerSystem>().meter, 73.0);

        transition(&mut app, GameState::Paused);
        app.world_mut()
            .resource_mut::<PauseContext>()
            .request_restart();
        transition(&mut app, GameState::Playing);
        let score = app.world().resource::<ScoreSystem>();
        assert_eq!(score.score, 0);
        assert_eq!(score.chain, 0);
        assert_eq!(score.run, RunStatistics::default());
        assert_eq!(app.world().resource::<SaltMinerSystem>().meter, 0.0);
        assert_eq!(app.world().resource::<ComboHeatSystem>().total_kills, 0);
        assert!(!app.world().resource::<RunResult>().recorded);
    }
}

#[test]
fn mission_continuation_keeps_totals_and_new_game_clears_them() {
    for continuation in [GameState::StageComplete, GameState::MissionBriefing] {
        let mut app = scoring_app("caldari_gallente", Faction::Gallente);
        transition(&mut app, GameState::Playing);
        app.world_mut().resource_mut::<ScoreSystem>().on_kill(500);
        fixed_tick(&mut app, 0.5);
        let before = app.world().resource::<ScoreSystem>().clone();
        transition(&mut app, continuation);
        transition(&mut app, GameState::Playing);
        let score = app.world().resource::<ScoreSystem>();
        assert_eq!(score.score, before.score);
        assert_eq!(score.run, before.run);
        assert_eq!(score.chain, 0);
        assert_eq!(score.chain_timer, 0.0);
        assert_eq!(score.multiplier, 1.0);

        for new_game in [GameState::MainMenu, GameState::ShipSelect] {
            transition(&mut app, GameState::GameOver);
            assert!(app.world().resource::<RunResult>().recorded);
            transition(&mut app, new_game);
            assert_eq!(app.world().resource::<ScoreSystem>().score, 0);
            assert_eq!(
                app.world().resource::<ScoreSystem>().run,
                RunStatistics::default()
            );
            assert!(!app.world().resource::<RunResult>().recorded);
            transition(&mut app, GameState::Playing);
            set_run(&mut app, 900, statistics(7, 42.0));
        }
    }
}

#[test]
fn personal_best_keeps_matching_metadata_and_pre_save_comparison() {
    let old_run = statistics(11, 180.0);
    for (points, new_best, comparison) in [
        (900, false, "100 to personal best"),
        (1000, false, "Personal best tied"),
        (1200, true, "New personal best +200"),
    ] {
        let mut app = scoring_app("caldari_gallente", Faction::Caldari);
        app.world_mut().resource_mut::<SaveData>().record_run_score(
            "cg_CALDARI",
            "cg_GALLENTE",
            1000,
            2,
            old_run,
        );
        transition(&mut app, GameState::Playing);
        app.world_mut()
            .resource_mut::<CGCampaignState>()
            .mission_index = 2;
        let current_run = statistics(20, 240.5);
        set_run(&mut app, points, current_run);
        transition(&mut app, GameState::GameOver);

        let result = &app.world().resource::<ResultsAtEntry>().0[0];
        assert!(
            result.recorded,
            "result must be finalized before UI entry hooks"
        );
        assert_eq!(result.score, points);
        assert_eq!(result.previous_best, 1000);
        assert_eq!(result.is_new_best(), new_best);
        assert_eq!(result.comparison(), comparison);
        let save = app.world().resource::<SaveData>();
        assert_eq!(save.high_scores.len(), 1);
        let best = &save.high_scores[0];
        assert_eq!(best.score, points.max(1000));
        assert_eq!(best.stage, if new_best { 3 } else { 2 });
        assert_eq!(best.run, Some(if new_best { current_run } else { old_run }));

        // A second terminal transition cannot overwrite the captured attempt.
        set_run(&mut app, 9999, statistics(99, 999.0));
        transition(&mut app, GameState::Victory);
        assert_eq!(app.world().resource::<RunResult>().score, points);
        assert_eq!(
            app.world().resource::<SaveData>().high_scores[0].score,
            points.max(1000)
        );
    }
}

#[test]
fn legacy_high_score_metadata_stays_unknown_until_a_strictly_better_run() {
    let legacy: HighScore = serde_json::from_str(
        r#"{"player_faction":"cg_CALDARI","enemy_faction":"cg_GALLENTE","score":1000,"stage":2}"#,
    )
    .expect("old saves without run statistics remain readable");
    assert_eq!(legacy.run, None);
    let mut save = SaveData::default();
    save.high_scores.push(legacy);
    let run = statistics(12, 75.0);
    for points in [900, 1000] {
        save.record_run_score("cg_CALDARI", "cg_GALLENTE", points, 3, run);
        assert_eq!(save.high_scores[0].run, None);
        assert_eq!(save.high_scores[0].stage, 2);
    }
    save.record_run_score("cg_CALDARI", "cg_GALLENTE", 1100, 3, run);
    let round_trip: SaveData =
        serde_json::from_str(&serde_json::to_string(&save).unwrap()).unwrap();
    assert_eq!(round_trip.high_scores[0].run, Some(run));
    assert_eq!(round_trip.high_scores[0].score, 1100);
    assert_eq!(round_trip.high_scores[0].stage, 3);
}

#[test]
fn cg_death_and_slice_completion_share_each_factions_existing_score_key() {
    for player in [Faction::Caldari, Faction::Gallente] {
        let mut app = scoring_app("caldari_gallente", player);
        transition(&mut app, GameState::Playing);
        app.world_mut()
            .resource_mut::<CGCampaignState>()
            .mission_index = 2;
        let first_run = statistics(5, 120.0);
        set_run(&mut app, 1000, first_run);
        transition(&mut app, GameState::GameOver);
        assert_eq!(app.world().resource::<RunResult>().previous_best, 0);
        assert_eq!(
            app.world().resource::<RunResult>().comparison(),
            "First scored run"
        );

        transition(&mut app, GameState::Playing);
        assert_eq!(
            app.world().resource::<ScoreSystem>().run,
            RunStatistics::default()
        );
        assert_eq!(app.world().resource::<ScoreSystem>().score, 0);
        assert!(!app.world().resource::<RunResult>().recorded);
        let second_run = statistics(8, 125.0);
        set_run(&mut app, 2000, second_run);
        transition(&mut app, GameState::SliceComplete);
        let save = app.world().resource::<SaveData>();
        assert_eq!(save.high_scores.len(), 1);
        let best = &save.high_scores[0];
        assert_eq!(best.player_faction, format!("cg_{}", player.short_name()));
        assert_eq!(
            best.enemy_faction,
            format!("cg_{}", player.rival().short_name())
        );
        assert_eq!(best.score, 2000);
        assert_eq!(best.stage, 3);
        assert_eq!(best.run, Some(second_run));
        let entries = &app.world().resource::<ResultsAtEntry>().0;
        assert_eq!(entries.len(), 2);
        assert!(entries[1].recorded);
        assert_eq!(entries[1].previous_best, 1000);
        assert!(entries[1].is_new_best());
    }
}

#[test]
fn elder_fleet_final_death_and_victory_record_mission_nine_for_both_sides() {
    for player in [Faction::Minmatar, Faction::Amarr] {
        let mut app = scoring_app("elder_fleet", player);
        transition(&mut app, GameState::Playing);
        app.world_mut()
            .resource_mut::<ElderFleetCampaignState>()
            .current_mission = 8;
        set_run(&mut app, 1000, statistics(7, 500.0));
        transition(&mut app, GameState::GameOver);
        assert_eq!(app.world().resource::<SaveData>().high_scores[0].stage, 9);

        transition(&mut app, GameState::Playing);
        let victorious_run = statistics(18, 650.0);
        set_run(&mut app, 2000, victorious_run);
        // EF increments the zero-based index after completing the final mission.
        app.world_mut()
            .resource_mut::<ElderFleetCampaignState>()
            .current_mission = 9;
        transition(&mut app, GameState::Victory);
        let save = app.world().resource::<SaveData>();
        assert_eq!(save.high_scores.len(), 1);
        let best = &save.high_scores[0];
        assert_eq!(best.player_faction, player.name());
        assert_eq!(best.enemy_faction, player.rival().name());
        assert_eq!(best.stage, 9);
        assert_eq!(best.run, Some(victorious_run));
        let entry = &app.world().resource::<ResultsAtEntry>().0[1];
        assert!(entry.recorded);
        assert_eq!(entry.previous_best, 1000);
        assert!(entry.is_new_best());
    }
}

#[test]
fn last_stand_exit_cleanup_cannot_publish_a_normal_campaign_record() {
    let mut app = scoring_app("caldari_gallente", Faction::Caldari);
    app.init_resource::<LastStandState>().add_systems(
        OnExit(GameState::Playing),
        |mut mode: ResMut<LastStandState>| mode.active = false,
    );
    transition(&mut app, GameState::Playing);
    app.world_mut().resource_mut::<LastStandState>().active = true;
    set_run(&mut app, 100_000, statistics(100, 1000.0));
    transition(&mut app, GameState::GameOver);

    assert!(!app.world().resource::<LastStandState>().active);
    assert!(app.world().resource::<SaveData>().high_scores.is_empty());
    assert!(!app.world().resource::<RunResult>().recorded);
    assert!(!app.world().resource::<ResultsAtEntry>().0[0].recorded);
}
