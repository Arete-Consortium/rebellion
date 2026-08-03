//! Triglavian boss phase integration tests
//!
//! Validates boss phase transitions, enrage mechanics, and projectile spawning
//! during BossFight state.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::GameState;
use rebellion::entities::boss::{
    Boss, BossAttack, BossData, BossMovement, BossState, MovementPattern,
};
use rebellion::entities::EnemyProjectile;
use rebellion::games::ActiveModule;

/// Spawns a player entity at the origin.
fn spawn_test_player(world: &mut World) {
    world.spawn((
        rebellion::entities::Player,
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// Helper to build a Leshak boss entity with configurable health.
fn spawn_test_boss(
    commands: &mut Commands,
    health: f32,
    fire_timer: f32,
    fire_rate: f32,
) -> Entity {
    commands
        .spawn((
            Boss,
            BossData {
                id: 1,
                stage: 1,
                name: "Test Leshak".to_string(),
                title: "Test Mission".to_string(),
                ship_class: "Battleship".to_string(),
                type_id: rebellion::games::triglavian_invasion::ships::triglavian::LESHAK,
                max_health: 1000.0,
                health,
                current_phase: 1,
                total_phases: 3,
                score_value: 2000,
                liberation_value: 10,
                stationary: false,
                dialogue_intro: "Test intro".to_string(),
                dialogue_defeat: "Test defeat".to_string(),
                is_enraged: false,
                enrage_threshold: 0.2,
            },
            BossState::Battle,
            BossMovement {
                pattern: MovementPattern::Sweep,
                timer: 0.0,
                speed: 100.0,
            },
            BossAttack {
                pattern: "steady_beam".to_string(),
                fire_timer,
                fire_rate,
                burst_count: 3,
                burst_remaining: 0,
            },
            Transform::from_xyz(0.0, 200.0, 0.0),
        ))
        .id()
}

#[test]
fn trig_boss_phase_transition_and_enrage() {
    let mut app = build_headless_app();

    // Mark Triglavian module active so generic campaign systems don't interfere
    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("triglavian_invasion");

    // Transition to BossFight so the system can run
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::BossFight);
    app.update();

    spawn_test_player(app.world_mut());

    let boss = spawn_test_boss(&mut app.world_mut().commands(), 1000.0, 0.0, 10.0);
    app.update(); // flush commands

    // -----------------------------------------------------------------
    // Phase 1 — full health, should stay phase 1
    // -----------------------------------------------------------------
    app.world_mut()
        .run_system_once(rebellion::games::triglavian_invasion::campaign::update_trig_boss)
        .expect("update_trig_boss should run");

    let data = app.world().get::<BossData>(boss).unwrap();
    assert_eq!(data.current_phase, 1);
    assert!(!data.is_enraged);

    // -----------------------------------------------------------------
    // Phase 1 → 2 at ≤66.7% health
    // -----------------------------------------------------------------
    app.world_mut()
        .entity_mut(boss)
        .get_mut::<BossData>()
        .unwrap()
        .health = 600.0;

    app.world_mut()
        .run_system_once(rebellion::games::triglavian_invasion::campaign::update_trig_boss)
        .expect("update_trig_boss should run");

    let data = app.world().get::<BossData>(boss).unwrap();
    assert_eq!(
        data.current_phase, 2,
        "should transition to phase 2 at 60% health"
    );

    let movement = app.world().get::<BossMovement>(boss).unwrap();
    assert!(
        (movement.speed - 120.0).abs() < 0.01,
        "speed should increase by 1.2× in phase 2, got {}",
        movement.speed
    );

    // -----------------------------------------------------------------
    // Phase 2 → 3 at ≤33.3% health
    // -----------------------------------------------------------------
    app.world_mut()
        .entity_mut(boss)
        .get_mut::<BossData>()
        .unwrap()
        .health = 300.0;

    app.world_mut()
        .run_system_once(rebellion::games::triglavian_invasion::campaign::update_trig_boss)
        .expect("update_trig_boss should run");

    let data = app.world().get::<BossData>(boss).unwrap();
    assert_eq!(
        data.current_phase, 3,
        "should transition to phase 3 at 30% health"
    );

    let movement = app.world().get::<BossMovement>(boss).unwrap();
    assert!(
        (movement.speed - 144.0).abs() < 0.01,
        "speed should increase by 1.2× again in phase 3, got {}",
        movement.speed
    );

    // -----------------------------------------------------------------
    // Enrage at ≤20% health
    // -----------------------------------------------------------------
    app.world_mut()
        .entity_mut(boss)
        .get_mut::<BossData>()
        .unwrap()
        .health = 150.0;

    app.world_mut()
        .run_system_once(rebellion::games::triglavian_invasion::campaign::update_trig_boss)
        .expect("update_trig_boss should run");

    let data = app.world().get::<BossData>(boss).unwrap();
    assert!(data.is_enraged, "should enrage at 15% health");

    let movement = app.world().get::<BossMovement>(boss).unwrap();
    assert!(
        (movement.speed - 216.0).abs() < 0.1,
        "speed should increase by 1.5× when enraged, got {}",
        movement.speed
    );
}

#[test]
fn trig_boss_spawns_projectiles_during_boss_fight() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("triglavian_invasion");

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::BossFight);
    app.update();

    spawn_test_player(app.world_mut());

    let mut q = app.world_mut().query::<&EnemyProjectile>();
    let before = q.iter(app.world()).count();

    // Spawn boss with fire_timer just below fire_rate so one tick triggers firing
    let _boss = spawn_test_boss(&mut app.world_mut().commands(), 1000.0, 0.79, 0.8);
    app.update(); // flush commands

    // Fire timer (0.79) + dt (~0.0167) ≥ 0.8 → should fire
    app.world_mut()
        .run_system_once(rebellion::games::triglavian_invasion::campaign::update_trig_boss)
        .expect("update_trig_boss should run");

    let mut q = app.world_mut().query::<&EnemyProjectile>();
    let after = q.iter(app.world()).count();
    let spawned = after - before;

    assert!(
        spawned > 0,
        "boss should spawn projectiles during BossFight: before={before}, after={after}"
    );

    // Leshak (non-enraged, phase 1) fires a 5-bullet spread
    assert_eq!(
        spawned, 5,
        "Leshak phase 1 should fire a 5-bullet spread, got {spawned}"
    );
}

#[test]
fn trig_boss_does_not_fire_when_not_in_battle_state() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("triglavian_invasion");

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::BossFight);
    app.update();

    spawn_test_player(app.world_mut());

    let mut q = app.world_mut().query::<&EnemyProjectile>();
    let before = q.iter(app.world()).count();

    // Spawn boss in Intro state (not Battle)
    let boss = spawn_test_boss(&mut app.world_mut().commands(), 1000.0, 0.79, 0.8);
    app.update(); // flush commands so entity exists

    app.world_mut().entity_mut(boss).insert(BossState::Intro);
    app.update(); // flush the insert

    app.world_mut()
        .run_system_once(rebellion::games::triglavian_invasion::campaign::update_trig_boss)
        .expect("update_trig_boss should run");

    let mut q = app.world_mut().query::<&EnemyProjectile>();
    let after = q.iter(app.world()).count();
    assert_eq!(
        after, before,
        "boss in Intro state should not fire projectiles"
    );
}
