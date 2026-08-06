//! `SaveData::load()` returns `default()` when the file is
//! missing rather than panicking. The `REBELLION_HOME` override
//! points at a guaranteed-empty tempdir, so the path "no file
//! → default" is the only reachable behavior.
//!
//! Also asserts that `load()` does **not** write a file as a
//! side effect — a writer in a "read" code path is a surprise
//! (especially if the tempdir is read-only).
//!
//! Process isolation: separate binary.

mod common;

use rebellion::core::SaveData;

#[test]
fn load_returns_default_when_save_file_missing() {
    let dir = common::install_save_home("missing_file");

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
    let path = common::resolved_save_path();
    assert!(
        !path.exists(),
        "load() must not write a file; got {path:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
