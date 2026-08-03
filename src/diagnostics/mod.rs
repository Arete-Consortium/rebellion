//! Diagnostics Plugin
//!
//! Development and runtime diagnostics: performance overlays, entity counters,
//! frame-time graphs, event tracing, debug commands, crash artifact capture.
//!
//! Per ERROR_AND_DIAGNOSTICS.md, this plugin is a pure reader — it must not
//! mutate gameplay or simulation state, even for debug convenience.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::core::GameState;
use crate::systems::perf_profile::PerfProfilePlugin;

/// Plugin that registers all diagnostic and profiling systems.
pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PerfProfilePlugin)
            .init_resource::<BossFrameProfiler>()
            .add_systems(
                Update,
                log_boss_frame_time_spikes.run_if(in_state(GameState::BossFight)),
            );
    }
}

// =============================================================================
// BOSS FRAME TIME PROFILER
// =============================================================================

/// Rolling-frame profiler that warns when frame time spikes during boss fights.
///
/// `FrameTimeDiagnosticsPlugin` is already registered by `TouchJoystickPlugin`
/// in native builds, so we only attach a consumer system here.
#[derive(Resource)]
pub struct BossFrameProfiler {
    samples: Vec<f32>,
    max_samples: usize,
}

impl Default for BossFrameProfiler {
    fn default() -> Self {
        Self {
            samples: Vec::with_capacity(60),
            max_samples: 60,
        }
    }
}

impl BossFrameProfiler {
    pub fn add_sample(&mut self, dt_secs: f32) {
        if self.samples.len() >= self.max_samples {
            self.samples.remove(0);
        }
        self.samples.push(dt_secs);
    }

    /// Average frame time in milliseconds.
    pub fn average_ms(&self) -> f32 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.samples.iter().sum::<f32>() / self.samples.len() as f32 * 1000.0
        }
    }
}

/// Log a warning when frame time exceeds 20 ms (drops below 50 fps) during
/// a boss fight. Also prints a rolling average so trends are visible in logs.
fn log_boss_frame_time_spikes(
    diagnostics: Res<DiagnosticsStore>,
    mut profiler: ResMut<BossFrameProfiler>,
) {
    if let Some(frame_time) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
    {
        let ft = frame_time as f32;
        profiler.add_sample(ft);
        if ft > 0.020 {
            warn!(
                "FRAME TIME SPIKE during boss fight: {:.2} ms (rolling avg {:.2} ms over {} frames)",
                ft * 1000.0,
                profiler.average_ms(),
                profiler.samples.len(),
            );
        }
    }
}
