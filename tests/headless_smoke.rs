//! Headless smoke test
//!
//! Builds the headless app and runs a few ticks without panicking.

use bevy::prelude::*;
use rebellion::app_builder::build_headless_app;
use rebellion::core::GameState;

#[test]
fn headless_app_runs_without_panic() {
    let mut app = build_headless_app();

    // Run a few updates to exercise startup + fixed update + update schedules
    for _ in 0..10 {
        app.update();
    }
}

#[test]
fn headless_app_transitions_to_playing() {
    let mut app = build_headless_app();

    // Transition from Loading (default) to Playing
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);

    // Run updates to process state transition + exercise Playing systems
    for _ in 0..60 {
        app.update();
    }

    // Verify we reached Playing state
    let current = app.world().resource::<State<GameState>>().get();
    assert_eq!(*current, GameState::Playing);
}
