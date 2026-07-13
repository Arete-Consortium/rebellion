//! Diagnostics Plugin
//!
//! Development and runtime diagnostics: performance overlays, entity counters,
//! frame-time graphs, event tracing, debug commands, crash artifact capture.
//!
//! Per ERROR_AND_DIAGNOSTICS.md, this plugin is a pure reader — it must not
//! mutate gameplay or simulation state, even for debug convenience.

use bevy::prelude::*;

use crate::systems::perf_profile::PerfProfilePlugin;

/// Plugin that registers all diagnostic and profiling systems.
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PerfProfilePlugin);
    }
}
