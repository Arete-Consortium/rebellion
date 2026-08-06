//! Verify the auto-save / reload cycle: write a save file, read
//! it back, mutate the in-memory state, write again, read again.
//!
//! `SavePlugin::auto_save` runs every time `resource_changed::<SaveData>`
//! fires (per `core/save.rs:25`). This test exercises the same
//! pattern at the API level — it's the closest headless
//! approximation of "player made progress, then came back
//! later".
//!
//! Process isolation: separate binary from
//! `save_disk_round_trip.rs`, so no env-var contention.

mod common;

use rebellion::core::SaveData;

#[test]
fn auto_save_writes_file_visible_to_load() {
    let dir = common::install_save_home("auto_save");

    // Empty defaults — first save.
    let initial = SaveData::default();
    initial.save();

    // Read what we wrote.
    let after_first_load = SaveData::load();
    assert!(
        after_first_load.stage_progress.is_empty(),
        "fresh save must have no stage_progress"
    );

    // Mutate, save again, re-read.
    let mut mutated = SaveData::load();
    mutated.complete_stage("Caldari", "Gallente", 3, 1);
    mutated.save();

    let final_state = SaveData::load();
    assert_eq!(
        final_state.get_highest_stage("Caldari", "Gallente"),
        3,
        "mutated save must show on next load"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
