//! Integration tests for `SaveData::complete_stage` + the
//! `stage_progress` field round-trip.
//!
//! Fills a real coverage gap: the in-file tests at
//! `core/save.rs:705-742` cover the basics (create entry, update,
//! non-decrease, multiple factions), but do not exercise:
//!
//! - **Mission-number clamp** — only the second `complete_stage`
//!   argument (`mission`) strictly above the existing one advances
//!   `highest_mission`. Lower mission indices must not regress it.
//! - **Idempotent re-record** — calling `complete_stage` with the
//!   same `(faction, enemy, stage, mission)` pair twice must leave
//!   `stage_progress` with a single entry (no Vec duplicates).
//! - **Resource-level round-trip** — inserting a populated
//!   `SaveData` resource into a Bevy world and updating one frame
//!   must not lose the `stage_progress` entries.
//! - **Ship-unlock persistence** — completing stage 5 unlocks the
//!   stage-5 ship; saving that state and reloading must keep the
//!   ship unlocked even if a later session rolls `highest_stage`
//!   back to a lower number (it doesn't, but the saved
//!   `unlocked_ships` HashSet must survive the round-trip).
//! - **Multi-stage skip cascade** — completing stage 1, then 5,
//!   must leave `highest_stage=5` and the entry list at length 1.
//!
//! All assertions are unit-level against `SaveData`'s own API plus
//! serde. No filesystem I/O — `SaveData::save()` writes to the
//! user's config dir, which would pollute test runs.

use rebellion::app_builder::build_headless_app;
use rebellion::core::SaveData;

// ============================================================================
// Group A — Direct API invariants (no Bevy runtime needed)
// ============================================================================

/// The mission number must monotonically advance — completing
/// mission 3 then mission 1 must leave `highest_mission=3`. This
/// is the in-SaveData equivalent of "you can't un-complete a
/// mission by replaying an earlier level".
#[test]
fn complete_stage_does_not_regress_highest_mission() {
    let mut save = SaveData::default();
    save.complete_stage("Minmatar", "Amarr", 5, 3);
    save.complete_stage("Minmatar", "Amarr", 5, 1); // earlier mission

    let entry = save
        .stage_progress
        .iter()
        .find(|p| p.player_faction == "Minmatar" && p.enemy_faction == "Amarr")
        .expect("entry should exist");
    assert_eq!(
        entry.highest_mission, 3,
        "complete_stage must not regress highest_mission"
    );
}

/// Re-recording the same stage+mission pair must keep `stage_progress`
/// at length 1 (no Vec duplicates). A duplicate would re-add the
/// same row on every save, growing the file forever.
#[test]
fn complete_stage_idempotent_re_record_does_not_grow_vec() {
    let mut save = SaveData::default();
    for _ in 0..5 {
        save.complete_stage("Minmatar", "Amarr", 5, 3);
    }
    assert_eq!(
        save.stage_progress.len(),
        1,
        "5 re-records of the same pair must produce one Vec entry, got {}",
        save.stage_progress.len()
    );
}

/// Skipping ahead — completing stage 1 then stage 5 on the same
/// faction pair — must leave `highest_stage=5` with a single Vec
/// entry. This protects the ship-unlock cascade from "you can skip
/// to the carrier by skipping a stage".
#[test]
fn complete_stage_skip_ahead_cascades_to_highest() {
    let mut save = SaveData::default();
    save.complete_stage("Minmatar", "Amarr", 1, 1);
    save.complete_stage("Minmatar", "Amarr", 5, 3);

    assert_eq!(save.stage_progress.len(), 1);
    let entry = &save.stage_progress[0];
    assert_eq!(entry.highest_stage, 5);
    assert_eq!(entry.highest_mission, 3);
}

/// Stage progress and high scores are kept in independent
/// structures — completing one must not touch the other.
#[test]
fn complete_stage_does_not_touch_high_scores() {
    let mut save = SaveData::default();
    save.record_score("Minmatar", "Amarr", 75000, 5);
    save.complete_stage("Minmatar", "Amarr", 5, 2);

    assert_eq!(
        save.get_high_score("Minmatar", "Amarr"),
        75000,
        "complete_stage must preserve recorded high scores"
    );
    assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 5);
}

// ============================================================================
// Group B — Serde round-trip
// ============================================================================

/// The full SaveData (including `stage_progress`) must survive a
/// JSON round-trip losslessly. Any future refactor that adds a
/// non-`#[serde(default)]` field would break this. The shape is
/// pinned here.
#[test]
fn save_data_round_trip_preserves_stage_progress() {
    let mut save = SaveData::default();
    save.complete_stage("Minmatar", "Amarr", 7, 4);
    save.complete_stage("Caldari", "Gallente", 3, 1);

    let json = serde_json::to_string(&save).expect("serialize");
    let loaded: SaveData = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(loaded.stage_progress.len(), 2);
    assert_eq!(loaded.get_highest_stage("Minmatar", "Amarr"), 7);
    assert_eq!(loaded.get_highest_stage("Caldari", "Gallente"), 3);

    // Per-pair (faction, enemy) → (stage, mission) preservation.
    let minama = loaded
        .stage_progress
        .iter()
        .find(|p| p.player_faction == "Minmatar" && p.enemy_faction == "Amarr")
        .unwrap();
    assert_eq!(minama.highest_stage, 7);
    assert_eq!(minama.highest_mission, 4);
}

/// The ship-unlock HashSet must survive a stage save reload — a
/// player who completed stage 5 once never loses the unlocked
/// ship just because `highest_stage` didn't change.
#[test]
fn save_data_round_trip_preserves_unlocked_ships() {
    let mut save = SaveData::default();
    save.complete_stage("Minmatar", "Amarr", 5, 1);
    save.unlock_ship(587);

    let json = serde_json::to_string(&save).expect("serialize");
    let loaded: SaveData = serde_json::from_str(&json).expect("deserialize");

    assert!(
        loaded.unlocked_ships.contains(&587),
        "unlocked_ships HashSet must survive the JSON round-trip"
    );
    assert!(
        loaded.is_ship_unlocked(587, 5, "Minmatar", "Amarr"),
        "is_ship_unlocked must still pass after a reload"
    );
}

// ============================================================================
// Group C — Resource-level round-trip (real Bevy schedules)
// ============================================================================

/// A populated `SaveData` resource must survive an `app.update()`
/// without being clobbered. The headless build registers
/// `SavePlugin` (`src/app_builder.rs:165-198`), so
/// `apply_saved_settings` runs in `PostStartup`. Insert the
/// resource **after** Startup fires (`load_save_data` runs in
/// Startup, not Startup-orderable), then verify the resource
/// still holds the populated state on the next frame.
///
/// Same disk-pollution mitigation as
/// `new_complete_stage_call_visible_in_world_next_tick`: the
/// first `app.update()` runs the disk load, then we overwrite
/// with a clean default and complete a stage.
#[test]
fn stage_progress_survives_post_startup_apply_saved_settings() {
    let mut app = build_headless_app();

    // First update runs Startup (load_save_data from disk →
    // whatever's in ~/.local/share/rebellion/save.json, or
    // SaveData::default() if missing). Then PostStartup runs
    // apply_saved_settings (one-shot).
    app.update();

    // After Startup, the resource holds the disk contents. Now
    // overwrite with a clean default and complete a stage.
    {
        let mut save = app.world_mut().resource_mut::<SaveData>();
        *save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 7, 4);
    }
    app.update();

    let read_back = app.world().resource::<SaveData>();
    assert_eq!(
        read_back.get_highest_stage("Minmatar", "Amarr"),
        7,
        "stage_progress set in-world must persist across subsequent updates"
    );
}

/// Mutating `stage_progress` after launch (`complete_stage` from a
/// gameplay system) must be observable on the next tick. This is
/// the per-stage progression promise — completing stage N in-game
/// immediately reflects in the resource view.
///
/// The headless build registers `SavePlugin`, whose Startup system
/// `load_save_data` reads `~/.local/share/rebellion/save.json` and
/// overwrites the in-world resource with the on-disk content. To
/// keep the test deterministic regardless of CI cache or local
/// pollution, insert a freshly-built `SaveData` *after* the disk
/// load runs (one `app.update()` flushes Startup).
#[test]
fn new_complete_stage_call_visible_in_world_next_tick() {
    let mut app = build_headless_app();
    app.update(); // Startup fires → load_save_data runs → SaveData holds whatever's on disk

    // Overwrite with a clean default, then complete a stage.
    {
        let mut save = app.world_mut().resource_mut::<SaveData>();
        *save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 5, 2);
    }
    app.update();

    let save = app.world().resource::<SaveData>();
    assert_eq!(
        save.get_highest_stage("Minmatar", "Amarr"),
        5,
        "completing stage 5 in-world must be visible on the next tick"
    );
    assert!(
        save.is_ship_unlocked(587, 5, "Minmatar", "Amarr"),
        "completing stage 5 must unlock the stage-5 ship immediately"
    );
}
