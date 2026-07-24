//! Campaign objective integration tests
//!
//! Validates mission completion logic, no-damage tracking, and bonus evaluation.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{
    BossType, CampaignState, DamageType, GameState, Mission, PlayerDamagedEvent,
};
use rebellion::gameplay::combat_outcomes::player_damage_outcomes;

/// A mission definition with no boss, used for testing no-boss completion paths.
fn test_no_boss_mission() -> Mission {
    Mission {
        id: "test_no_boss",
        name: "Test No Boss",
        description: "Mission without a boss for testing",
        primary_objective: "Survive all waves",
        bonus_objective: None,
        boss: BossType::None,
        enemy_waves: 2,
        souls_to_liberate: 0,
        timed_survival_seconds: 0.0,
        kill_count_target: 0,
    }
}

#[test]
fn player_damage_sets_no_damage_taken_false() {
    let mut app = build_headless_app();

    // CampaignState starts fresh with no_damage_taken = true
    let start = app.world().resource::<CampaignState>().no_damage_taken;
    assert!(start, "CampaignState should start with no_damage_taken=true");

    // Emit a player-damage event
    app.world_mut().send_event(PlayerDamagedEvent {
        damage: 10.0,
        damage_type: DamageType::Thermal,
        source_position: Vec2::ZERO,
        shield_damage: 5.0,
        armor_damage: 3.0,
        hull_damage: 2.0,
        destroyed: false,
    });

    // Run the system that processes damage events
    app.world_mut()
        .run_system_once(player_damage_outcomes)
        .expect("run player_damage_outcomes");

    let end = app.world().resource::<CampaignState>().no_damage_taken;
    assert!(
        !end,
        "no_damage_taken should flip to false after PlayerDamagedEvent"
    );
}

#[test]
fn no_boss_mission_completes_without_boss() {
    let mut app = build_headless_app();

    // Set up a mission with BossType::None and advance past all waves
    {
        let mut campaign = app.world_mut().resource_mut::<CampaignState>();
        campaign.start_mission();
        campaign.current_wave = 3; // past enemy_waves (2)
    }

    let mission = test_no_boss_mission();
    let mut campaign = app.world_mut().resource_mut::<CampaignState>();
    let result = campaign.evaluate_post_wave(&mission);

    assert_eq!(
        result,
        Some(GameState::StageComplete),
        "No-boss mission should transition to StageComplete after all waves cleared"
    );
    assert!(
        campaign.primary_complete,
        "primary_complete should be set for no-boss mission"
    );
}

#[test]
fn bonus_complete_when_no_damage_taken() {
    let mut app = build_headless_app();

    let mission = test_no_boss_mission();

    // Case 1: no_damage_taken = true → bonus_complete should be true
    {
        let mut campaign = app.world_mut().resource_mut::<CampaignState>();
        campaign.start_mission();
        campaign.current_wave = 3; // past all waves
        campaign.no_damage_taken = true;
        campaign.evaluate_post_wave(&mission);
    }
    {
        let campaign = app.world().resource::<CampaignState>();
        assert!(
            campaign.bonus_complete,
            "bonus_complete should be true when no_damage_taken is true"
        );
    }

    // Case 2: no_damage_taken = false, souls met → bonus_complete should be true
    {
        let mut campaign = app.world_mut().resource_mut::<CampaignState>();
        campaign.start_mission();
        campaign.current_wave = 3;
        campaign.no_damage_taken = false;
        campaign.mission_souls = 10;
        let mut mission2 = test_no_boss_mission();
        mission2.souls_to_liberate = 5; // souls met
        campaign.evaluate_post_wave(&mission2);
        assert!(
            campaign.bonus_complete,
            "bonus_complete should be true when souls_to_liberate threshold is met"
        );
    }

    // Case 3: no_damage_taken = false, souls NOT met → bonus_complete should be false
    {
        let mut campaign = app.world_mut().resource_mut::<CampaignState>();
        campaign.start_mission();
        campaign.current_wave = 3;
        campaign.no_damage_taken = false;
        campaign.mission_souls = 2;
        let mut mission3 = test_no_boss_mission();
        mission3.souls_to_liberate = 5; // souls NOT met
        campaign.evaluate_post_wave(&mission3);
        assert!(
            !campaign.bonus_complete,
            "bonus_complete should be false when neither no_damage_taken nor souls threshold met"
        );
    }
}
