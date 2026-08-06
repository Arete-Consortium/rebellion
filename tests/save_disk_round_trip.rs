//! Round-trip a populated `SaveData` through disk:
//!
//!   save() → file on disk → load() → equal contents
//!
//! This is the path `SavePlugin::auto_save` runs every time the
//! resource changes, and the path `SavePlugin::load_save_data`
//! runs at startup. If it doesn't round-trip, every saved
//! session is effectively lost.
//!
//! Process isolation: this binary is its own Cargo test target
//! (`cargo test --test save_disk_round_trip`), so `REBELLION_HOME`
//! set by `install_save_home` is owned outright — no other test
//! can race with it. See `tests/common/mod.rs` for the rationale.

mod common;

use rebellion::core::SaveData;

#[test]
fn save_then_load_round_trips_through_disk() {
    let dir = common::install_save_home("round_trip");

    let mut save = SaveData::default();
    save.complete_stage("Minmatar", "Amarr", 7, 4);
    save.record_score("Minmatar", "Amarr", 75000, 7);
    save.unlock_ship(587);

    save.save(); // disk write via REBELLION_HOME

    let loaded = SaveData::load(); // disk read via REBELLION_HOME
    assert_eq!(
        loaded.get_highest_stage("Minmatar", "Amarr"),
        7,
        "highest_stage must survive the disk round-trip"
    );
    assert_eq!(
        loaded.get_high_score("Minmatar", "Amarr"),
        75000,
        "high score must survive the disk round-trip"
    );
    assert!(
        loaded.is_ship_unlocked(587, 5, "Minmatar", "Amarr"),
        "unlocked ship must survive the disk round-trip"
    );

    // Best-effort cleanup. The OS will reap the tempdir if this
    // fails (e.g., another test inspects it via the path).
    let _ = std::fs::remove_dir_all(&dir);
}
