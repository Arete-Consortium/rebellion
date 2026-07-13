//! Presentation Plugin
//!
//! Visual and audio presentation: sprites, effects, particles, camera,
//! screen shake, hit flash, damage numbers, dialogue display, music, audio,
//! HUD, menus, backgrounds, transitions.
//!
//! Per DEPENDENCY_RULES.md, this plugin reads simulation/gameplay outcome
//! events but never mutates authoritative state.

use bevy::prelude::*;

/// Plugin that registers all presentation systems.
pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, _app: &mut App) {
        // Empty shell for Mission 1 — systems will migrate here in later PRs.
    }
}
