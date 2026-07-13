//! Powerup Icon Loading
//!
//! Loads powerup icons from the assets/powerups directory.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::core::CollectibleType;

/// Powerup icons plugin
pub struct PowerupIconsPlugin;

impl Plugin for PowerupIconsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PowerupIconCache>()
            .add_systems(Startup, load_powerup_icons);
    }
}

/// Cache of loaded powerup icon handles
#[derive(Resource, Default)]
pub struct PowerupIconCache {
    /// Map of collectible type -> texture handle
    pub icons: HashMap<CollectibleType, Handle<Image>>,
}

impl PowerupIconCache {
    /// Get icon for a collectible type
    pub fn get(&self, collectible_type: &CollectibleType) -> Option<Handle<Image>> {
        self.icons.get(collectible_type).cloned()
    }
}

/// Map collectible types to icon filenames
fn get_icon_filename(collectible_type: &CollectibleType) -> Option<&'static str> {
    match collectible_type {
        CollectibleType::ShieldBoost => Some("shield_hardener.png"),
        CollectibleType::ArmorRepair => Some("armor_hardener.png"),
        CollectibleType::HullRepair => Some("reinforced_bulkheads.png"),
        CollectibleType::Overdrive => Some("microwarpdrive.png"),
        CollectibleType::DamageBoost => Some("combat_booster.png"),
        CollectibleType::Invulnerability => Some("assault_damage_control.png"),
        CollectibleType::Nanite => Some("nanite_paste.png"),
        CollectibleType::ExtraLife => Some("speed_booster.png"),
        // Persistent weapon-mods — faction-authentic module icons
        CollectibleType::ScatterLauncher => Some("scourge_rage_missile.png"),
        CollectibleType::RailSpike => Some("republic_fleet_barrage.png"),
        CollectibleType::PlasmaLance => Some("conflagration_pulse.png"),
        CollectibleType::HomingSwarm => Some("warrior_drone_swarm.png"),
        CollectibleType::VortonProjector => Some("vorton_projector.png"),
        _ => None, // Credits, Refugee, Capacitor use simple shapes
    }
}

/// Load powerup icons from assets directory
/// Load powerup icons via Bevy's AssetServer. Works on both native and
/// WASM — the previous implementation used `std::env::current_dir()`
/// and `fs::read`, which silently fails on WASM (no filesystem). On the
/// browser the icons rendered as colored placeholder rects because
/// the cache was empty.
fn load_powerup_icons(mut cache: ResMut<PowerupIconCache>, asset_server: Res<AssetServer>) {
    let types = [
        CollectibleType::ShieldBoost,
        CollectibleType::ArmorRepair,
        CollectibleType::HullRepair,
        CollectibleType::Overdrive,
        CollectibleType::DamageBoost,
        CollectibleType::Invulnerability,
        CollectibleType::Nanite,
        CollectibleType::ExtraLife,
        CollectibleType::ScatterLauncher,
        CollectibleType::RailSpike,
        CollectibleType::PlasmaLance,
        CollectibleType::HomingSwarm,
        CollectibleType::VortonProjector,
    ];

    for collectible_type in types {
        if let Some(filename) = get_icon_filename(&collectible_type) {
            let path = format!("powerups/{}", filename);
            let handle: Handle<Image> = asset_server.load(&path);
            cache.icons.insert(collectible_type, handle);
        }
    }

    info!(
        "Queued {} powerup icons for load via AssetServer",
        cache.icons.len()
    );
}
