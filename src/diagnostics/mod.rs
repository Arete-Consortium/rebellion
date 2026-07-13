//! Diagnostics Plugin
//!
//! Development and runtime diagnostics: performance overlays, entity counters,
//! frame-time graphs, event tracing, debug commands, crash artifact capture.
//!
//! Per ERROR_AND_DIAGNOSTICS.md, this plugin is a pure reader — it must not
//! mutate gameplay or simulation state, even for debug convenience.

use bevy::prelude::*;

/// Plugin that registers all diagnostic and profiling systems.
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, _app: &mut App) {
        // Empty shell for Mission 1 — systems will migrate here in later PRs.
    }
}
