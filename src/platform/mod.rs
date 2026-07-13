//! Platform Plugin
//!
//! Platform-specific input, window, and runtime integration:
//! keyboard, gamepad, touch, WASM bindings, native window.
//!
//! Polls raw input and reduces it into game-intent structures consumed by
//! gameplay and simulation systems.

use bevy::prelude::*;

/// Plugin that registers all platform input and runtime systems.
pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, _app: &mut App) {
        // Empty shell for Mission 1 — systems will migrate here in later PRs.
    }
}
