//! Projectile end-to-end integration test
//!
//! Verifies that a player projectile spawns, travels, collides with an enemy,
//! and reduces enemy health in headless mode.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{
    AmmoType, GameState, PlayerFireEvent, SpawnEnemyEvent, SpawnPattern, WeaponType,
};
use rebellion::entities::{Enemy, EnemyStats};
use rebellion::simulation::state_hash::SimStateHash;

/// Spawn an enemy directly above the player for a guaranteed collision.
fn send_spawn_event(mut events: EventWriter<SpawnEnemyEvent>) {
    events.send(SpawnEnemyEvent {
        enemy_type: "597".to_string(), // Punisher (Amarr frigate)
        position: Vec2::new(0.0, -200.0),
        spawn_pattern: SpawnPattern::Single,
    });
}

/// Fire a player projectile straight upward toward the enemy.
fn send_fire_event(mut events: EventWriter<PlayerFireEvent>) {
    events.send(PlayerFireEvent {
        position: Vec2::new(0.0, -250.0),
        direction: Vec2::new(0.0, 1.0),
        weapon_type: WeaponType::Laser,
        bullet_color: Color::srgb(1.0, 0.2, 0.2),
        damage: 10.0,
        burst_count: 1,
        spread_angle: 0.0,
        ammo_type: AmmoType::default(),
        crit_chance_override: None,
        crit_mult_override: None,
        pierce: 0,
        homing: 0.0,
        burn_dps: 0.0,
        chain_targets: 0,
    });
}

#[test]
fn projectile_hits_enemy_and_reduces_health() {
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

    // Identify our test enemy by its negative-y position (wave enemies spawn
    // at the top of the screen, positive y). Capture initial health.
    let initial_health = {
        let mut q = app.world_mut().query::<(&Enemy, &EnemyStats, &Transform)>();
        let (_, stats, _) = q
            .iter(app.world())
            .find(|(_, _, t)| t.translation.y < 0.0)
            .expect("test enemy spawned below screen center");
        stats.health
    };

    // Capture state hash before projectile fire
    let hash_before = app.world().resource::<SimStateHash>().0;

    // Fire projectile from player position upward
    app.world_mut()
        .run_system_once(send_fire_event)
        .expect("send fire event");

    // Run ticks until projectile collides with enemy (~10 frames at 600 px/s)
    for _ in 0..30 {
        app.update();
    }

    // Verify our test enemy's health decreased
    let health_after = {
        let mut q = app.world_mut().query::<(&Enemy, &EnemyStats, &Transform)>();
        let (_, stats, _) = q
            .iter(app.world())
            .find(|(_, _, t)| t.translation.y < 0.0)
            .expect("test enemy still exists");
        stats.health
    };

    assert!(
        health_after < initial_health,
        "enemy should take damage: {health_after} >= {initial_health}"
    );

    // Verify state hash changed after collision
    let hash_after = app.world().resource::<SimStateHash>().0;
    assert_ne!(
        hash_before, hash_after,
        "state hash should change after projectile collision"
    );
}
