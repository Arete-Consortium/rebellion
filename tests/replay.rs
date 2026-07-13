//! Replay round-trip test
//!
//! Records input, serializes to JSON, deserializes, and replays.

use rebellion::replay::serializer::ReplayData;
use rebellion::replay::{ReplayPlayback, ReplayRecorder};

#[test]
fn replay_round_trip() {
    // Start recording
    let mut recorder = ReplayRecorder::default();
    recorder.start(42);

    // Simulate 3 frames of input
    for _ in 0..3 {
        recorder.data.frames.push(Default::default());
    }

    // Stop and get data
    let data = recorder.stop().expect("recording stopped");
    assert_eq!(data.frames.len(), 3);
    assert_eq!(data.header.mission_seed, 42);

    // Serialize to JSON
    let json = data.to_json().expect("serialization");
    assert!(!json.is_empty());

    // Deserialize
    let loaded = ReplayData::from_json(&json).expect("deserialization");
    assert_eq!(loaded.frames.len(), 3);
    assert_eq!(loaded.header.mission_seed, 42);

    // Playback
    let mut playback = ReplayPlayback::default();
    playback.start(loaded);
    assert!(playback.active);

    // Tick through all frames
    let mut ticked = 0;
    while playback.tick().is_some() {
        ticked += 1;
    }
    assert_eq!(ticked, 3);
    assert!(!playback.active);
}
