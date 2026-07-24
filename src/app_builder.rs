//! App Builder
//!
//! Provides `build_headless_app()` for CI/tests — no window, no renderer, no audio.
//! Core ECS, schedules, and simulation systems run and process FixedUpdate ticks.

use bevy::prelude::*;

use crate::core::{
    ActCompleteEvent, BossSpawnEvent, GameEventsPlugin, GameState, MissionCompleteEvent,
    MissionStartEvent, SavePlugin, WaveCompleteEvent,
};
use crate::core::events::BossDefeatedEvent;
use crate::gameplay::GameplayPlugin;
use crate::replay::ReplayPlugin;
use crate::simulation::{SimulationPlugin, FIXED_TIMESTEP_SECS};

/// Build a headless app for CI smoke tests.
///
/// # Notes
/// - Uses `MinimalPlugins` (no window, no render, no audio).
/// - Adds `AssetPlugin` and `StatesPlugin` for game infrastructure.
/// - Registers `SimulationPlugin` and `GameplayPlugin` for deterministic tests.
/// - Stubs resources normally provided by presentation / platform plugins.
pub fn build_headless_app() -> App {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::state::app::StatesPlugin,
    ));

    // Fixed timestep
    app.insert_resource(Time::<Fixed>::from_seconds(FIXED_TIMESTEP_SECS));

    // In headless mode, advance time by exactly one fixed tick per update
    // so that FixedUpdate systems run deterministically in tests.
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(FIXED_TIMESTEP_SECS),
    ));

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

    // Gameplay systems (deterministic, no rendering required)
    app.add_plugins((GameplayPlugin, SavePlugin, GameEventsPlugin, ReplayPlugin));

    // Events added by gameplay sub-plugins (not covered by GameEventsPlugin)
    app.add_event::<crate::systems::ability::AbilityActivatedEvent>()
        .add_event::<crate::systems::ability::AbilityEndedEvent>()
        .add_event::<crate::systems::boss::BossSpawnEvent>()
        .add_event::<crate::systems::boss::BossDefeatedEvent>()
        .add_event::<BossDefeatedEvent>() // core::events variant used by check_boss_defeated
        .add_event::<crate::systems::dialogue::DialogueEvent>()
        .add_event::<crate::systems::joystick::RumbleRequest>()
        .add_event::<crate::systems::joystick::BackButtonEvent>()
        .add_event::<crate::core::EnemyDestroyedEvent>();

    // Stub resources normally provided by presentation / platform plugins
    app.insert_resource(crate::simulation::SimulationRng::from_seed(
        crate::simulation::DEFAULT_MISSION_SEED,
    ))
    .init_resource::<crate::core::ScoreSystem>()
    .init_resource::<crate::core::SaltMinerSystem>()
    .init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<crate::assets::ShipSpriteCache>()
        .init_resource::<crate::assets::ShipModelCache>()
        .init_resource::<crate::assets::PowerupIconCache>()
        .init_resource::<crate::systems::JoystickState>()
        .init_resource::<crate::systems::ScreenFlash>()
        .init_resource::<crate::systems::ScreenShake>()
        .init_resource::<crate::systems::SoundSettings>()
        .init_resource::<crate::systems::RumbleSettings>()
        .init_resource::<crate::systems::DialogueSystem>()
        .init_resource::<crate::games::ActiveModule>();

    app
}
