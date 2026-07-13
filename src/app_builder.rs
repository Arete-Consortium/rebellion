//! App Builder
//!
//! Provides `build_headless_app()` for CI/tests — no window, no renderer, no audio.
//! Core ECS, schedules, and simulation systems run and process FixedUpdate ticks.

use bevy::prelude::*;

use crate::core::{
    ActCompleteEvent, BossSpawnEvent, GameState, MissionCompleteEvent, MissionStartEvent,
    WaveCompleteEvent,
};
use crate::simulation::{SimulationPlugin, FIXED_TIMESTEP_SECS};

/// Build a headless app for CI smoke tests.
///
/// # Notes
/// - Uses `MinimalPlugins` (no window, no render, no audio).
/// - Adds `AssetPlugin` and `StatesPlugin` for game infrastructure.
/// - Registers only `SimulationPlugin` to avoid asset-loading systems that
///   depend on `bevy_render`.
pub fn build_headless_app() -> App {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::state::app::StatesPlugin,
    ));

    // Fixed timestep
    app.insert_resource(Time::<Fixed>::from_seconds(FIXED_TIMESTEP_SECS));

    // Game state
    app.init_state::<GameState>();

    // Campaign events
    app.add_event::<MissionStartEvent>()
        .add_event::<MissionCompleteEvent>()
        .add_event::<WaveCompleteEvent>()
        .add_event::<BossSpawnEvent>()
        .add_event::<ActCompleteEvent>();

    // System set ordering
    app.configure_sets(
        Update,
        (
            crate::simulation::SimSet::Simulation,
            crate::simulation::SimSet::Gameplay,
            crate::simulation::SimSet::Presentation,
        )
            .chain(),
    );

    // Core simulation (no presentation, no content loading)
    app.add_plugins(SimulationPlugin);

    app
}
