//! Scoring chain integration test
//!
//! Verifies that ScoreSystem and SaltMinerSystem update correctly when an enemy
//! is destroyed in headless mode.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{
    AmmoType, GameState, PlayerFireEvent, SaltMinerSystem, ScoreSystem, SpawnEnemyEvent,
    SpawnPattern, WeaponType,
};
use rebellion::entities::{Enemy, EnemyStats};
use rebellion::simulation::state_hash::SimStateHash;

/// Spawn a low-health enemy directly above the player.
fn send_spawn_event(mut events: EventWriter<SpawnEnemyEvent>) {
    // Executioner (Amarr frigate) — 25 HP, easy to one-shot.
    events.send(SpawnEnemyEvent {
        enemy_type: "589".to_string(),
        position: Vec2::new(0.0, -200.0),
        spawn_pattern: SpawnPattern::Single,
    });
}

/// Fire a high-damage projectile straight upward to kill the enemy in one hit.
fn send_fire_event(mut events: EventWriter<PlayerFireEvent>) {
    events.send(PlayerFireEvent {
        position: Vec2::new(0.0, -250.0),
        direction: Vec2::new(0.0, 1.0),
        weapon_type: WeaponType::Laser,
        bullet_color: Color::srgb(1.0, 0.2, 0.2),
        damage: 30.0,
        burst_count: 1,
        spread_angle: 0.0,
        ammo_type: AmmoType::default(),
        crit_chance_override: Some(0.0), // Guarantee no crit so damage is predictable
        crit_mult_override: None,
        pierce: 0,
        homing: 0.0,
        burn_dps: 0.0,
        chain_targets: 0,
    });
}

#[test]
fn scoring_systems_update_on_enemy_kill() {
    let mut app = build_headless_app();

    // Transition to Playing so gameplay systems are active
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Spawn enemy
    app.world_mut()
        .run_system_once(send_spawn_event)
        .expect("send spawn event");

    // Wait for event propagation and spatial grid registration
    for _ in 0..5 {
        app.update();
    }

    // Identify our test enemy by its negative-y position
    let initial_health = {
        let mut q = app.world_mut().query::<(&Enemy, &EnemyStats, &Transform)>();
        let (_, stats, _) = q
            .iter(app.world())
            .find(|(_, _, t)| t.translation.y < 0.0)
            .expect("test enemy spawned below screen center");
        stats.health
    };
    assert_eq!(initial_health, 25.0, "Executioner should have 25 HP");

    // Capture baseline scoring resources
    let score_before = app.world().resource::<ScoreSystem>().clone();
    let salt_before = app.world().resource::<SaltMinerSystem>().meter;
    let hash_before = app.world().resource::<SimStateHash>().0;

    assert_eq!(score_before.chain, 0, "chain should start at 0");
    assert_eq!(
        score_before.multiplier, 1.0,
        "multiplier should start at 1.0"
    );
    assert_eq!(salt_before, 0.0, "salt miner meter should start at 0");

    // Fire projectile from player position upward
    app.world_mut()
        .run_system_once(send_fire_event)
        .expect("send fire event");

    // Run ticks until projectile collides, enemy dies, and score updates
    for _ in 0..30 {
        app.update();
    }

    // Verify scoring resources updated
    let score_after = app.world().resource::<ScoreSystem>();
    let salt_after = app.world().resource::<SaltMinerSystem>().meter;

    assert!(
        score_after.chain_timer > 0.0,
        "chain_timer should be positive after kill, got {}",
        score_after.chain_timer
    );
    assert!(
        score_after.multiplier > 1.0,
        "multiplier should increase after kill, got {}",
        score_after.multiplier
    );
    assert!(
        score_after.score > score_before.score,
        "score should increase after kill, got {} vs {}",
        score_after.score,
        score_before.score
    );
    assert!(
        salt_after > salt_before,
        "salt miner meter should increase after kill, got {} vs {}",
        salt_after,
        salt_before
    );

    // Verify state hash changed
    let hash_after = app.world().resource::<SimStateHash>().0;
    assert_ne!(
        hash_before, hash_after,
        "state hash should change after enemy destruction"
    );
}
