//! Game Systems
//!
//! Core gameplay systems: collision, spawning, scoring, effects, input, dialogue, audio.

pub mod ability;
pub mod audio;
pub mod boss;
pub mod campaign;
pub mod collision;
pub mod dialogue;
pub mod effects;
pub mod joystick;
pub mod maneuvers;
pub mod music;
pub mod perf_profile;
pub mod scoring;
pub mod scoring_v2;
pub mod spawning;
pub mod touch_joystick;
pub mod wav_encoder;

pub use ability::*;
pub use audio::*;
pub use boss::*;
pub use campaign::CampaignPlugin;
pub use collision::*;
pub use dialogue::*;
pub use effects::*;
pub use joystick::*;
pub use maneuvers::*;
pub use scoring::*;
pub use scoring_v2::*;
pub use spawning::*;
// touch_joystick intentionally not glob-re-exported — its public items
// are accessed via `crate::systems::touch_joystick::...` if needed.

use bevy::prelude::*;

/// Plugin that registers all gameplay systems
pub struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AbilityPlugin,
            CollisionPlugin,
            SpawningPlugin,
            ScoringPlugin,
            ScoringSystemPlugin,
            BossPlugin,
            ManeuverPlugin,
            CampaignPlugin,
        ));
    }
}
