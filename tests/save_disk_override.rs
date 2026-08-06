//! When `REBELLION_HOME` is set, `save()` must write to the
//! tempdir's `save.json` — not the platform data dir. Without
//! this, a misconfigured env var would silently save to the
//! user's real save file.
//!
//! Process isolation: separate binary. Within this binary,
//! `REBELLION_HOME` is unique and unobserved by other tests.

mod common;

use rebellion::core::SaveData;

#[test]
fn rebellion_home_override_routes_to_tempdir() {
    let dir = common::install_save_home("override_routing");

    // Confirm the tempdir we set is what `save_path` resolves to.
    let expected = common::resolved_save_path();
    assert_eq!(
        expected,
        dir.join("save.json"),
        "REBELLION_HOME must be honored by save_path()"
    );

    let save = SaveData::default();
    save.save();

    assert!(
        expected.exists(),
        "REBELLION_HOME-redirected save must write to {expected:?}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
