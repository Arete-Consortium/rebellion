//! Native result-screen fixture with synthetic statistics and a private muted save.
//! This captures presentation, not a completed campaign or physical controller test.
//! cargo run --offline --locked --example run_results_review -- <fresh-output-dir>
use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};
use rebellion::{
    app_builder::{ControllerGate, RebellionAppConfig},
    core::{Faction, GameSession, GameState, RunResult, RunStatistics, SaveData, ScoreSystem},
    games::{
        caldari_gallente::CGCampaignState, elder_fleet::ElderFleetCampaignState, ActiveModule,
    },
    systems::dialogue::DialogueSystem,
};
use std::path::{Path, PathBuf};
mod support;

#[derive(Resource)]
struct Review {
    output: PathBuf,
    elapsed: f32,
    phase_elapsed: f32,
    phase: u8,
}

impl Review {
    fn advance(&mut self, phase: u8) {
        self.phase = phase;
        self.phase_elapsed = 0.0;
    }

    fn captured(&self, name: &str) -> bool {
        self.output
            .join(format!("{name}.png"))
            .metadata()
            .is_ok_and(|file| file.len() > 0)
    }
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("supply a fresh output directory");
    std::fs::create_dir_all(&output).unwrap();
    for name in [
        "stage.png",
        "slice.png",
        "death.png",
        "victory.png",
        "review-result.json",
    ] {
        assert!(
            !output.join(name).exists(),
            "review output already exists: {name}"
        );
    }
    let profile = std::env::temp_dir().join(format!(
        "rebellion-run-results-review-{}",
        std::process::id()
    ));
    std::fs::create_dir(&profile).unwrap();
    std::fs::write(profile.join("save.json"), r#"{"stage_progress":[],"unlocked_ships":[],"lifetime_credits":0,"high_scores":[],"settings":{"master_volume":0,"music_volume":0,"sfx_volume":0}}"#).unwrap();
    std::env::set_var("REBELLION_HOME", &profile);
    let mut app = RebellionAppConfig::native().build();
    support::install_controller(&mut app, 0.0);
    for mut window in app
        .world_mut()
        .query::<&mut Window>()
        .iter_mut(app.world_mut())
    {
        window.title = "Rebellion — isolated run result review".into();
        window.resolution = (800.0_f32, 700.0_f32).into();
        window.focused = false;
    }
    app.insert_resource(Review {
        output,
        elapsed: 0.0,
        phase_elapsed: 0.0,
        phase: 0,
    })
    .add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    })
    .add_systems(PostUpdate, review)
    .run();
}

fn sample_score() -> ScoreSystem {
    // The live combo has expired; retained run statistics must still be shown.
    ScoreSystem {
        score: 12_345,
        run: RunStatistics {
            max_chain: 14,
            max_multiplier: 2.4,
            combat_seconds: 125.2,
        },
        souls_liberated: 42,
        ..default()
    }
}

fn capture(
    commands: &mut Commands,
    output: &Path,
    name: &str,
    texts: &Query<&Text>,
    score: &ScoreSystem,
    comparison: Option<&str>,
) {
    assert_eq!(score.chain, 0, "live combo must be expired in this fixture");
    assert_eq!(score.run.max_chain, 14);
    assert_eq!(score.run.combat_time(), "2:05");
    let summary = score.run.summary();
    assert!(
        texts.iter().any(|text| text.0 == summary),
        "{name}: run summary missing"
    );
    if let Some(comparison) = comparison {
        assert!(
            texts.iter().any(|text| text.0 == comparison),
            "{name}: personal-best comparison missing"
        );
    }
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(output.join(format!("{name}.png"))));
}

#[allow(clippy::too_many_arguments)]
fn review(
    mut commands: Commands,
    mut review: ResMut<Review>,
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    mut active: ResMut<ActiveModule>,
    mut session: ResMut<GameSession>,
    mut cg: ResMut<CGCampaignState>,
    mut ef: ResMut<ElderFleetCampaignState>,
    mut score: ResMut<ScoreSystem>,
    result: Res<RunResult>,
    mut save: ResMut<SaveData>,
    mut dialogue: ResMut<DialogueSystem>,
    gate: Res<ControllerGate>,
    texts: Query<&Text>,
    mut exit: EventWriter<AppExit>,
) {
    review.elapsed += time.delta_secs();
    review.phase_elapsed += time.delta_secs();
    assert!(
        review.elapsed < 24.0,
        "result review timed out in phase {}",
        review.phase
    );
    dialogue.clear();
    dialogue.queue.clear();
    match review.phase {
        0 if *state.get() == GameState::MainMenu && !gate.is_blocked() => {
            active.set_module("caldari_gallente");
            active.set_faction("caldari", "gallente");
            *session = GameSession::new(Faction::Caldari, Faction::Gallente);
            *cg = CGCampaignState::default();
            *score = sample_score();
            save.record_score("cg_CALDARI", "cg_GALLENTE", 10_000, 3);
            next.set(GameState::StageComplete);
            review.advance(1);
        }
        1 if *state.get() == GameState::StageComplete && review.phase_elapsed > 1.5 => {
            capture(&mut commands, &review.output, "stage", &texts, &score, None);
            review.advance(2);
        }
        2 if review.captured("stage") => {
            cg.mission_index = 2;
            next.set(GameState::SliceComplete);
            review.advance(3);
        }
        3 if *state.get() == GameState::SliceComplete && review.phase_elapsed > 1.5 => {
            assert!(result.recorded);
            assert_eq!(result.previous_best, 10_000);
            capture(
                &mut commands,
                &review.output,
                "slice",
                &texts,
                &score,
                Some("New personal best +2345"),
            );
            review.advance(4);
        }
        4 if review.captured("slice") => {
            // Actual attempt-boundary reset; do not carry the completed result into death.
            next.set(GameState::MainMenu);
            review.advance(5);
        }
        5 if *state.get() == GameState::MainMenu => {
            assert!(!result.recorded);
            *score = sample_score();
            cg.mission_index = 1;
            save.record_score("cg_CALDARI", "cg_GALLENTE", 20_000, 3);
            next.set(GameState::GameOver);
            review.advance(6);
        }
        6 if *state.get() == GameState::GameOver && review.phase_elapsed > 1.5 => {
            assert!(result.recorded);
            assert_eq!(result.previous_best, 20_000);
            capture(
                &mut commands,
                &review.output,
                "death",
                &texts,
                &score,
                Some("7655 to personal best"),
            );
            review.advance(7);
        }
        7 if review.captured("death") => {
            next.set(GameState::MainMenu);
            review.advance(8);
        }
        8 if *state.get() == GameState::MainMenu => {
            assert!(!result.recorded);
            active.set_module("elder_fleet");
            active.set_faction("minmatar", "amarr");
            *session = GameSession::new(Faction::Minmatar, Faction::Amarr);
            ef.current_mission = ef.total_missions();
            *score = sample_score();
            save.record_score(Faction::Minmatar.name(), Faction::Amarr.name(), 12_345, 9);
            next.set(GameState::Victory);
            review.advance(9);
        }
        9 if *state.get() == GameState::Victory && review.phase_elapsed > 1.5 => {
            assert!(result.recorded);
            assert_eq!(result.previous_best, 12_345);
            capture(
                &mut commands,
                &review.output,
                "victory",
                &texts,
                &score,
                Some("Personal best tied"),
            );
            review.advance(10);
        }
        10 if review.captured("victory") => {
            let evidence = serde_json::json!({
                "scripted_not_human_playtest": true,
                "statistics_are_synthetic": true,
                "logical_window": [800, 700],
                "screenshots": ["stage.png", "slice.png", "death.png", "victory.png"],
                "score": 12345,
                "expired_live_chain": 0,
                "max_chain": 14,
                "max_multiplier": 2.4,
                "combat_seconds": 125.2,
                "displayed_combat_time": "2:05",
                "comparison_texts_verified": ["New personal best +2345", "7655 to personal best", "Personal best tied"],
                "elapsed_seconds": review.elapsed,
            });
            std::fs::write(
                review.output.join("review-result.json"),
                serde_json::to_vec_pretty(&evidence).unwrap(),
            )
            .unwrap();
            exit.send(AppExit::Success);
            review.advance(11);
        }
        _ => {}
    }
}
