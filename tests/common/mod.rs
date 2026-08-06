//! Shared helpers for the per-process save-disk tests.
//!
//! Each test in `tests/save_disk_*.rs` is its own Cargo test
//! binary, so each runs in a fresh process that owns
//! `REBELLION_HOME` outright — no `Mutex`, no race. The
//! `tests/common/` directory is not picked up by Cargo as a
//! test target (only top-level `tests/*.rs` files are), so this
//! module is excluded from binary discovery automatically.
//!
//! Why per-process instead of a `Mutex`:
//!
//! - `std::env::set_var` is process-global. A `Mutex` serializes
//!   tests but the lock lives in the same process as the
//!   reader, so reader/writer pairs still race on visibility.
//!   Newer Rust (1.84+) marks `set_var` as `unsafe` for this
//!   reason.
//! - A subprocess that sets `REBELLION_HOME` only mutates its
//!   own env block. The parent's view of the env var is
//!   unchanged.
//! - Each `tests/save_disk_*.rs` is one Cargo test target, which
//!   compiles into one binary and runs in one process. Splitting
//!   the four scenarios into four files costs ~4 × 5 seconds of
//!   compile time, but eliminates the env-var race class
//!   entirely. The `dynamic_linking` Bevy dev-dep makes
//!   per-test binary compile much cheaper than release builds.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Set `REBELLION_HOME` to a fresh tempdir and return that
/// path. The caller is responsible for verifying the test ran
/// in isolation. We can't `Drop`-clean from a child process
/// easily (the test process exits and the OS reclaims the
/// tempdir), so the tempdir lives under `std::env::temp_dir()`
/// with a name that includes PID + a per-call nonced timestamp.
/// Tests that want deterministic cleanup can `fs::remove_dir_all`
/// themselves; the OS will reap leftover tempdirs on reboot.
pub fn install_save_home(test_name: &str) -> PathBuf {
    let pid = std::process::id();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = env::temp_dir()
        .join(format!("rebellion_save_disk_{pid}_{nonce}_{test_name}"));
    fs::create_dir_all(&dir).expect("create temp save home");
    // SAFETY: process-isolated — no other thread reads this
    // env var. If Rust upgrades `set_var` to unconditional
    // `unsafe`, wrap this in an `unsafe` block.
    env::set_var("REBELLION_HOME", &dir);
    dir
}

/// Resolve the save file path the way `save_path()` does:
/// `<REBELLION_HOME>/save.json` if the env var is set, else
/// fall through to `dirs::data_dir()`. Each test uses this to
/// assert on file existence / contents without depending on
/// the internal helper.
#[allow(dead_code)] // not every test binary uses this
pub fn resolved_save_path() -> PathBuf {
    match env::var("REBELLION_HOME") {
        Ok(root) => PathBuf::from(root).join("save.json"),
        Err(_) => dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rebellion")
            .join("save.json"),
    }
}

/// Silence the "unused import" warning that surfaces when a
/// test file only uses one of the helpers above.
#[allow(dead_code)]
pub fn _path_marker(p: &Path) -> &Path {
    p
}