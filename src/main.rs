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

use rebellion::app_builder::RebellionAppConfig;

fn main() {
    // WASM: Set up panic hook for better error messages
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let mut app = RebellionAppConfig::native().build();

    // Setup
    app.add_systems(Startup, setup);

    app.run();
}

/// Initial game setup
fn setup(mut commands: Commands) {
    // Use 2D camera - sprites work reliably with this
    commands.spawn(Camera2d);

    info!("Rebellion initialized!");
}
