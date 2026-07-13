//! Content Plugin
//!
//! Asset loading, content validation, and data-driven campaign definitions.
//! Responsible for loading RON assets, validating schema versions, and
//! inserting plain data resources for gameplay consumption.
//!
//! Per DEPENDENCY_RULES.md, gameplay must not import content loader types
//! directly; content is loaded here and handed to gameplay via resources.

use bevy::prelude::*;

/// Plugin that registers all content loading and validation systems.
pub struct ContentPlugin;

impl Plugin for ContentPlugin {
    fn build(&self, _app: &mut App) {
        // Empty shell for Mission 1 — systems will migrate here in later PRs.
    }
}
