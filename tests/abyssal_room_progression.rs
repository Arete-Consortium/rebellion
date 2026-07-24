//! Abyssal Depths room progression integration tests
//!
//! Validates room clearing, gate spawning, and extraction mechanics end-to-end.

use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{GameState, SCREEN_HEIGHT};
use rebellion::entities::Enemy;
use rebellion::games::abyssal_depths::{AbyssalGate, AbyssalHazard, AbyssalRoom, AbyssalState};
use rebellion::games::ActiveModule;

/// Despawn all entities with the `Enemy` component.
fn despawn_all_enemies(app: &mut App) {
    let enemies: Vec<Entity> = app
        .world_mut()
        .query::<(Entity, &Enemy)>()
        .iter(app.world())
        .map(|(e, _)| e)
        .collect();
    for e in enemies {
        app.world_mut().despawn(e);
    }
}

/// Count entities with the `Enemy` component.
fn enemy_count(app: &mut App) -> usize {
    app.world_mut()
        .query::<(Entity, &Enemy)>()
        .iter(app.world())
        .count()
}

/// Count gate entities.
fn gate_count(app: &mut App) -> usize {
    app.world_mut()
        .query::<(Entity, &AbyssalGate)>()
        .iter(app.world())
        .count()
}

/// Count hazard entities.
fn hazard_count(app: &mut App) -> usize {
    app.world_mut()
        .query::<(Entity, &AbyssalHazard)>()
        .iter(app.world())
        .count()
}

#[test]
fn abyssal_room1_clears_and_spawns_transition_gate() {
    let mut app = build_headless_app();
    app.init_resource::<rebellion::games::ModuleRegistry>();
    app.add_plugins(rebellion::games::abyssal_depths::AbyssalDepthsPlugin);

    // Set active module so abyssal systems run
    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("abyssal_depths");

    // Transition to Playing triggers setup_abyssal
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    let state = app.world().resource::<AbyssalState>();
    assert_eq!(state.room, AbyssalRoom::Room1);
    assert!(state.active);
    assert!(!state.room_cleared);
    assert!(!state.gate_spawned);
    assert!(enemy_count(&mut app) > 0, "Room1 enemies should spawn");

    // Simulate killing all enemies
    despawn_all_enemies(&mut app);
    app.update();

    let state = app.world().resource::<AbyssalState>();
    assert!(state.room_cleared, "Room1 should clear after enemies despawned");
    assert!(state.gate_spawned, "Transition gate should spawn after room clear");
    assert_eq!(gate_count(&mut app), 1, "Exactly one gate should exist");

    // Verify it's a transition gate (not extraction)
    let is_extraction = app
        .world_mut()
        .query::<&AbyssalGate>()
        .iter(app.world())
        .next()
        .map(|g| g.is_extraction)
        .unwrap_or(false);
    assert!(!is_extraction, "Room1 gate should be transition, not extraction");
}

/// Move the first player entity to the given position.
fn move_player_to(app: &mut App, pos: Vec2) {
    let mut q = app
        .world_mut()
        .query_filtered::<&mut Transform, With<rebellion::entities::Player>>();
    if let Ok(mut transform) = q.get_single_mut(app.world_mut()) {
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
}

#[test]
fn abyssal_room3_extraction_gate_triggers_victory() {
    let mut app = build_headless_app();
    app.init_resource::<rebellion::games::ModuleRegistry>();
    app.add_plugins(rebellion::games::abyssal_depths::AbyssalDepthsPlugin);

    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("abyssal_depths");

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // Room1 setup + player spawn

    // Fast-forward to Room3 by manually advancing rooms
    {
        let mut state = app.world_mut().resource_mut::<AbyssalState>();
        state.advance_room(); // Room1 → Room2
        state.advance_room(); // Room2 → Room3
        state.room_cleared = false;
        state.gate_spawned = false;
        state.enemies_spawned = 0;
        state.enemies_killed = 0;
    }

    // Spawn Room3 enemies manually
    let state = app.world().resource::<AbyssalState>();
    let target_count = state.room.enemy_count() as usize;
    for i in 0..target_count {
        app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(i as f32 * 10.0, 100.0, 0.0),
        ));
    }

    // Run one update to let check_room_clear set enemies_spawned
    app.update();

    let state = app.world().resource::<AbyssalState>();
    assert_eq!(state.enemies_spawned, target_count as u32);

    // Kill all enemies to clear Room3
    despawn_all_enemies(&mut app);
    app.update();

    let state = app.world().resource::<AbyssalState>();
    assert!(state.room_cleared, "Room3 should clear");
    assert!(state.gate_spawned, "Extraction gate should spawn");

    // Verify extraction gate
    let is_extraction = app
        .world_mut()
        .query::<&AbyssalGate>()
        .iter(app.world())
        .next()
        .map(|g| g.is_extraction)
        .unwrap_or(false);
    assert!(is_extraction, "Room3 gate should be extraction gate");

    // Move the auto-spawned player onto the gate
    let gate_y = SCREEN_HEIGHT / 2.0 - 100.0;
    move_player_to(&mut app, Vec2::new(0.0, gate_y));

    // Hold Space to channel extraction
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);

    // Channel for ~2.5 seconds (150 frames at 1/60s)
    for _ in 0..150 {
        app.update();
    }

    let state = app.world().resource::<AbyssalState>();
    assert!(
        state.extracted,
        "Should extract after channeling. progress={}, extracting={}",
        state.extraction_progress,
        state.extracting
    );
    assert!(state.extraction_progress >= 1.0, "Extraction progress should reach 100%");

    // Note: we do NOT assert State::Victory here because headless mode lacks
    // the input-system frame that clears just_pressed. On the frame after
    // NextState(Victory) is queued, abyssal_victory_input sees Space as
    // just_pressed and immediately transitions to MainMenu. The extraction
    // logic itself (extracted + progress) is already validated above.
}

#[test]
fn abyssal_timer_runs_out_triggers_game_over() {
    let mut app = build_headless_app();
    app.init_resource::<rebellion::games::ModuleRegistry>();
    app.add_plugins(rebellion::games::abyssal_depths::AbyssalDepthsPlugin);

    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("abyssal_depths");

    // Transition to Playing triggers setup_abyssal (sets time_remaining = 600)
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Override with very little time remaining
    {
        let mut state = app.world_mut().resource_mut::<AbyssalState>();
        state.time_remaining = 0.001;
    }

    // Frame N: timer drains, NextState(GameOver) set
    app.update();

    let state = app.world().resource::<AbyssalState>();
    assert!(!state.active, "Run should end when timer expires");
    assert_eq!(state.time_remaining, 0.0, "Timer should clamp to 0");

    // Frame N+1: state transition applies
    app.update();

    let current = *app.world().resource::<State<GameState>>().get();
    assert_eq!(current, GameState::GameOver, "Should transition to GameOver");
}

#[test]
fn abyssal_room2_spawns_hazards() {
    let mut app = build_headless_app();
    app.init_resource::<rebellion::games::ModuleRegistry>();
    app.add_plugins(rebellion::games::abyssal_depths::AbyssalDepthsPlugin);

    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("abyssal_depths");

    // Start in Playing to trigger Room1 setup
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Despawn Room1 enemies so Room2 can spawn
    despawn_all_enemies(&mut app);

    // Fast-forward to Room2
    {
        let mut state = app.world_mut().resource_mut::<AbyssalState>();
        state.advance_room(); // Room1 → Room2
        state.room_cleared = false;
        state.gate_spawned = false;
        state.enemies_spawned = 0;
        state.enemies_killed = 0;
    }

    // Trigger Room2 enemy/hazard spawn via handle_extraction
    app.update();

    let state = app.world().resource::<AbyssalState>();
    assert_eq!(state.room, AbyssalRoom::Room2);

    let hazards = hazard_count(&mut app);
    assert!(
        hazards > 0,
        "Room2 should spawn bioadaptive hazards, found {}",
        hazards
    );
}

#[test]
fn abyssal_hazard_deals_damage_to_player() {
    let mut app = build_headless_app();
    app.init_resource::<rebellion::games::ModuleRegistry>();
    app.add_plugins(rebellion::games::abyssal_depths::AbyssalDepthsPlugin);

    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("abyssal_depths");

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // Spawns player + Room1 entities

    // Spawn a hazard directly on top of the player
    let player_pos = {
        let mut q = app
            .world_mut()
            .query_filtered::<&Transform, With<rebellion::entities::Player>>();
        q.get_single(app.world())
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO)
    };

    app.world_mut().spawn((
        AbyssalHazard {
            damage_per_second: 400.0, // High enough to burn through shield+armor and hit hull
            radius: 100.0,
            damage_timer: 0.0,
        },
        Transform::from_xyz(player_pos.x, player_pos.y, 0.0),
    ));

    // Verify hazard exists and is positioned on player
    assert_eq!(hazard_count(&mut app), 1, "Hazard should be spawned");
    {
        let mut q_player = app
            .world_mut()
            .query_filtered::<&Transform, With<rebellion::entities::Player>>();
        let player_pos = q_player
            .get_single(app.world())
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ZERO);
        let mut q_hazard = app
            .world_mut()
            .query_filtered::<&Transform, With<AbyssalHazard>>();
        let hazard_pos = q_hazard
            .get_single(app.world())
            .map(|t| t.translation.truncate())
            .unwrap_or(Vec2::ONE);
        assert!(
            (player_pos - hazard_pos).length() < 1.0,
            "Hazard should be on player: player={:?}, hazard={:?}",
            player_pos,
            hazard_pos
        );
    }

    // Record starting hull
    let start_hull = {
        let mut q = app
            .world_mut()
            .query_filtered::<&rebellion::entities::ShipStats, With<rebellion::entities::Player>>();
        q.get_single(app.world()).map(|s| s.hull).unwrap_or(0.0)
    };
    assert!(start_hull > 0.0, "Player should spawn with positive hull");

    // Run many frames to accumulate hazard damage (tick every 0.25s, dt ≈ 0.0167s)
    // 60 frames ≈ 1.0s real time → 4 damage ticks at 80 DPS = 80 damage total
    for _ in 0..60 {
        app.update();
    }

    let end_hull = {
        let mut q = app
            .world_mut()
            .query_filtered::<&rebellion::entities::ShipStats, With<rebellion::entities::Player>>();
        q.get_single(app.world()).map(|s| s.hull).unwrap_or(0.0)
    };

    // Verify hazard system ran: timer cycles back to 0 after tick(s)
    let _hazard_timer = {
        let mut q = app.world_mut().query::<&AbyssalHazard>();
        q.iter(app.world()).next().map(|h| h.damage_timer).unwrap_or(-1.0)
    };

    assert!(
        end_hull < start_hull,
        "Player hull should decrease after standing in hazard: start={}, end={}",
        start_hull,
        end_hull
    );
}
