//! Replay Playback
//!
//! Replays captured input frames against the simulation.

use bevy::prelude::*;

use super::serializer::ReplayData;

/// Resource that drives replay playback.
#[derive(Resource, Debug, Clone, Default)]
pub struct ReplayPlayback {
    /// Is playback active?
    pub active: bool,
    /// Frame index currently being replayed.
    pub current_frame: usize,
    /// Total frames in the replay.
    pub total_frames: usize,
    /// The replay data being played back.
    pub data: Option<ReplayData>,
}

impl ReplayPlayback {
    /// Load replay data and start playback from frame 0.
    pub fn start(&mut self, data: ReplayData) {
        self.total_frames = data.frames.len();
        self.data = Some(data);
        self.current_frame = 0;
        self.active = true;
    }

    /// Stop playback.
    pub fn stop(&mut self) {
        self.active = false;
        self.current_frame = 0;
        self.data = None;
    }

    /// Advance one frame and return the current frame's inputs.
    /// Returns `None` when playback reaches the end.
    pub fn tick(&mut self) -> Option<super::serializer::ReplayFrame> {
        if !self.active {
            return None;
        }
        let data = self.data.as_ref()?;
        if self.current_frame >= self.total_frames {
            self.active = false;
            return None;
        }
        let frame = data.frames[self.current_frame].clone();
        self.current_frame += 1;
        Some(frame)
    }
}

/// System that injects replay inputs each frame during playback.
pub fn replay_playback_system(
    mut playback: ResMut<ReplayPlayback>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    if !playback.active {
        return;
    }
    let Some(frame) = playback.tick() else {
        return;
    };

    // Inject key presses
    for key in frame.keys_pressed {
        keyboard.press(key);
    }
    // Inject key releases
    for key in frame.keys_released {
        keyboard.release(key);
    }
}
