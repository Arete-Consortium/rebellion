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

use combat_reactions::{
    boss_health_callouts, enemy_death_reactions, enemy_hit_reactions, player_death_reactions,
    player_hit_reactions, spawn_chain_bolts, tick_chain_bolts,
};

pub mod combat_reactions;

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
            ))
            .add_systems(
                Update,
                (
                    enemy_hit_reactions,
                    boss_health_callouts,
                    enemy_death_reactions,
                    player_hit_reactions,
                    player_death_reactions,
                    spawn_chain_bolts,
                    tick_chain_bolts,
                )
                    .after(crate::simulation::CollisionPhase::Resolution)
                    .run_if(in_state(crate::core::GameState::Playing)),
            );
    }
}
