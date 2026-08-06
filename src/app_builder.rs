//! App Builder
//!
//! Provides `RebellionAppConfig` for constructing the Bevy app in any mode
//! (native, headless test). Eliminates duplication between `main.rs`
//! and `app_builder.rs`.

use bevy::prelude::*;

use crate::core::{
    ActCompleteEvent, CampaignBossSpawned, GameEventsPlugin, GameState, KeyBindingsPlugin,
    MissionCompleteEvent, MissionStartEvent, SavePlugin, WaveCompleteEvent,
};
use crate::core::events::BossDefeatedEvent;
use crate::gameplay::GameplayPlugin;
use crate::simulation::{SimulationPlugin, FIXED_TIMESTEP_SECS};

/// Runtime mode for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMode {
    /// Full native build with window, renderer, audio, and all plugins.
    Native,
    /// Headless build for CI/tests — no window, no renderer, no audio.
    Headless,
}

/// Configuration for building a Rebellion application.
///
/// Use `RebellionAppConfig::native()` for the player-facing binary
/// and `RebellionAppConfig::headless_test()` for deterministic CI tests.
pub struct RebellionAppConfig {
    pub mode: RuntimeMode,
}

impl RebellionAppConfig {
    /// Native build with full presentation stack.
    pub fn native() -> Self {
        Self {
            mode: RuntimeMode::Native,
        }
    }

    /// Headless build for deterministic integration tests.
    pub fn headless_test() -> Self {
        Self {
            mode: RuntimeMode::Headless,
        }
    }

    /// Build the configured Bevy `App`.
    pub fn build(self) -> App {
        let mut app = App::new();

        match self.mode {
            RuntimeMode::Native => self.configure_native_bevy_plugins(&mut app),
            RuntimeMode::Headless => self.configure_headless_bevy_plugins(&mut app),
        }

        self.configure_shared(&mut app);

        match self.mode {
            RuntimeMode::Native => self.configure_native_plugins(&mut app),
            RuntimeMode::Headless => self.configure_headless_plugins(&mut app),
        }

        app
    }

    // -- Bevy plugin configuration --

    fn configure_native_bevy_plugins(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: crate::core::WINDOW_TITLE.into(),
                resolution: (crate::core::SCREEN_WIDTH, crate::core::SCREEN_HEIGHT).into(),
                resizable: true,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                canvas: Some("#bevy-canvas".to_string()),
                ..default()
            }),
            ..default()
        }));

        // Egui is native-only
        app.add_plugins(bevy_egui::EguiPlugin);
    }

    fn configure_headless_bevy_plugins(&self, app: &mut App) {
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::state::app::StatesPlugin,
        ));

        // In headless mode, advance time by exactly one fixed tick per update
        // so that FixedUpdate systems run deterministically in tests.
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(FIXED_TIMESTEP_SECS),
        ));
    }

    // -- Shared configuration --

    fn configure_shared(&self, app: &mut App) {
        // Fixed timestep for authoritative simulation systems
        app.insert_resource(Time::<Fixed>::from_seconds(FIXED_TIMESTEP_SECS));

        // Game state
        app.init_state::<GameState>();

        // Campaign events
        app.add_event::<MissionStartEvent>()
            .add_event::<MissionCompleteEvent>()
            .add_event::<WaveCompleteEvent>()
            .add_event::<CampaignBossSpawned>()
            .add_event::<ActCompleteEvent>();

        // System set ordering: Simulation → Gameplay → Presentation
        app.configure_sets(
            Update,
            (
                crate::simulation::SimSet::Simulation,
                crate::simulation::SimSet::Gameplay,
                crate::simulation::SimSet::Presentation,
            )
                .chain(),
        );

        // Core simulation (authoritative — no presentation)
        app.add_plugins(SimulationPlugin);

        // KeyBindings is the authoritative input table. Registered in
        // configure_shared so both native and headless app paths
        // guarantee the resource exists before any gameplay system
        // reads it. See `core::keybindings` for the binding rules.
        app.add_plugins(KeyBindingsPlugin);
    }

    // -- Mode-specific plugin configuration --

    fn configure_native_plugins(&self, app: &mut App) {
        use crate::{
            content::ContentPlugin,
            core::{AchievementPlugin, AnalyticsPlugin},
            diagnostics::DiagnosticsPlugin,
            games::GameModulesPlugin,
            platform::PlatformPlugin,
            presentation::PresentationPlugin,
        };

        app.add_plugins((
            PlatformPlugin,
            ContentPlugin,
            GameplayPlugin,
            PresentationPlugin,
            DiagnosticsPlugin,
            SavePlugin,
            AnalyticsPlugin,
            AchievementPlugin,
            GameEventsPlugin,
            GameModulesPlugin,
        ));
    }

    fn configure_headless_plugins(&self, app: &mut App) {
        use crate::replay::ReplayPlugin;

        app.add_plugins((GameplayPlugin, SavePlugin, GameEventsPlugin, ReplayPlugin));

        // Events added by gameplay sub-plugins (not covered by GameEventsPlugin)
        app.add_event::<crate::systems::ability::AbilityActivatedEvent>()
            .add_event::<crate::systems::ability::AbilityEndedEvent>()
            .add_event::<crate::systems::boss::BossEntitySpawned>()
            .add_event::<crate::systems::boss::BossEntityDefeated>()
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
    }
}

/// Build a headless app for CI smoke tests.
///
/// Backward-compatible wrapper around `RebellionAppConfig::headless_test().build()`.
pub fn build_headless_app() -> App {
    RebellionAppConfig::headless_test().build()
}
