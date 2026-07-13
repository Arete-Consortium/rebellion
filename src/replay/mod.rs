//! Replay System
//!
//! Records player input and replays it for deterministic testing.
//!
//! # Usage
//! 1. Call `ReplayRecorder::start(seed)` before gameplay.
//! 2. `ReplayPlugin` automatically captures input each frame.
//! 3. Call `ReplayRecorder::stop()` to get `ReplayData`.
//! 4. Save `ReplayData::to_json()` to disk.
//! 5. Load from disk, call `ReplayPlayback::start(data)` to replay.

pub mod playback;
pub mod recorder;
pub mod serializer;

use bevy::prelude::*;

pub use playback::{replay_playback_system, ReplayPlayback};
pub use recorder::{replay_record_system, ReplayRecorder};
pub use serializer::ReplayData;

/// Plugin that registers replay recording and playback resources.
pub struct ReplayPlugin;

impl Plugin for ReplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReplayRecorder>()
            .init_resource::<ReplayPlayback>()
            .add_systems(Update, replay_record_system);
    }
}
