//! Elder Fleet campaign integration tests
//!
//! Validates Elder Fleet-specific campaign systems:
//! enemy spawning, boss spawning, and mission progression.

use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::GameState;
use rebellion::entities::{BossData, BossState, Enemy};
use rebellion::games::elder_fleet::ef_campaign::ElderFleetCampaignState;
use rebellion::games::ActiveModule;

#[test]
fn elder_fleet_spawns_enemies_on_wave_start() {
    let mut app = build_headless_app();
    app.init_resource::<rebellion::games::ModuleRegistry>();
    app.init_resource::<ActiveModule>();
    app.add_plugins(rebellion::games::elder_fleet::ElderFleetPlugin);

    // Set Elder Fleet as active module with Minmatar faction
    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("elder_fleet");
        active.player_faction = Some("minmatar".to_string());
    }

    // Transition to Playing
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // OnEnter(Playing) + start_ef_mission

    // Run updates to let spawn_ef_wave trigger
    for _ in 0..5 {
        app.update();
    }

    let enemy_count = {
        let mut q = app.world_mut().query::<&Enemy>();
        q.iter(app.world()).count()
    };
    assert!(
        enemy_count > 0,
        "Elder Fleet should spawn enemies on wave start, got {}",
        enemy_count
    );

    let state = app.world().resource::<ElderFleetCampaignState>();
    assert!(
        state.current_wave >= 1,
        "First wave should spawn and advance current_wave, got {}",
        state.current_wave
    );
}

#[test]
fn elder_fleet_spawns_boss_after_waves() {
    let mut app = build_headless_app();
    app.init_resource::<rebellion::games::ModuleRegistry>();
    app.init_resource::<ActiveModule>();
    app.add_plugins(rebellion::games::elder_fleet::ElderFleetPlugin);

    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("elder_fleet");
        active.player_faction = Some("minmatar".to_string());
    }

    // Transition to Playing
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Set wave past mission limit (mission 0 has 3 waves)
    {
        let mut state = app.world_mut().resource_mut::<ElderFleetCampaignState>();
        state.current_wave = 4; // past 3 waves
        state.enemies_remaining = 0;
    }

    // Despawn any enemies so wave check passes
    let enemy_entities: Vec<Entity> = {
        let mut q = app.world_mut().query::<(Entity, &Enemy)>();
        q.iter(app.world()).map(|(e, _)| e).collect()
    };
    for entity in enemy_entities {
        if let Ok(ec) = app.world_mut().get_entity_mut(entity) {
            ec.despawn();
        }
    }

    // Run updates so update_ef_mission detects boss time
    for _ in 0..5 {
        app.update();
    }

    let state = app.world().resource::<ElderFleetCampaignState>();
    let game_state = app.world().resource::<State<GameState>>().get();
    assert!(
        state.boss_spawned || *game_state == GameState::BossIntro,
        "Elder Fleet should transition to BossIntro after waves, got state={:?} boss_spawned={}",
        game_state,
        state.boss_spawned
    );
}

#[test]
fn elder_fleet_amarr_campaign_spawns_minmatar_enemies() {
    let mut app = build_headless_app();
    app.init_resource::<rebellion::games::ModuleRegistry>();
    app.init_resource::<ActiveModule>();
    app.add_plugins(rebellion::games::elder_fleet::ElderFleetPlugin);

    // Set Elder Fleet with Amarr faction (enemies = Minmatar)
    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("elder_fleet");
        active.player_faction = Some("amarr".to_string());
    }

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    for _ in 0..5 {
        app.update();
    }

    let enemy_count = {
        let mut q = app.world_mut().query::<&Enemy>();
        q.iter(app.world()).count()
    };
    assert!(
        enemy_count > 0,
        "Amarr campaign should spawn Minmatar enemies, got {}",
        enemy_count
    );
}

#[test]
fn elder_fleet_boss_intro_transitions_to_boss_fight() {
    let mut app = build_headless_app();
    app.init_resource::<rebellion::games::ModuleRegistry>();
    app.init_resource::<ActiveModule>();
    app.add_plugins(rebellion::games::elder_fleet::ElderFleetPlugin);

    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("elder_fleet");
        active.player_faction = Some("minmatar".to_string());
    }

    // Transition to BossIntro
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::BossIntro);
    app.update(); // OnEnter(BossIntro) spawns boss

    // Verify boss spawned in Intro state
    {
        let mut q = app.world_mut().query::<(&BossData, &BossState)>();
        let (data, state) = q.get_single(app.world()).expect("boss should spawn");
        assert_eq!(*state, BossState::Intro, "Boss should start in Intro state");
        assert!(
            data.name.contains("Squadron Leader"),
            "Expected first mission boss, got {}",
            data.name
        );
    }

    // Run updates to let ef_boss_intro transition to BossFight
    for _ in 0..130 {
        app.update(); // ~2s at fixed timestep 1/60
    }

    let game_state = app.world().resource::<State<GameState>>().get();
    assert_eq!(
        *game_state,
        GameState::BossFight,
        "BossIntro should transition to BossFight after intro timer, got {:?}",
        game_state
    );

    // Verify boss is now in Battle state
    {
        let mut q = app.world_mut().query::<&BossState>();
        let state = q.get_single(app.world()).expect("boss should exist");
        assert_eq!(*state, BossState::Battle, "Boss should be in Battle state");
    }
}
