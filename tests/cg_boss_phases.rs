//! Caldari-Gallente boss phase integration tests
//!
//! Validates CG boss phase transitions and projectile spawning during BossFight.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{Difficulty, GameState};
// Boss imports not needed for CG-specific tests
use rebellion::entities::EnemyProjectile;
use rebellion::games::caldari_gallente::cg_campaign::{CGBoss, CGBossAttack, CGBossMovement};
use rebellion::games::caldari_gallente::campaign::CGBossType;

/// Spawn a player entity at the given position.
fn spawn_test_player(world: &mut World, pos: Vec2) {
    world.spawn((
        rebellion::entities::Player,
        Transform::from_xyz(pos.x, pos.y, 0.0),
    ));
}

/// Spawn a CG FleetCommander boss with configurable health and fire timer.
fn spawn_test_cg_boss(commands: &mut Commands, health: f32, fire_timer: f32, fire_rate: f32) -> Entity {
    let max_health = CGBossType::FleetCommander.health();
    commands.spawn((
        CGBoss {
            boss_type: CGBossType::FleetCommander,
            health,
            max_health,
            current_phase: 1,
            total_phases: CGBossType::FleetCommander.phases(),
        },
        CGBossMovement { timer: 0.0, speed: 100.0 },
        CGBossAttack { fire_timer, fire_rate },
        rebellion::entities::EnemyStats {
            type_id: 0,
            name: "Test CG Boss".to_string(),
            health,
            max_health,
            speed: 80.0,
            score_value: 1000,
            is_boss: true,
            liberation_value: 10,
        },
        Transform::from_xyz(0.0, 200.0, 0.0),
    )).id()
}

#[test]
fn cg_boss_phase_transition() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::BossFight);
    app.update();

    spawn_test_player(app.world_mut(), Vec2::new(0.0, 0.0));

    // Spawn boss at full health with high fire rate so it doesn't fire.
    // Use FleetCommander.max_health (1800) so phase thresholds align.
    let max_hp = CGBossType::FleetCommander.health();
    let boss = spawn_test_cg_boss(&mut app.world_mut().commands(), max_hp, 0.0, 10.0);
    app.update(); // flush commands

    // Phase 1 — full health, no transition
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::update_cg_boss)
        .expect("update_cg_boss should run");

    let boss_data = app.world().get::<CGBoss>(boss).unwrap();
    assert_eq!(boss_data.current_phase, 1);

    // Reduce health to 60% (below 66.7% threshold for phase 1→2)
    {
        let mut e = app.world_mut().entity_mut(boss);
        let mut cg = e.get_mut::<CGBoss>().unwrap();
        cg.health = max_hp * 0.6;
        let mut stats = e.get_mut::<rebellion::entities::EnemyStats>().unwrap();
        stats.health = max_hp * 0.6;
    }

    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::update_cg_boss)
        .expect("update_cg_boss should run");

    let boss_data = app.world().get::<CGBoss>(boss).unwrap();
    assert_eq!(boss_data.current_phase, 2, "should transition to phase 2 at 60% health");

    let movement = app.world().get::<CGBossMovement>(boss).unwrap();
    assert!(
        (movement.speed - 120.0).abs() < 0.01,
        "speed should increase by 1.2× in phase 2, got {}",
        movement.speed
    );

    let attack = app.world().get::<CGBossAttack>(boss).unwrap();
    assert!(
        (attack.fire_rate - 8.0).abs() < 0.01,
        "fire_rate should decrease by 0.8× in phase 2, got {}",
        attack.fire_rate
    );

    // Reduce health to 30% (below 33.3% threshold for phase 2→3)
    {
        let mut e = app.world_mut().entity_mut(boss);
        let mut cg = e.get_mut::<CGBoss>().unwrap();
        cg.health = max_hp * 0.3;
        let mut stats = e.get_mut::<rebellion::entities::EnemyStats>().unwrap();
        stats.health = max_hp * 0.3;
    }

    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::update_cg_boss)
        .expect("update_cg_boss should run");

    let boss_data = app.world().get::<CGBoss>(boss).unwrap();
    assert_eq!(boss_data.current_phase, 3, "should transition to phase 3 at 30% health");

    let movement = app.world().get::<CGBossMovement>(boss).unwrap();
    assert!(
        (movement.speed - 144.0).abs() < 0.01,
        "speed should increase by 1.2× again in phase 3, got {}",
        movement.speed
    );
}

#[test]
fn cg_boss_spawns_projectiles_during_boss_fight() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::BossFight);
    app.update();

    spawn_test_player(app.world_mut(), Vec2::new(0.0, 0.0));

    let before = {
        let mut q = app.world_mut().query::<(Entity, &EnemyProjectile)>();
        q.iter(app.world()).count()
    };

    // Spawn boss with fire_timer near fire_rate
    let _boss = spawn_test_cg_boss(&mut app.world_mut().commands(), CGBossType::FleetCommander.health(), 0.79, 0.8);
    app.update(); // flush commands

    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::update_cg_boss)
        .expect("update_cg_boss should run");

    let after = {
        let mut q = app.world_mut().query::<(Entity, &EnemyProjectile)>();
        q.iter(app.world()).count()
    };

    assert!(
        after > before,
        "CG boss should spawn projectiles during BossFight: before={before}, after={after}"
    );
}

#[test]
fn cg_boss_damage_scales_with_difficulty() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::BossFight);
    app.update();

    spawn_test_player(app.world_mut(), Vec2::new(0.0, 0.0));

    // Insert Carebear difficulty
    app.world_mut().insert_resource(Difficulty::Carebear);

    // Spawn at full health so boss stays in phase 1 for predictable damage
    let _boss = spawn_test_cg_boss(&mut app.world_mut().commands(), CGBossType::FleetCommander.health(), 0.79, 0.8);
    app.update(); // flush commands

    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::update_cg_boss)
        .expect("update_cg_boss should run");

    // Query spawned projectiles for damage
    let mut q = app.world_mut().query::<&rebellion::entities::ProjectileDamage>();
    let damages: Vec<f32> = q.iter(app.world()).map(|d| d.damage).collect();

    assert!(
        !damages.is_empty(),
        "projectiles should spawn with scaled damage"
    );

    // Carebear enemy_damage_mult = 0.5
    // Base damage for phase 1 = 20.0 + (1 * 5) = 25.0
    // Scaled = 25.0 * 0.5 = 12.5
    for d in &damages {
        assert!(
            (*d - 12.5).abs() < 0.1,
            "Carebear damage should be ~12.5, got {}",
            d
        );
    }
}
