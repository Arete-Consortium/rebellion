//! Exercise recovery through the same public API used by startup and autosave.
mod common;

use rebellion::core::SaveData;
use std::fs;

#[test]
fn corrupt_or_missing_primary_recovers_backup_without_destroying_evidence() {
    let dir = common::install_save_home("recovery");
    let primary = dir.join("save.json");
    let backup = dir.join("save.json.bak");
    let mut save = SaveData {
        lifetime_credits: 1250,
        ..Default::default()
    };
    save.unlock_ship(587);
    save.save();
    save.lifetime_credits = 2500;
    save.save();
    assert_eq!(SaveData::load().lifetime_credits, 2500);
    assert_eq!(
        serde_json::from_slice::<SaveData>(&fs::read(&backup).unwrap())
            .unwrap()
            .lifetime_credits,
        1250
    );

    let damaged = b"{\"lifetime_credits\":2500,";
    fs::write(&primary, damaged).unwrap();
    let recovered = SaveData::load();
    assert_eq!(recovered.lifetime_credits, 1250);
    assert!(recovered.unlocked_ships.contains(&587));
    assert_eq!(fs::read(&primary).unwrap(), damaged, "load is read-only");
    recovered.save();
    assert_eq!(SaveData::load().lifetime_credits, 1250);
    assert!(fs::read_dir(&dir).unwrap().any(|entry| {
        let path = entry.unwrap().path();
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("save.corrupt-")
            && fs::read(path).unwrap() == damaged
    }));

    fs::remove_file(&primary).unwrap();
    assert_eq!(SaveData::load().lifetime_credits, 1250);
    assert!(backup.is_file());

    // A write failure must not replace the previous good primary or backup.
    recovered.save();
    let previous = fs::read(&primary).unwrap();
    fs::remove_file(&backup).unwrap();
    fs::create_dir(&backup).unwrap();
    save.lifetime_credits = 9999;
    save.save();
    assert_eq!(fs::read(&primary).unwrap(), previous);
    assert!(!fs::read_dir(&dir).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")
    }));
    fs::remove_dir_all(dir).unwrap();
}
