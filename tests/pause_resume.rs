//! Exercise production entry/exit hooks without advancing combat between transitions.

use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{GameState, PauseContext};
use rebellion::entities::{
    Enemy, EnemyProjectile, EnemyProjectileBundle, Friendly, Player, PlayerProjectile,
    PlayerProjectileBundle, ShipStats,
};
use rebellion::games::abyssal_depths::{AbyssalRoom, AbyssalState};
use rebellion::games::caldari_gallente::cg_campaign::{CGBoss, CGSpawnTelegraph};
use rebellion::games::caldari_gallente::last_stand_gameplay::LastStandTitan;
use rebellion::games::caldari_gallente::{CGCampaignState, CGSessionTimer, LastStandState};
use rebellion::games::{ActiveModule, GameModulesPlugin};
use rebellion::systems::spawning::{EnemyCarrier, WaveManager};

fn setup(module: &str) -> App {
    let mut app = build_headless_app();
    app.add_plugins(GameModulesPlugin);
    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module(module);
    app.update(); // Initialize startup resources while still in Loading.
    app
}

fn transition(app: &mut App, state: GameState) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(state);
    app.world_mut().run_schedule(StateTransition);
    assert_eq!(*app.world().resource::<State<GameState>>().get(), state);
}

fn resume(app: &mut App) {
    app.world_mut()
        .resource_scope(|world, pause: Mut<PauseContext>| {
            pause.resume(&mut world.resource_mut::<NextState<GameState>>());
        });
    app.world_mut().run_schedule(StateTransition);
}

fn only_entity<T: Component>(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<T>>()
        .get_single(app.world())
        .expect("exactly one entity should exist")
}

#[test]
fn resume_preserves_healthy_and_damaged_player_entities() {
    for damaged in [false, true] {
        let mut app = setup("caldari_gallente");
        transition(&mut app, GameState::Playing);
        let player = only_entity::<Player>(&mut app);
        if damaged {
            let mut stats = app.world_mut().get_mut::<ShipStats>(player).unwrap();
            stats.shield = 0.0;
            stats.armor = 7.0;
            stats.hull = 11.0;
            stats.capacitor = 13.0;
        }
        let before = app.world().get::<ShipStats>(player).unwrap().clone();
        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation = Vec3::new(91.0, -47.0, 10.0);

        for _ in 0..3 {
            transition(&mut app, GameState::Paused);
            assert_eq!(only_entity::<Player>(&mut app), player);
            resume(&mut app);
            assert_eq!(
                *app.world().resource::<State<GameState>>().get(),
                GameState::Playing
            );
            assert_eq!(only_entity::<Player>(&mut app), player);
            let after = app.world().get::<ShipStats>(player).unwrap();
            assert_eq!(after.shield, before.shield);
            assert_eq!(after.armor, before.armor);
            assert_eq!(after.hull, before.hull);
            assert_eq!(after.capacitor, before.capacitor);
            assert_eq!(
                app.world().get::<Transform>(player).unwrap().translation,
                Vec3::new(91.0, -47.0, 10.0)
            );
        }
    }
}

#[test]
fn cg_resume_preserves_wave_timer_enemies_escorts_and_carrier() {
    let mut app = setup("caldari_gallente");
    transition(&mut app, GameState::Playing);
    {
        let mut campaign = app.world_mut().resource_mut::<CGCampaignState>();
        campaign.current_wave = 3;
        campaign.wave_delay_timer = 1.25;
    }
    app.world_mut().resource_mut::<CGSessionTimer>().start(42.0);
    app.world_mut().resource_mut::<WaveManager>().wave = 7;
    let enemy = app.world_mut().spawn(Enemy).id();
    let escort = app.world_mut().spawn(Friendly).id();
    let carrier = only_entity::<EnemyCarrier>(&mut app);
    let player_projectile = app
        .world_mut()
        .spawn(PlayerProjectileBundle::default())
        .id();
    let enemy_projectile = app.world_mut().spawn(EnemyProjectileBundle::default()).id();
    let telegraph = app.world_mut().spawn(CGSpawnTelegraph { timer: 0.0 }).id();
    let preserved = [
        enemy,
        escort,
        carrier,
        player_projectile,
        enemy_projectile,
        telegraph,
    ];

    transition(&mut app, GameState::Paused);
    for entity in preserved {
        assert!(app.world().get_entity(entity).is_ok());
    }
    resume(&mut app);

    let campaign = app.world().resource::<CGCampaignState>();
    assert_eq!(campaign.current_wave, 3);
    assert_eq!(campaign.wave_delay_timer, 1.25);
    assert_eq!(
        app.world().resource::<CGSessionTimer>().start_time,
        Some(42.0)
    );
    assert_eq!(app.world().resource::<WaveManager>().wave, 7);
    for entity in preserved {
        assert!(app.world().get_entity(entity).is_ok());
    }
    assert_eq!(only_entity::<EnemyCarrier>(&mut app), carrier);
}

#[test]
fn boss_resume_restores_boss_fight_and_preserves_boss_damage_and_phase() {
    let mut app = setup("caldari_gallente");
    transition(&mut app, GameState::Playing);
    let player = only_entity::<Player>(&mut app);
    app.world_mut()
        .resource_mut::<CGCampaignState>()
        .mission_index = 1;
    transition(&mut app, GameState::BossIntro);
    transition(&mut app, GameState::BossFight);
    let boss = only_entity::<CGBoss>(&mut app);
    {
        let mut state = app.world_mut().get_mut::<CGBoss>(boss).unwrap();
        state.health = 111.0;
        state.current_phase = 2;
    }
    let escort = app.world_mut().spawn(Friendly).id();

    transition(&mut app, GameState::Paused);
    resume(&mut app);

    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::BossFight
    );
    assert_eq!(only_entity::<Player>(&mut app), player);
    assert_eq!(only_entity::<CGBoss>(&mut app), boss);
    let state = app.world().get::<CGBoss>(boss).unwrap();
    assert_eq!(state.health, 111.0);
    assert_eq!(state.current_phase, 2);
    assert!(app.world().get_entity(escort).is_ok());
}

#[test]
fn explicit_cg_restart_still_reinitializes_the_mission() {
    let mut app = setup("caldari_gallente");
    transition(&mut app, GameState::Playing);
    let player = only_entity::<Player>(&mut app);
    let enemy = app.world_mut().spawn(Enemy).id();
    // Use the real projectile bundle, including physics and damage, so an
    // old shot cannot survive a restart and hit the next mission's enemies.
    let player_projectile = app
        .world_mut()
        .spawn(PlayerProjectileBundle::default())
        .id();
    app.world_mut()
        .resource_mut::<CGCampaignState>()
        .current_wave = 3;

    transition(&mut app, GameState::Paused);
    assert!(app.world().get_entity(player_projectile).is_ok());
    app.world_mut()
        .resource_mut::<PauseContext>()
        .request_restart();
    transition(&mut app, GameState::Playing);

    assert_ne!(only_entity::<Player>(&mut app), player);
    assert_eq!(app.world().resource::<CGCampaignState>().current_wave, 1);
    assert!(app.world().get_entity(enemy).is_err());
    assert!(app.world().get_entity(player_projectile).is_err());
    only_entity::<EnemyCarrier>(&mut app);
}

#[test]
fn quitting_a_paused_cg_boss_fight_removes_gameplay_entities() {
    let mut app = setup("caldari_gallente");
    transition(&mut app, GameState::Playing);
    let player = only_entity::<Player>(&mut app);
    app.world_mut()
        .resource_mut::<CGCampaignState>()
        .mission_index = 1;
    transition(&mut app, GameState::BossIntro);
    transition(&mut app, GameState::BossFight);
    let boss = only_entity::<CGBoss>(&mut app);
    let enemy = app.world_mut().spawn(Enemy).id();
    let player_projectile = app
        .world_mut()
        .spawn(PlayerProjectileBundle::default())
        .id();
    let enemy_projectile = app.world_mut().spawn(EnemyProjectileBundle::default()).id();
    let telegraph = app.world_mut().spawn(CGSpawnTelegraph { timer: 0.0 }).id();
    let old_entities = [
        player,
        boss,
        enemy,
        player_projectile,
        enemy_projectile,
        telegraph,
    ];

    transition(&mut app, GameState::Paused);
    for entity in old_entities {
        assert!(
            app.world().get_entity(entity).is_ok(),
            "pause preserves combat"
        );
    }
    transition(&mut app, GameState::MainMenu);
    for entity in old_entities {
        assert!(
            app.world().get_entity(entity).is_err(),
            "quit cleans the old run"
        );
    }
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, Or<(
                With<Player>,
                With<Enemy>,
                With<CGBoss>,
                With<PlayerProjectile>,
                With<EnemyProjectile>,
                With<CGSpawnTelegraph>,
            )>>()
            .iter(app.world())
            .count(),
        0,
        "no CG combat entities remain behind the main menu"
    );
}

#[test]
fn abyssal_resume_keeps_room_progress_and_remaining_time() {
    let mut app = setup("abyssal_depths");
    transition(&mut app, GameState::Playing);
    {
        let mut state = app.world_mut().resource_mut::<AbyssalState>();
        state.room = AbyssalRoom::Room2;
        state.time_remaining = 321.0;
        state.enemies_killed = 4;
    }

    transition(&mut app, GameState::Paused);
    assert!(app.world().resource::<AbyssalState>().active);
    resume(&mut app);

    let state = app.world().resource::<AbyssalState>();
    assert!(state.active);
    assert_eq!(state.room, AbyssalRoom::Room2);
    assert_eq!(state.time_remaining, 321.0);
    assert_eq!(state.enemies_killed, 4);
}

#[test]
fn last_stand_resume_preserves_the_titan_and_active_mode() {
    let mut app = setup("caldari_gallente");
    app.world_mut().resource_mut::<LastStandState>().active = true;
    transition(&mut app, GameState::Playing);
    let titan = only_entity::<LastStandTitan>(&mut app);

    transition(&mut app, GameState::Paused);
    assert!(app.world().resource::<LastStandState>().active);
    resume(&mut app);

    assert!(app.world().resource::<LastStandState>().active);
    assert_eq!(only_entity::<LastStandTitan>(&mut app), titan);
}

#[test]
fn last_stand_restart_keeps_the_mode_and_resets_run_progress() {
    let mut app = setup("caldari_gallente");
    app.world_mut().resource_mut::<LastStandState>().start();
    transition(&mut app, GameState::Playing);
    let titan = only_entity::<LastStandTitan>(&mut app);
    {
        let mut state = app.world_mut().resource_mut::<LastStandState>();
        state.evacuation_progress = 47.0;
        state.shield = 12.0;
    }

    transition(&mut app, GameState::Paused);
    app.world_mut()
        .resource_mut::<PauseContext>()
        .request_restart();
    transition(&mut app, GameState::Playing);

    let state = app.world().resource::<LastStandState>();
    assert!(state.active);
    assert_eq!(state.evacuation_progress, 0.0);
    assert_eq!(state.shield, LastStandState::default().shield);
    assert_ne!(only_entity::<LastStandTitan>(&mut app), titan);
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<Player>>()
            .iter(app.world())
            .count(),
        0,
        "Last Stand must not turn into the ordinary ship campaign"
    );
}

fn inventory_panel_count(app: &mut App) -> usize {
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .filter(|text| text.0 == "MODULES")
        .count()
}

#[test]
fn inventory_hud_survives_resume_and_is_cleaned_on_restart_and_quit() {
    let mut app = setup("caldari_gallente");
    app.init_resource::<rebellion::systems::touch_joystick::MobileMode>();
    app.add_plugins(rebellion::ui::hud::HudPlugin);
    app.world_mut()
        .resource_mut::<rebellion::core::ItchMode>()
        .enabled = false;
    transition(&mut app, GameState::Playing);
    assert_eq!(inventory_panel_count(&mut app), 1);

    transition(&mut app, GameState::Paused);
    resume(&mut app);
    assert_eq!(inventory_panel_count(&mut app), 1);

    transition(&mut app, GameState::Paused);
    app.world_mut()
        .resource_mut::<PauseContext>()
        .request_restart();
    transition(&mut app, GameState::Playing);
    assert_eq!(inventory_panel_count(&mut app), 1);

    transition(&mut app, GameState::Paused);
    transition(&mut app, GameState::MainMenu);
    assert_eq!(inventory_panel_count(&mut app), 0);
}
