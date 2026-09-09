//! Scripted native rendering review, separate from the shipped game.
//! Run from the checkout: cargo run --example transport_review -- minmatar
//! (or amarr). Uses a fresh muted save and exits after capturing briefing,
//! combat and objective-failure screens. This is not a human playthrough.
use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};
use rebellion::{
    app_builder::RebellionAppConfig,
    core::*,
    entities::*,
    games::{
        elder_fleet::{transport::*, ElderFleetCampaignState},
        ActiveModule,
    },
};
use std::path::PathBuf;
mod support;

#[derive(Resource)]
struct Review {
    faction: Faction,
    output: PathBuf,
    phase: u8,
    seconds: f32,
    samples_ms: Vec<f64>,
}

fn main() {
    let faction = match std::env::args().nth(1).as_deref() {
        Some("amarr") => Faction::Amarr,
        Some("minmatar") | None => Faction::Minmatar,
        _ => panic!("expected minmatar or amarr"),
    };
    let output = std::env::current_dir()
        .unwrap()
        .join("build/playtest-review/transport-20260908")
        .join(faction.short_name().to_lowercase());
    std::fs::create_dir_all(&output).unwrap();
    let profile =
        std::env::temp_dir().join(format!("rebellion-transport-review-{}", std::process::id()));
    std::fs::create_dir(&profile).unwrap();
    std::fs::write(profile.join("save.json"), r#"{"stage_progress":[],"unlocked_ships":[],"lifetime_credits":0,"high_scores":[],"settings":{"master_volume":0,"music_volume":0,"sfx_volume":0}}"#).unwrap();
    std::env::set_var("REBELLION_HOME", profile);
    fastrand::seed(1944);
    let mut app = RebellionAppConfig::native().build();
    support::install_controller(&mut app, 0.0);
    for mut window in app
        .world_mut()
        .query::<&mut Window>()
        .iter_mut(app.world_mut())
    {
        window.title = "Rebellion — isolated transport review".into();
        window.focused = false;
    }
    app.insert_resource(Review {
        faction,
        output,
        phase: 0,
        seconds: 0.0,
        samples_ms: Vec::with_capacity(4096),
    })
    .add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    })
    .add_systems(PostUpdate, review)
    .run();
}

fn capture(commands: &mut Commands, review: &Review, name: &str) {
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(review.output.join(name)));
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn review(
    mut commands: Commands,
    mut review: ResMut<Review>,
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    mut active: ResMut<ActiveModule>,
    mut campaign: ResMut<ElderFleetCampaignState>,
    mut session: ResMut<GameSession>,
    mut player: Query<
        (&mut Transform, &mut PowerupEffects),
        (With<Player>, Without<MissionTransport>),
    >,
    transport: Query<&Transform, With<MissionTransport>>,
    mut exit: EventWriter<AppExit>,
) {
    review.seconds += time.delta_secs();
    if review.phase == 0 && *state.get() == GameState::MainMenu {
        active.set_module("elder_fleet");
        active.player_faction = Some(review.faction.short_name().to_lowercase());
        let rival = if review.faction == Faction::Amarr {
            Faction::Minmatar
        } else {
            Faction::Amarr
        };
        *session = GameSession::new(review.faction, rival);
        campaign.current_mission = 1;
        next.set(GameState::MissionBriefing);
        review.phase = 1;
        review.seconds = 0.0;
    } else if review.phase == 1 && review.seconds > 1.0 {
        capture(&mut commands, &review, "briefing.png");
        review.phase = 2;
    } else if review.phase == 2 && review.seconds > 2.0 {
        next.set(GameState::Playing);
        review.phase = 3;
        review.seconds = 0.0;
    }
    if *state.get() == GameState::Playing {
        if let Ok((mut transform, mut powerups)) = player.get_single_mut() {
            powerups.invuln_timer = 2.0;
            if let Ok(target) = transport.get_single() {
                transform.translation.x = target.translation.x + 75.0;
                transform.translation.y = target.translation.y - 35.0;
            }
        }
        if review.seconds > 2.0 && review.samples_ms.len() < 4096 {
            review.samples_ms.push(time.delta_secs_f64() * 1000.0);
        }
    }
    if review.phase == 3 && review.seconds > 6.0 {
        capture(&mut commands, &review, "encounter.png");
        review.phase = 4;
    }
    if review.phase == 4 && review.seconds > 8.0 {
        if let Ok(target) = transport.get_single() {
            commands.spawn((
                EnemyProjectile,
                ProjectileDamage {
                    damage: 10_000.0,
                    ..default()
                },
                *target,
            ));
        }
        review.phase = 5;
    }
    if review.phase == 5 && *state.get() == GameState::GameOver {
        review.phase = 6;
        review.seconds = 0.0;
    }
    if review.phase == 6 && review.seconds > 1.0 {
        capture(&mut commands, &review, "failure.png");
        review.phase = 7;
    }
    if review.phase == 7 && review.seconds > 2.0 {
        review.samples_ms.sort_by(f64::total_cmp);
        let samples = &review.samples_ms;
        let percentile = |p: f64| {
            samples
                .get(((samples.len().saturating_sub(1)) as f64 * p).round() as usize)
                .copied()
        };
        let report = serde_json::json!({"fixture": "scripted transport encounter; invulnerable pilot; includes screenshot capture", "profile": if cfg!(debug_assertions) { "debug" } else { "release" }, "samples": samples.len(), "p50_ms": percentile(0.5), "p95_ms": percentile(0.95), "p99_ms": percentile(0.99), "max_ms": samples.last()});
        std::fs::write(
            review.output.join("frame-times.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        exit.send(AppExit::Success);
    } else if review.seconds > 45.0 {
        panic!(
            "render review did not finish: {:?}, phase {}",
            state.get(),
            review.phase
        );
    }
}
