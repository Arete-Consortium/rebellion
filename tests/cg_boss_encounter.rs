//! CG Boss Encounter Deterministic Tests
//!
//! Validates that Mission 2 (Patrol Commander) and Mission 3 (Fleet Commander)
//! bosses spawn with tuned HP, transition phases at expected thresholds,
//! and produce deterministic state hashes.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{Difficulty, Faction, GameSession, GameState};
use rebellion::games::caldari_gallente::campaign::{CGCampaignState, CGBossType};
use rebellion::games::caldari_gallente::cg_campaign::{CGBoss, CGBossMovement};
use rebellion::simulation::state_hash::SimStateHash;

/// Configure app for Caldari-Gallente boss encounter.
fn setup_cg_boss(app: &mut App, mission_index: usize) {
    app.init_resource::<CGCampaignState>();
    {
        let mut cg = app.world_mut().resource_mut::<CGCampaignState>();
        cg.mission_index = mission_index;
        cg.in_mission = true;
    }
    {
        let mut session = app.world_mut().resource_mut::<GameSession>();
        session.enemy_faction = Faction::Caldari;
    }
    {
        let mut next = app.world_mut().resource_mut::<NextState<GameState>>();
        next.set(GameState::BossIntro);
    }
}

#[test]
fn patrol_commander_spawns_with_correct_hp() {
    let mut app = build_headless_app();
    setup_cg_boss(&mut app, 1); // Mission 2 index

    // Insert Normal difficulty
    app.world_mut().insert_resource(Difficulty::Newbro);

    app.update(); // flush state transition
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_boss)
        .expect("spawn_cg_boss should run");
    app.update(); // flush commands

    let mut q = app.world_mut().query::<&rebellion::entities::EnemyStats>();
    let stats: Vec<&rebellion::entities::EnemyStats> = q.iter(app.world()).collect();
    let boss_stats = stats.iter().find(|s| s.is_boss).expect("boss should spawn");

    let expected = CGBossType::PatrolCommander.health() * Difficulty::Newbro.enemy_health_mult();
    assert!(
        (boss_stats.max_health - expected).abs() < 0.1,
        "PatrolCommander HP should be {expected}, got {}",
        boss_stats.max_health
    );
}

#[test]
fn patrol_commander_phase_transitions() {
    let mut app = build_headless_app();
    setup_cg_boss(&mut app, 1);
    app.world_mut().insert_resource(Difficulty::Newbro);

    app.update();
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_boss)
        .expect("spawn_cg_boss should run");
    app.update(); // flush commands

    // Find the boss entity
    let boss_entity = {
        let mut q = app.world_mut().query::<(Entity, &rebellion::entities::EnemyStats)>();
        q.iter(app.world())
            .find(|(_, s)| s.is_boss)
            .map(|(e, _)| e)
            .expect("boss should exist")
    };

    let max_hp = CGBossType::PatrolCommander.health() * Difficulty::Newbro.enemy_health_mult();

    // Phase 1 at full health
    {
        let boss = app.world().get::<CGBoss>(boss_entity).unwrap();
        assert_eq!(boss.current_phase, 1, "should start in phase 1");
    }

    // Drop to 40% → should transition to phase 2 (threshold for 2-phase: 1 - 1/2 = 0.5)
    {
        let mut e = app.world_mut().entity_mut(boss_entity);
        let mut boss = e.get_mut::<CGBoss>().unwrap();
        boss.health = max_hp * 0.4;
        let mut stats = e.get_mut::<rebellion::entities::EnemyStats>().unwrap();
        stats.health = max_hp * 0.4;
    }

    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::update_cg_boss)
        .expect("update_cg_boss should run");

    {
        let boss = app.world().get::<CGBoss>(boss_entity).unwrap();
        assert_eq!(
            boss.current_phase, 2,
            "PatrolCommander should transition to phase 2 below 50% HP"
        );
    }

    let movement = app.world().get::<CGBossMovement>(boss_entity).unwrap();
    assert!(
        (movement.speed - 96.0).abs() < 0.01,
        "PatrolCommander speed should increase 1.2× in phase 2, got {}",
        movement.speed
    );
}

#[test]
fn fleet_commander_spawns_with_correct_hp() {
    let mut app = build_headless_app();
    setup_cg_boss(&mut app, 2); // Mission 3 index

    app.world_mut().insert_resource(Difficulty::Newbro);

    app.update();
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_boss)
        .expect("spawn_cg_boss should run");
    app.update();

    let mut q = app.world_mut().query::<&rebellion::entities::EnemyStats>();
    let stats: Vec<&rebellion::entities::EnemyStats> = q.iter(app.world()).collect();
    let boss_stats = stats.iter().find(|s| s.is_boss).expect("boss should spawn");

    let expected = CGBossType::FleetCommander.health() * Difficulty::Newbro.enemy_health_mult();
    assert!(
        (boss_stats.max_health - expected).abs() < 0.1,
        "FleetCommander HP should be {expected}, got {}",
        boss_stats.max_health
    );
}

#[test]
fn fleet_commander_has_three_phases() {
    let mut app = build_headless_app();
    setup_cg_boss(&mut app, 2);
    app.world_mut().insert_resource(Difficulty::Newbro);

    app.update();
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_boss)
        .expect("spawn_cg_boss should run");
    app.update();

    let boss_entity = {
        let mut q = app.world_mut().query::<(Entity, &rebellion::entities::EnemyStats)>();
        q.iter(app.world())
            .find(|(_, s)| s.is_boss)
            .map(|(e, _)| e)
            .expect("boss should exist")
    };

    let max_hp = CGBossType::FleetCommander.health() * Difficulty::Newbro.enemy_health_mult();

    // Phase 1
    {
        let boss = app.world().get::<CGBoss>(boss_entity).unwrap();
        assert_eq!(boss.total_phases, 3, "FleetCommander should have 3 phases");
        assert_eq!(boss.current_phase, 1);
    }

    // Phase 2 at 60% (threshold = 1 - 1/3 = 0.667)
    {
        let mut e = app.world_mut().entity_mut(boss_entity);
        let mut boss = e.get_mut::<CGBoss>().unwrap();
        boss.health = max_hp * 0.6;
        let mut stats = e.get_mut::<rebellion::entities::EnemyStats>().unwrap();
        stats.health = max_hp * 0.6;
    }
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::update_cg_boss)
        .expect("update_cg_boss should run");
    {
        let boss = app.world().get::<CGBoss>(boss_entity).unwrap();
        assert_eq!(boss.current_phase, 2);
    }

    // Phase 3 at 30% (threshold = 1 - 2/3 = 0.333)
    {
        let mut e = app.world_mut().entity_mut(boss_entity);
        let mut boss = e.get_mut::<CGBoss>().unwrap();
        boss.health = max_hp * 0.3;
        let mut stats = e.get_mut::<rebellion::entities::EnemyStats>().unwrap();
        stats.health = max_hp * 0.3;
    }
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::update_cg_boss)
        .expect("update_cg_boss should run");
    {
        let boss = app.world().get::<CGBoss>(boss_entity).unwrap();
        assert_eq!(boss.current_phase, 3);
    }
}

#[test]
fn boss_encounter_produces_deterministic_state_hash() {
    let mut app1 = build_headless_app();
    setup_cg_boss(&mut app1, 2);
    app1.world_mut().insert_resource(Difficulty::Newbro);
    app1.update();
    app1.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_boss)
        .expect("spawn_cg_boss should run");
    app1.update();
    let hash1 = app1.world().resource::<SimStateHash>().0;

    let mut app2 = build_headless_app();
    setup_cg_boss(&mut app2, 2);
    app2.world_mut().insert_resource(Difficulty::Newbro);
    app2.update();
    app2.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_boss)
        .expect("spawn_cg_boss should run");
    app2.update();
    let hash2 = app2.world().resource::<SimStateHash>().0;

    assert_eq!(
        hash1, hash2,
        "FleetCommander boss spawn should produce deterministic state hash"
    );
}
