//! Native save transactions. Never truncate the only valid copy of progress.
use super::SaveData;
use bevy::prelude::*;
#[cfg(unix)]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

fn read_save(path: &Path) -> io::Result<SaveData> {
    serde_json::from_slice(&fs::read(path)?)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub(super) fn load(path: &Path) -> SaveData {
    match read_save(path) {
        Ok(save) => return save,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => warn!("Cannot read primary save {:?}: {}", path, error),
    }
    let backup = path.with_extension("json.bak");
    match read_save(&backup) {
        Ok(save) => {
            warn!(
                "Recovered progress from {:?}; primary left untouched",
                backup
            );
            save
        }
        Err(error) => {
            if error.kind() != io::ErrorKind::NotFound {
                warn!("Cannot read backup save {:?}: {}", backup, error);
            }
            SaveData::default()
        }
    }
}

/// Write a complete replacement beside its destination, sync it, then rename.
/// A failed write/rename leaves the destination intact and removes its temporary.
fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = path.with_file_name(format!("save.atomic-{}.tmp", nonce()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        #[cfg(unix)]
        if let Some(parent) = path.parent() {
            File::open(parent)?.sync_all()?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn nonce() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!(
        "{}-{timestamp}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
}

fn preserve_corrupt(path: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
    let preserved = path.with_file_name(format!("save.corrupt-{}.json", nonce()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&preserved)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(preserved)
}

pub(super) fn save(save: &SaveData, path: &Path) -> io::Result<()> {
    let data = serde_json::to_vec_pretty(save)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::read(path) {
        Ok(previous) => {
            if previous == data {
                return Ok(());
            }
            if serde_json::from_slice::<SaveData>(&previous).is_ok() {
                atomic_write(&path.with_extension("json.bak"), &previous)?;
            } else {
                // Startup may have recovered the backup or returned defaults. Keep
                // the damaged original before any subsequent autosave replaces it.
                let preserved = preserve_corrupt(path, &previous)?;
                warn!("Preserved unreadable save at {:?}", preserved);
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    atomic_write(path, &data)?;
    info!("Saved progress to {:?}", path);
    Ok(())
}
