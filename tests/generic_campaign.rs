//! Generic campaign (Elder Fleet) integration tests
//!
//! Validates that the shared campaign system in `systems/campaign.rs` correctly
//! spawns waves, handles boss transitions, and advances mission state.

use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{CampaignState, GameState};
use rebellion::entities::Enemy;

#[test]
fn generic_campaign_spawns_enemies_on_wave_start() {
    let mut app = build_headless_app();

    // Transition to Playing so OnEnter(Playing) systems run
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // process state transition + OnEnter(Playing)

    // FixedUpdate may need an extra tick to trigger after state transition
    for _ in 0..5 {
        app.update();
    }

    // Verify Playing state and mission started
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::Playing
    );
    let campaign = app.world().resource::<CampaignState>();
    assert!(campaign.in_mission, "Campaign should be in a mission");
    assert_eq!(campaign.current_wave, 2, "First wave should spawn and advance to 2");

    // Count spawned enemies
    let enemy_count = {
        let mut q = app.world_mut().query::<&Enemy>();
        q.iter(app.world()).count()
    };
    assert!(
        enemy_count > 0,
        "Generic campaign should spawn enemies on wave start, got {}",
        enemy_count
    );
}

#[test]
fn generic_campaign_spawns_boss_after_waves() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // transition + start_mission

    // First mission has 3 waves; advance current_wave past enemy_waves
    {
        let mut campaign = app.world_mut().resource_mut::<CampaignState>();
        campaign.current_wave = 5; // past the 3 waves of mission 1
    }

    // Despawn any enemies from the first wave so spawn_next_wave proceeds
    let enemy_entities: Vec<Entity> = {
        let mut q = app.world_mut().query::<(Entity, &Enemy)>();
        q.iter(app.world()).map(|(e, _)| e).collect()
    };
    for entity in enemy_entities {
        if let Ok(ec) = app.world_mut().get_entity_mut(entity) {
            ec.despawn();
        }
    }

    // Run updates to let FixedUpdate check for boss time and process state transition
    for _ in 0..5 {
        app.update();
    }

    // Boss should be spawned or BossIntro should be queued
    let campaign = app.world().resource::<CampaignState>();
    let state = app.world().resource::<State<GameState>>().get();
    assert!(
        campaign.boss_spawned || *state == GameState::BossIntro || *state == GameState::BossFight,
        "Campaign should spawn boss or transition to BossIntro/BossFight after waves, got state={:?} boss_spawned={}",
        state,
        campaign.boss_spawned
    );
}
