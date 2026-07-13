//! Replay Serializer
//!
//! Versioned replay file format for save/load.

use bevy::prelude::KeyCode;
use serde::{Deserialize, Serialize};

/// Current replay format version.
pub const REPLAY_VERSION: u32 = 1;

/// Header at the start of every replay file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayHeader {
    pub version: u32,
    pub mission_seed: u64,
    pub total_frames: u32,
}

impl Default for ReplayHeader {
    fn default() -> Self {
        Self {
            version: REPLAY_VERSION,
            mission_seed: 0,
            total_frames: 0,
        }
    }
}

/// Serializable replay frame.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayFrame {
    /// Keyboard keys pressed this frame.
    pub keys_pressed: Vec<KeyCode>,
    /// Keyboard keys released this frame.
    pub keys_released: Vec<KeyCode>,
}

/// Full replay data (header + frames).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplayData {
    pub header: ReplayHeader,
    pub frames: Vec<ReplayFrame>,
}

impl ReplayData {
    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
