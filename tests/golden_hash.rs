//! Golden hash regression test
//!
//! Replays a deterministic 180-frame input sequence in headless mode and
//! asserts three `SimStateHash` snapshots. If the golden hashes drift, the
//! test fails — forcing the developer to investigate the source of
//! non-determinism before merging.
//!
//! To update the golden values after an intentional physics or game-logic
//! change, run:
//!   UPDATE_GOLDEN=1 cargo test golden_hash_regression -- --nocapture

use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::GameState;
use rebellion::entities::{Enemy, EnemyStats};
use rebellion::replay::serializer::{ReplayFrame, ReplayHeader};
use rebellion::replay::{ReplayData, ReplayPlayback};
use rebellion::simulation::sim_id::SimId;
use rebellion::simulation::state_hash::SimStateHash;
use rebellion::systems::spawning::{EnemyCarrier, WaveManager};

/// Total replay length in frames (3 seconds at 60 FPS).
const REPLAY_FRAMES: usize = 180;

#[test]
fn golden_hash_regression() {
    let mut app = build_headless_app();

    // ── Phase 1: Enter Playing state ──────────────────────────────────────
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // Frame 1: OnEnter(Playing) spawns player, carrier, wave enemies

    // ── Phase 2: Sanitise the scene for determinism ───────────────────────
    // Despawn all wave enemies (they were spawned by spawn_next_wave).
    let wave_enemies: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<Enemy>>()
        .iter(app.world())
        .collect();
    for e in wave_enemies {
        app.world_mut().commands().entity(e).despawn_recursive();
    }

    // Despawn the carrier so it cannot launch fighters via fastrand.
    let carriers: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<EnemyCarrier>>()
        .iter(app.world())
        .collect();
    for e in carriers {
        app.world_mut().commands().entity(e).despawn_recursive();
    }

    // Prevent wave_spawning from running later in the replay.
    {
        let mut wm = app.world_mut().resource_mut::<WaveManager>();
        wm.wave_delay = 1000.0;
        wm.in_delay = true;
    }

    // Spawn a fully deterministic enemy with no weapon, no AI, no fastrand.
    app.world_mut().commands().spawn((
        Enemy,
        EnemyStats {
            type_id: 597,
            name: "Punisher".into(),
            health: 1000.0,
            max_health: 1000.0,
            speed: 0.0,
            score_value: 0,
            is_boss: false,
            liberation_value: 0,
        },
        Transform::from_xyz(0.0, -200.0, 0.0),
        SimId(1000),
    ));

    // Apply all queued commands and let assign_sim_ids / state_hash run.
    app.update(); // Frame 2: manual enemy now has SimId, scene is clean

    // ── Phase 3: Build and start replay ──────────────────────────────────
    let mut replay = ReplayData {
        header: ReplayHeader {
            version: 1,
            mission_seed: 0,
            total_frames: REPLAY_FRAMES as u32,
        },
        frames: Vec::with_capacity(REPLAY_FRAMES),
    };

    for i in 0..REPLAY_FRAMES {
        let mut frame = ReplayFrame::default();
        if i == 0 {
            frame.keys_pressed = vec![KeyCode::ArrowUp, KeyCode::Space];
        } else if i == 5 {
            frame.keys_released = vec![KeyCode::ArrowUp];
        } else if i == 10 {
            frame.keys_released = vec![KeyCode::Space];
        }
        replay.frames.push(frame);
    }

    app.world_mut()
        .resource_mut::<ReplayPlayback>()
        .start(replay);

    // ── Phase 4: Run replay and capture hashes ────────────────────────────
    let mut captured = Vec::new();
    for frame_idx in 0..REPLAY_FRAMES {
        app.update();

        if frame_idx == 59 || frame_idx == 119 || frame_idx == 179 {
            let hash = app.world().resource::<SimStateHash>().0;
            captured.push(hash);
        }
    }

    // If UPDATE_GOLDEN is set, print the current hashes and skip assertion.
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        println!("Golden hashes (paste into test):");
        println!("  frame 60:  {}", captured[0]);
        println!("  frame 120: {}", captured[1]);
        println!("  frame 180: {}", captured[2]);
        return;
    }

    // ── Phase 5: Assert against known-good snapshots ──────────────────────
    // These values were captured on 2026-07-12 with the deterministic scene.
    let expected: Vec<u64> = vec![
        13982864121045574152,
        13982864121045574152,
        13982864121045574152,
    ];

    assert_eq!(
        captured, expected,
        "SimStateHash drift detected. Run `UPDATE_GOLDEN=1 cargo test golden_hash_regression -- --nocapture` to see new values."
    );
}
