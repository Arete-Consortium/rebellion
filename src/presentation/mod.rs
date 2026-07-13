//! Presentation Plugin
//!
//! Visual and audio presentation: sprites, effects, particles, camera,
//! screen shake, hit flash, damage numbers, dialogue display, music, audio,
//! HUD, menus, backgrounds, transitions.
//!
//! Per DEPENDENCY_RULES.md, this plugin reads simulation/gameplay outcome
//! events but never mutates authoritative state.

use bevy::prelude::*;

use crate::core::AudioSettings;
use crate::systems::{
    audio::AudioPlugin, dialogue::DialoguePlugin, effects::EffectsPlugin, music::MusicPlugin,
};
use crate::ui::UiPlugin;

/// Plugin that registers all presentation systems.
pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioSettings>()
            .add_plugins((
                EffectsPlugin,
                AudioPlugin,
                MusicPlugin,
                DialoguePlugin,
                UiPlugin,
            ));
    }
}
