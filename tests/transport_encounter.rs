//! Objective proof through the production fixed-step and campaign schedules.
//! Wave clearance is accelerated via health; boarding, damage, pause and boss
//! transitions are real. These checks do not qualify human difficulty.
use bevy::prelude::*;
use rebellion::app_builder::build_headless_app;
use rebellion::core::{GameState, PauseContext};
use rebellion::entities::{
    Enemy, EnemyProjectile, EnemyStats, EscortData, Player, PlayerProjectile, ProjectileDamage,
};
use rebellion::games::elder_fleet::transport::*;
use rebellion::games::elder_fleet::{ElderFleetCampaignState, ElderFleetPlugin};
use rebellion::games::{ActiveModule, ModuleRegistry};

fn state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}

fn setup(faction: &str, mission: u32) -> App {
    let mut app = build_headless_app();
    app.init_resource::<ModuleRegistry>()
        .add_plugins(ElderFleetPlugin);
    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("elder_fleet");
        active.player_faction = Some(faction.into());
    }
    app.world_mut()
        .resource_mut::<ElderFleetCampaignState>()
        .current_mission = mission;
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    move_player(&mut app, Vec2::new(-300.0, -230.0));
    app
}

fn move_player(app: &mut App, pos: Vec2) {
    for mut transform in app
        .world_mut()
        .query_filtered::<&mut Transform, With<Player>>()
        .iter_mut(app.world_mut())
    {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

fn transport(app: &mut App) -> (Entity, Vec2) {
    let (entity, transform) = app
        .world_mut()
        .query_filtered::<(Entity, &Transform), With<MissionTransport>>()
        .single(app.world());
    (entity, transform.translation.truncate())
}

fn clear_wave(app: &mut App) {
    for mut stats in app
        .world_mut()
        .query_filtered::<&mut EnemyStats, Without<rebellion::entities::Boss>>()
        .iter_mut(app.world_mut())
    {
        stats.health = 0.0;
    }
}

fn tick(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

#[test]
fn only_hedion_missions_spawn_a_non_enemy_transport() {
    for faction in ["minmatar", "amarr"] {
        for mission in [0, 1, 2] {
            let mut app = setup(faction, mission);
            let count = app
                .world_mut()
                .query_filtered::<Entity, With<MissionTransport>>()
                .iter(app.world())
                .count();
            assert_eq!(count, usize::from(mission == 1));
            let objective = app.world().resource::<TransportObjective>();
            assert_eq!(objective.phase == TransportPhase::Active, mission == 1);
            if mission == 1 {
                let (entity, _) = transport(&mut app);
                assert!(
                    app.world().get::<Enemy>(entity).is_none(),
                    "transport must not count as an enemy or receive player fire"
                );
            }
        }
    }
}

#[test]
fn boarding_progress_requires_proximity_and_survives_dodging_and_pause() {
    let mut app = setup("minmatar", 1);
    tick(&mut app, 30);
    assert_eq!(
        app.world().resource::<TransportObjective>().work_seconds,
        0.0
    );
    let (entity, pos) = transport(&mut app);
    move_player(&mut app, pos + Vec2::new(70.0, 0.0));
    tick(&mut app, 30);
    let work = app.world().resource::<TransportObjective>().work_seconds;
    assert!(work > 0.4);
    move_player(&mut app, Vec2::new(-300.0, -230.0));
    tick(&mut app, 20);
    assert_eq!(
        app.world().resource::<TransportObjective>().work_seconds,
        work
    );
    let elapsed = app.world().resource::<TransportObjective>().elapsed;
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Paused);
    tick(&mut app, 60);
    assert_eq!(
        app.world().resource::<TransportObjective>().elapsed,
        elapsed
    );
    assert!(app.world().get::<MissionTransport>(entity).is_some());
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    assert_eq!(
        transport(&mut app).0,
        entity,
        "resume must preserve the escort"
    );
    assert_eq!(
        app.world().resource::<TransportObjective>().work_seconds,
        work
    );
    assert!(app.world().resource::<TransportObjective>().elapsed < elapsed + 0.1);
}

#[test]
fn hostile_projectiles_damage_the_transport_but_player_fire_does_not() {
    let mut app = setup("amarr", 1);
    let (entity, pos) = transport(&mut app);
    app.world_mut().spawn((
        PlayerProjectile,
        ProjectileDamage {
            damage: 1000.0,
            ..default()
        },
        Transform::from_translation(pos.extend(9.0)),
    ));
    app.update();
    assert_eq!(
        app.world().get::<EscortData>(entity).unwrap().health,
        TRANSPORT_HEALTH
    );
    let projectile = app
        .world_mut()
        .spawn((
            EnemyProjectile,
            ProjectileDamage {
                damage: 37.0,
                ..default()
            },
            Transform::from_translation(pos.extend(9.0)),
        ))
        .id();
    app.update();
    assert_eq!(
        app.world().get::<EscortData>(entity).unwrap().health,
        TRANSPORT_HEALTH - 37.0
    );
    assert!(
        app.world().get_entity(projectile).is_err(),
        "a hit consumes the projectile"
    );
    assert_eq!(state(&app), GameState::Playing);
}

#[test]
fn pilot_can_intercept_a_shot_without_double_damage_to_the_transport() {
    let mut app = setup("amarr", 1);
    let (entity, pos) = transport(&mut app);
    move_player(&mut app, pos);
    let projectile = app
        .world_mut()
        .spawn((
            EnemyProjectile,
            ProjectileDamage {
                damage: 10.0,
                ..default()
            },
            Transform::from_translation(pos.extend(9.0)),
        ))
        .id();
    app.update();
    assert!(app.world().get_entity(projectile).is_err());
    assert_eq!(
        app.world().get::<EscortData>(entity).unwrap().health,
        TRANSPORT_HEALTH
    );
}

#[test]
fn destruction_at_the_completion_boundary_fails_without_unlocking_the_boss() {
    let mut app = setup("amarr", 1);
    let (_, pos) = transport(&mut app);
    move_player(&mut app, pos + Vec2::new(80.0, 0.0));
    app.world_mut()
        .resource_mut::<TransportObjective>()
        .work_seconds = ESCORT_SECONDS - 0.001;
    app.world_mut().spawn((
        EnemyProjectile,
        ProjectileDamage {
            damage: 1000.0,
            ..default()
        },
        Transform::from_translation(pos.extend(9.0)),
    ));
    tick(&mut app, 3);
    assert_eq!(state(&app), GameState::GameOver);
    assert_eq!(
        app.world().resource::<TransportObjective>().failure(),
        Some(TransportFailure::Destroyed)
    );
    let campaign = app.world().resource::<ElderFleetCampaignState>();
    assert_eq!(campaign.current_mission, 1);
    assert!(!campaign.boss_spawned && !campaign.mission_complete);
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<MissionTransport>>()
            .iter(app.world())
            .count(),
        0
    );
}

#[test]
fn cleared_waves_do_not_skip_the_deadline_and_retry_starts_fresh() {
    let mut app = setup("minmatar", 1);
    for _ in 0..5500 {
        clear_wave(&mut app);
        app.update();
        if state(&app) == GameState::GameOver {
            break;
        }
        assert_eq!(
            state(&app),
            GameState::Playing,
            "unfinished transport must block BossIntro"
        );
    }
    assert_eq!(state(&app), GameState::GameOver);
    assert_eq!(
        app.world().resource::<TransportObjective>().failure(),
        Some(TransportFailure::Escaped)
    );
    assert_eq!(
        app.world()
            .resource::<ElderFleetCampaignState>()
            .current_wave,
        3
    );
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::ShipSelect);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    let objective = app.world().resource::<TransportObjective>();
    assert_eq!(objective.phase, TransportPhase::Active);
    assert_eq!(objective.work_seconds, 0.0);
    assert!(objective.elapsed < 0.1);
    let (entity, _) = transport(&mut app);
    assert_eq!(
        app.world().get::<EscortData>(entity).unwrap().health,
        TRANSPORT_HEALTH
    );
    assert_eq!(
        app.world()
            .resource::<ElderFleetCampaignState>()
            .current_mission,
        1
    );
}

#[test]
fn explicit_restart_from_pause_replaces_the_transport_once() {
    let mut app = setup("minmatar", 1);
    let (previous, pos) = transport(&mut app);
    move_player(&mut app, pos);
    tick(&mut app, 10);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Paused);
    app.update();
    app.world_mut()
        .resource_mut::<PauseContext>()
        .request_restart();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    assert_ne!(transport(&mut app).0, previous);
    assert!(app.world().get_entity(previous).is_err());
    assert!(app.world().resource::<TransportObjective>().elapsed < 0.1);
}

#[test]
fn both_faction_objectives_gate_carrier_waves_then_real_boss_damage_advances_once() {
    for faction in ["minmatar", "amarr"] {
        let mut app = setup(faction, 1);
        let mut arrivals = std::collections::BTreeSet::new();
        for _ in 0..4000 {
            let (_, pos) = transport(&mut app);
            // Keep offset from the hull, so this does not body-block its shots.
            move_player(&mut app, pos + Vec2::new(70.0, 0.0));
            clear_wave(&mut app);
            app.update();
            for carrier in app
                .world_mut()
                .query::<&rebellion::systems::spawning::EnemyCarrier>()
                .iter(app.world())
            {
                arrivals.insert(carrier.wave_number);
            }
            if state(&app) == GameState::BossIntro {
                break;
            }
        }
        assert_eq!(
            state(&app),
            GameState::BossIntro,
            "{faction} failed: {:?}",
            app.world().resource::<TransportObjective>()
        );
        assert_eq!(arrivals.into_iter().collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(
            app.world().resource::<TransportObjective>().phase,
            TransportPhase::Secured
        );
        tick(&mut app, 140);
        assert_eq!(state(&app), GameState::BossFight);
        let boss_pos = app
            .world_mut()
            .query_filtered::<&Transform, With<rebellion::entities::Boss>>()
            .single(app.world())
            .translation;
        app.world_mut().spawn((
            PlayerProjectile,
            ProjectileDamage {
                damage: 100_000.0,
                crit_chance: 0.0,
                ..default()
            },
            Transform::from_translation(boss_pos),
        ));
        tick(&mut app, 5);
        assert_eq!(state(&app), GameState::StageComplete);
        assert_eq!(
            app.world()
                .resource::<ElderFleetCampaignState>()
                .current_mission,
            2
        );
        tick(&mut app, 5);
        assert_eq!(
            app.world()
                .resource::<ElderFleetCampaignState>()
                .current_mission,
            2
        );
    }
}
