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
mod core;
mod entities;
mod games;
mod systems;
mod ui;

use assets::AssetsPlugin;
use core::{
    AchievementPlugin, ActCompleteEvent, AnalyticsPlugin, AudioSettings, BossSpawnEvent,
    CampaignState, CurrentStage, Difficulty, EndlessMode, GameEventsPlugin, GameProgress,
    GameSession, GameState, InputConfig, MissionCompleteEvent, MissionStartEvent, SaltMinerSystem,
    SavePlugin, ScoreSystem, SelectedShip, ShipUnlocks, WaveCompleteEvent,
};
use entities::EntitiesPlugin;
use games::GameModulesPlugin;
use systems::SystemsPlugin;
use ui::UiPlugin;

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
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        // Game state
        .init_state::<GameState>()
        // Resources
        .init_resource::<ScoreSystem>()
        .init_resource::<SaltMinerSystem>()
        .init_resource::<GameProgress>()
        .init_resource::<InputConfig>()
        .init_resource::<AudioSettings>()
        .init_resource::<Difficulty>()
        .init_resource::<SelectedShip>()
        .init_resource::<CurrentStage>()
        .init_resource::<ShipUnlocks>()
        .init_resource::<CampaignState>()
        .init_resource::<GameSession>()
        .init_resource::<EndlessMode>()
        // Campaign events
        .add_event::<MissionStartEvent>()
        .add_event::<MissionCompleteEvent>()
        .add_event::<WaveCompleteEvent>()
        .add_event::<BossSpawnEvent>()
        .add_event::<ActCompleteEvent>()
        // Game plugins
        .add_plugins((
            SavePlugin,
            AnalyticsPlugin,
            AchievementPlugin,
            AssetsPlugin,
            GameEventsPlugin,
            EntitiesPlugin,
            SystemsPlugin,
            UiPlugin,
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
