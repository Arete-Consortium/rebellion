//! Mission 1 (Orbital Skirmish) encounter integration tests
//!
//! Validates that Mission 1 spawns enemies with correct scaling,
//! varied formations, and deterministic state hashes.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{Faction, GameSession, GameState};
use rebellion::entities::{enemy::EnemyWeapon, Enemy, EnemyStats};
use rebellion::games::caldari_gallente::campaign::CGCampaignState;
use rebellion::games::caldari_gallente::{CaldariGallenteShips, VerticalSliceMode};
use rebellion::games::ActiveModule;
use rebellion::simulation::state_hash::SimStateHash;

/// Helper: configure the app for Caldari-Gallente Mission 1 with Gallente player.
fn setup_cg_mission1(app: &mut App) {
    // Insert resources normally provided by CaldariGallentePlugin
    app.init_resource::<CGCampaignState>();
    app.init_resource::<CaldariGallenteShips>();
    app.init_resource::<VerticalSliceMode>();

    // Set active module
    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("caldari_gallente");
        active.set_faction("gallente", "caldari");
    }
    // Set session enemy faction
    {
        let mut session = app.world_mut().resource_mut::<GameSession>();
        session.enemy_faction = Faction::Caldari;
    }
    // Reset campaign to Mission 1
    {
        let mut cg = app.world_mut().resource_mut::<CGCampaignState>();
        *cg = CGCampaignState::default();
        cg.mission_index = 0;
    }
}

/// A visible carrier now precedes the first wave. This also exercises the
/// no-texture headless case: its animation clock must still finish.
fn finish_carrier_arrival(app: &mut App) {
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_wave)
        .unwrap();
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<Enemy>>()
            .iter(app.world())
            .count(),
        0,
        "enemy wave must wait for the carrier arrival"
    );
    for _ in 0..130 {
        app.update();
    }
    for carrier in app
        .world_mut()
        .query::<&rebellion::systems::spawning::EnemyCarrier>()
        .iter(app.world())
    {
        assert_eq!(carrier.warp_progress, 1.0);
    }
}

#[test]
fn mission1_spawns_scaled_enemies() {
    let mut app = build_headless_app();
    setup_cg_mission1(&mut app);

    // Transition to Playing so OnEnter(Playing) systems run
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // process transition + start_cg_mission

    finish_carrier_arrival(&mut app);

    // Manually invoke spawn_cg_wave since GameModulesPlugin isn't in headless
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_wave)
        .expect("spawn_cg_wave should run");
    app.update(); // flush commands

    // Query spawned enemies
    let enemies: Vec<(EnemyStats, EnemyWeapon)> = {
        let mut q = app.world_mut().query::<(&EnemyStats, &EnemyWeapon)>();
        q.iter(app.world())
            .map(|(s, w)| (s.clone(), w.clone()))
            .collect()
    };

    assert!(!enemies.is_empty(), "Mission 1 wave 1 should spawn enemies");

    // Mission 1 scaling: 2x HP, 60% damage
    for (stats, weapon) in &enemies {
        assert!(
            stats.health > 40.0,
            "Mission 1 enemy HP should be scaled up (got {} for {})",
            stats.health,
            stats.name
        );
        assert!(
            weapon.damage <= 14.0,
            "Mission 1 enemy damage should be scaled down (got {} for {})",
            weapon.damage,
            stats.name
        );
    }
}

#[test]
fn mission1_wave_spawn_is_deterministic() {
    let mut app1 = build_headless_app();
    setup_cg_mission1(&mut app1);
    app1.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app1.update();
    for _ in 0..10 {
        app1.update();
    }
    let hash1 = app1.world().resource::<SimStateHash>().0;

    let mut app2 = build_headless_app();
    setup_cg_mission1(&mut app2);
    app2.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app2.update();
    for _ in 0..10 {
        app2.update();
    }
    let hash2 = app2.world().resource::<SimStateHash>().0;

    assert_eq!(
        hash1, hash2,
        "Mission 1 wave spawn should produce deterministic state hash"
    );
}

#[test]
fn mission1_enemies_have_varied_positions() {
    let mut app = build_headless_app();
    setup_cg_mission1(&mut app);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // start_cg_mission runs

    finish_carrier_arrival(&mut app);

    // Manually invoke spawn
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_wave)
        .expect("spawn_cg_wave should run");
    app.update(); // flush commands

    let positions: Vec<Vec2> = {
        let mut q = app.world_mut().query::<(&Enemy, &Transform)>();
        q.iter(app.world())
            .map(|(_, t)| t.translation.truncate())
            .collect()
    };

    assert!(
        positions.len() >= 3,
        "Mission 1 wave 1 should spawn at least 3 enemies"
    );

    // Wave 1 is deliberately a horizontal line. Its small random Y jitter can
    // legitimately be almost equal across a wave, so verify the guaranteed
    // horizontal spread instead of making the check depend on lucky RNG.
    let min_x = positions.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = positions
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (max_x - min_x) > 100.0,
        "Mission 1 line must spread across the playfield, got min={min_x} max={max_x}"
    );
}

#[test]
fn mission2_has_moderate_scaling() {
    let mut app = build_headless_app();
    setup_cg_mission1(&mut app);

    // Start at Mission 2
    {
        let mut cg = app.world_mut().resource_mut::<CGCampaignState>();
        cg.mission_index = 1;
    }

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // start_cg_mission runs

    finish_carrier_arrival(&mut app);

    // Manually invoke spawn
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_wave)
        .expect("spawn_cg_wave should run");
    app.update(); // flush commands

    let enemies: Vec<(EnemyStats, EnemyWeapon)> = {
        let mut q = app.world_mut().query::<(&EnemyStats, &EnemyWeapon)>();
        q.iter(app.world())
            .map(|(s, w)| (s.clone(), w.clone()))
            .collect()
    };

    assert!(!enemies.is_empty(), "Mission 2 wave 1 should spawn enemies");

    // Mission 2 scaling: 1.5x HP, 80% damage
    for (stats, weapon) in &enemies {
        assert!(
            stats.health > 30.0 && stats.health < 120.0,
            "Mission 2 HP should be moderate (got {} for {})",
            stats.health,
            stats.name
        );
        assert!(
            weapon.damage <= 18.0,
            "Mission 2 damage should be slightly reduced (got {} for {})",
            weapon.damage,
            stats.name
        );
    }
}
