//! Integration tests for the disk cycle of `SaveData::save` /
//! `load`.
//!
//! Closes a real gap: the in-SaveData tests in
//! `core/save.rs:812` cover the *serde* path (JSON → JSON object)
//! but not the *disk* path (SaveData::save() writes a real file
//! at `save_path()`, then SaveData::load() reads it back). The
//! disk path is what `SavePlugin`'s `auto_save` and `load_save_data`
//! systems hit on every session boundary.
//!
//! Test isolation: the `REBELLION_HOME` env var (added in
//! `core/save.rs:158-168`) redirects `save_path()` away from the
//! user's real `~/.local/share/rebellion/save.json`. Each test
//! uses a unique subdirectory under `std::env::temp_dir()` so
//! parallel `cargo test` runs don't collide.

use rebellion::core::SaveData;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// One-up counter shared across all tests in this file. Combined
/// with `std::process::id()` and the test name, gives a unique
/// temp directory per test invocation even under `cargo test
/// --test integration_save_disk` parallelism.
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Serialize tests that mutate the process-global `REBELLION_HOME`
/// env var. `std::env::set_var` is racy across threads (readers
/// can see partial writes), so all tests in this file that touch
/// the override must acquire this lock — both when installing the
/// override and while the test runs. Tests that don't mutate the
/// env var can ignore it.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// RAII guard for the env-var override. Drops the override on
/// scope exit (test cleanup) so the next test can set its own.
/// The temp directory it points at is also removed, but only if
/// it's the one we created — never touches anything else.
struct SaveHomeGuard {
    /// Held for the lifetime of the guard — released on Drop.
    /// Holding the lock for the whole test (not just install)
    /// is what makes the test deterministic under cargo test's
    /// default parallel test runner.
    _lock: std::sync::MutexGuard<'static, ()>,
    original: Option<String>,
    dir: PathBuf,
}

impl SaveHomeGuard {
    fn new(test_name: &str) -> Self {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let pid = std::process::id();
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = env::temp_dir()
            .join(format!("rebellion_test_{pid}_{n}_{test_name}"));
        fs::create_dir_all(&dir).expect("create temp test dir");
        let original = env::var("REBELLION_HOME").ok();
        // SAFETY: env::set_var is documented as unsafe in 1.84+
        // only when concurrent readers can race — we hold the
        // ENV_LOCK mutex for the guard's lifetime, which is
        // longer than any concurrent reader's view. If Rust
        // upgrades this to unconditional `unsafe`, add an
        // `unsafe` block here.
        env::set_var("REBELLION_HOME", &dir);
        Self {
            _lock,
            original,
            dir,
        }
    }
}

impl Drop for SaveHomeGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(v) => env::set_var("REBELLION_HOME", v),
            None => env::remove_var("REBELLION_HOME"),
        }
        let _ = fs::remove_dir_all(&self.dir);
    }
}

// ============================================================================
// Group A — Disk cycle round-trip
// ============================================================================

/// Round-trip a populated `SaveData` through disk. This is the
/// path `SavePlugin::auto_save` runs every time a resource
/// changes, and the path `SavePlugin::load_save_data` runs at
/// startup. If it doesn't round-trip, every saved session is
/// effectively lost.
#[test]
fn save_then_load_round_trips_through_disk() {
    let _guard = SaveHomeGuard::new("round_trip");

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
}

/// Auto-save fires whenever the `SaveData` resource changes.
/// Verify the actual file is written (not just that `save()`
/// returns silently) and that subsequent `load()` returns the
/// mutated contents.
#[test]
fn auto_save_writes_file_visible_to_load() {
    let _guard = SaveHomeGuard::new("auto_save");

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
}

// ============================================================================
// Group B — Default and missing-file paths
// ============================================================================

/// `SaveData::load()` returns `default()` when the file is missing
/// rather than panicking. The `REBELLION_HOME` override points at
/// a guaranteed-empty tempdir, so the path "no file → default"
/// is the only reachable behavior.
#[test]
fn load_returns_default_when_save_file_missing() {
    let _guard = SaveHomeGuard::new("missing_file");

    // load() against an empty dir → defaults.
    let prior = SaveData::load();
    assert_eq!(
        prior.get_highest_stage("Minmatar", "Amarr"),
        0,
        "missing save must produce default SaveData"
    );
    assert_eq!(
        prior.lifetime_credits, 0,
        "default lifetime_credits must be zero"
    );

    // Sanity: no save.json was created — `load()` doesn't write.
    // Resolve the expected path the same way save_path() does.
    let root = env::var("REBELLION_HOME").expect("guard set REBELLION_HOME");
    let path = PathBuf::from(root).join("save.json");
    assert!(
        !path.exists(),
        "load() must not write a file; got {path:?}"
    );
}

// ============================================================================
// Group C — Env-var override actually redirects the path
// ============================================================================

/// The production path is `$XDG_DATA_HOME/rebellion/save.json`
/// (or platform equivalent). Without the env-var override set,
/// `save()` and `load()` would use that path. With it set to a
/// tempdir, the same `save()`/`load()` cycle must NOT touch the
/// user's real save file. We can't easily assert "the real save
/// file is unchanged" (it's whatever the tester last played),
/// so we assert the inverse: the env-overridden path is what
/// actually got read and written.
#[test]
fn rebellion_home_override_routes_to_tempdir() {
    let _guard = SaveHomeGuard::new("override_routing");

    // Confirm the tempdir we set is what `save_path` resolves to.
    let root = env::var("REBELLION_HOME").expect("guard set REBELLION_HOME");
    let expected = PathBuf::from(root).join("save.json");

    let save = SaveData::default();
    save.save();

    assert!(
        expected.exists(),
        "REBELLION_HOME-redirected save must write to {expected:?}"
    );
}
