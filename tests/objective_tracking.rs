//! Objective tracking integration tests
//!
//! Validates that mission objectives (no-damage bonus, bonus completion) are
//! tracked correctly by gameplay systems.

use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{CampaignState, DamageType, GameState, PlayerDamagedEvent};

#[test]
fn player_damage_sets_no_damage_taken_false() {
    let mut app = build_headless_app();

    // Transition to Playing so FixedUpdate systems run
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // process state transition

    // Verify we're in Playing state
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::Playing
    );

    // Set campaign state: no_damage_taken should start true
    {
        let mut campaign = app.world_mut().resource_mut::<CampaignState>();
        campaign.no_damage_taken = true;
    }

    // Send a player damaged event
    app.world_mut()
        .resource_mut::<Events<PlayerDamagedEvent>>()
        .send(PlayerDamagedEvent {
            damage: 10.0,
            damage_type: DamageType::Thermal,
            source_position: Vec2::ZERO,
            shield_damage: 0.0,
            armor_damage: 0.0,
            hull_damage: 10.0,
            destroyed: false,
        });

    // Run one update: First schedule swaps event buffers, FixedUpdate runs
    // player_damage_outcomes which consumes the event
    app.update();

    // Assert no_damage_taken was flipped to false
    let campaign = app.world().resource::<CampaignState>();
    assert!(
        !campaign.no_damage_taken,
        "no_damage_taken should be false after player takes damage"
    );
}

#[test]
fn start_mission_resets_no_damage_taken_true() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Simulate damage during a mission
    {
        let mut campaign = app.world_mut().resource_mut::<CampaignState>();
        campaign.no_damage_taken = false;
    }

    // Start a new mission — should reset to true
    {
        let mut campaign = app.world_mut().resource_mut::<CampaignState>();
        campaign.start_mission();
    }

    let campaign = app.world().resource::<CampaignState>();
    assert!(
        campaign.no_damage_taken,
        "no_damage_taken should reset to true on mission start"
    );
}

#[test]
fn enemy_death_increments_enemies_killed() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Start mission to set in_mission = true
    {
        let mut campaign = app.world_mut().resource_mut::<CampaignState>();
        campaign.start_mission();
    }

    // Verify starting count is 0
    {
        let campaign = app.world().resource::<CampaignState>();
        assert_eq!(campaign.enemies_killed, 0, "enemies_killed should start at 0");
    }

    // Player entity is spawned automatically by PlayerPlugin on OnEnter(Playing)

    // Send an EnemyDestroyedEvent
    app.world_mut()
        .resource_mut::<Events<rebellion::core::EnemyDestroyedEvent>>()
        .send(rebellion::core::EnemyDestroyedEvent {
            enemy: Entity::PLACEHOLDER,
            position: Vec2::ZERO,
            enemy_type: "Test Enemy".to_string(),
            score_value: 100,
            was_boss: false,
            liberation_value: 1,
            type_id: 587,
        });

    // Run one update to process the event
    app.update();

    let campaign = app.world().resource::<CampaignState>();
    assert_eq!(
        campaign.enemies_killed, 1,
        "enemies_killed should increment when enemy destroyed event is processed"
    );
}
