//! Replay Recorder
//!
//! Captures input events frame-by-frame into a ReplayData buffer.

use bevy::prelude::*;

use super::serializer::{ReplayData, ReplayFrame, ReplayHeader};

/// Resource that records gameplay input for later playback.
#[derive(Resource, Debug, Clone)]
pub struct ReplayRecorder {
    /// Is recording active?
    pub active: bool,
    /// Pre-allocated frame buffer (avoids per-frame allocation).
    pub data: ReplayData,
    /// Maximum frames to record (prevents unbounded growth).
    pub max_frames: usize,
}

impl Default for ReplayRecorder {
    fn default() -> Self {
        Self {
            active: false,
            data: ReplayData::default(),
            max_frames: 60 * 60 * 10, // 10 minutes at 60 fps
        }
    }
}

impl ReplayRecorder {
    /// Start a new recording with the given mission seed.
    pub fn start(&mut self, mission_seed: u64) {
        self.active = true;
        self.data = ReplayData {
            header: ReplayHeader {
                version: super::serializer::REPLAY_VERSION,
                mission_seed,
                total_frames: 0,
            },
            frames: Vec::with_capacity(60 * 60), // Pre-size for 1 minute
        };
    }

    /// Stop recording and return the captured data.
    pub fn stop(&mut self) -> Option<ReplayData> {
        if !self.active {
            return None;
        }
        self.active = false;
        self.data.header.total_frames = self.data.frames.len() as u32;
        Some(self.data.clone())
    }

    /// Record a single frame of keyboard input.
    pub fn record_frame(&mut self, keyboard: &ButtonInput<KeyCode>) {
        if !self.active {
            return;
        }
        if self.data.frames.len() >= self.max_frames {
            self.active = false;
            return;
        }

        let frame = ReplayFrame {
            keys_pressed: keyboard.get_pressed().copied().collect(),
            keys_released: keyboard.get_just_released().copied().collect(),
        };
        self.data.frames.push(frame);
    }
}

/// System that records input each frame.
pub fn replay_record_system(
    mut recorder: ResMut<ReplayRecorder>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    recorder.record_frame(&keyboard);
}
