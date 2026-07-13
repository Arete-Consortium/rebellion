//! Rebellion - Arcade Space Shooter
//!
//! A Rust/Bevy space arcade game featuring 5 campaigns,
//! factional warfare mechanics, and procedural content.

// Bevy systems naturally have complex query types and many parameters
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

// WASM: Enable better panic messages in browser console
#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;

mod assets;
mod content;
mod core;
mod diagnostics;
mod entities;
mod gameplay;
mod games;
mod platform;
mod presentation;
mod simulation;
mod systems;
mod ui;

use content::ContentPlugin;
use core::{
    AchievementPlugin, ActCompleteEvent, AnalyticsPlugin, BossSpawnEvent, GameEventsPlugin,
    GameState, MissionCompleteEvent, MissionStartEvent, SavePlugin, WaveCompleteEvent,
};
use diagnostics::DiagnosticsPlugin;
use gameplay::GameplayPlugin;
use games::GameModulesPlugin;
use platform::PlatformPlugin;
use presentation::PresentationPlugin;
use simulation::{SimulationPlugin, FIXED_TIMESTEP_SECS};

fn main() {
    // WASM: Set up panic hook for better error messages
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: core::WINDOW_TITLE.into(),
                resolution: (core::SCREEN_WIDTH, core::SCREEN_HEIGHT).into(),
                resizable: true,
                // WASM: fit the canvas to the browser window
                fit_canvas_to_parent: true,
                // WASM: prevent right-click context menu on canvas
                prevent_default_event_handling: true,
                canvas: Some("#bevy-canvas".to_string()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        // Fixed timestep for authoritative simulation systems
        .insert_resource(Time::<Fixed>::from_seconds(FIXED_TIMESTEP_SECS))
        // Game state
        .init_state::<GameState>()
        // Campaign events
        .add_event::<MissionStartEvent>()
        .add_event::<MissionCompleteEvent>()
        .add_event::<WaveCompleteEvent>()
        .add_event::<BossSpawnEvent>()
        .add_event::<ActCompleteEvent>()
        // System set ordering: Simulation → Gameplay → Presentation
        .configure_sets(
            Update,
            (
                simulation::SimSet::Simulation,
                simulation::SimSet::Gameplay,
                simulation::SimSet::Presentation,
            )
                .chain(),
        )
        // Game plugins
        .add_plugins((
            PlatformPlugin,
            ContentPlugin,
            SimulationPlugin,
            GameplayPlugin,
            PresentationPlugin,
            DiagnosticsPlugin,
            SavePlugin,
            AnalyticsPlugin,
            AchievementPlugin,
            GameEventsPlugin,
            GameModulesPlugin,
        ))
        // Setup
        .add_systems(Startup, setup)
        .run();
}

/// Initial game setup
fn setup(mut commands: Commands) {
    // Use 2D camera - sprites work reliably with this
    commands.spawn(Camera2d);

    info!("Rebellion initialized!");
}
