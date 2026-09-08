//! Rebellion - Arcade Space Shooter
//!
//! A Rust/Bevy space arcade game featuring 5 campaigns,
//! factional warfare mechanics, and procedural content.

// Bevy systems naturally have complex query types and many parameters
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;

// WASM: Enable better panic messages in browser console
#[cfg(target_arch = "wasm32")]
use console_error_panic_hook;
#[cfg(not(target_arch = "wasm32"))]
use std::{env, path::PathBuf};

use rebellion::app_builder::RebellionAppConfig;

fn main() {
    // WASM: Set up panic hook for better error messages
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    #[cfg(not(target_arch = "wasm32"))]
    configure_portable_asset_root();

    let mut app = RebellionAppConfig::native().build();

    // Setup
    app.add_systems(Startup, setup);

    app.run();
}

#[cfg(not(target_arch = "wasm32"))]
fn configure_portable_asset_root() {
    let exe_dir = match env::current_exe() {
        Ok(path) => path.parent().map(PathBuf::from),
        Err(err) => {
            eprintln!("[startup] could not resolve executable path: {err}");
            return;
        }
    };

    let Some(exe_dir) = exe_dir else {
        eprintln!("[startup] executable path has no parent directory");
        return;
    };

    // Prefer the macOS app bundle resource layout when present:
    // <.app>/Contents/MacOS/rebellion -> <.app>/Contents/Resources/assets
    let bundle_resources = exe_dir.join("..").join("Resources");
    let candidate_roots = [bundle_resources, exe_dir];

    for candidate in candidate_roots {
        let assets_dir = candidate.join("assets");
        if assets_dir.is_dir() {
            let asset_root = match assets_dir.parent() {
                Some(parent) => parent,
                None => &candidate,
            };
            if let Err(err) = env::set_current_dir(asset_root) {
                eprintln!(
                    "[startup] failed to switch working directory to {}: {err}",
                    asset_root.display()
                );
                continue;
            }

            if let Ok(root) = env::current_dir() {
                env::set_var("BEVY_ASSET_ROOT", root.to_string_lossy().to_string());
            }
            return;
        }
    }

    let current = match env::current_dir() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("[startup] could not read current directory: {err}");
            return;
        }
    };
    if current.join("assets").is_dir() {
        env::set_var("BEVY_ASSET_ROOT", current.to_string_lossy().to_string());
    }
}

/// Initial game setup
fn setup(mut commands: Commands) {
    // Use 2D camera - sprites work reliably with this
    commands.spawn(Camera2d);

    info!("Rebellion initialized!");
}
