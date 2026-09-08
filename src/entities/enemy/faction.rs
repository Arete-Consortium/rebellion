//! Faction Lookup Tables
//!
//! Maps enemy type IDs to faction-specific colors, engine trails, weapons, and rotation corrections.

use crate::core::*;
use crate::systems::EngineTrail;
use bevy::prelude::*;

/// Get faction color for enemy type
pub(super) fn get_enemy_color(type_id: u32) -> Color {
    match type_id {
        // Amarr - Gold (frigates, destroyers, battlecruisers)
        597 | 589 | 591 | 16236 | 24696 | 624 => COLOR_AMARR,
        // Caldari - Steel Blue (frigates, destroyers, battlecruisers)
        603 | 602 | 583 | 16238 | 24698 => COLOR_CALDARI,
        // Gallente - Green (frigates, destroyers, battlecruisers)
        593 | 594 | 608 | 16240 | 24700 => COLOR_GALLENTE,
        // Minmatar - Rust (frigates)
        587 | 585 | 598 | 622 => COLOR_MINMATAR,
        // Triglavian - Crimson (Damavik, Vedmak, Leshak, Kikimora, Drekavac, Nergal)
        47269 | 47270 | 47271 | 49710 | 49711 | 52250 => COLOR_TRIGLAVIAN,
        _ => Color::srgb(0.5, 0.5, 0.5),
    }
}

/// Get engine trail for faction based on type_id
pub(super) fn get_faction_engine_trail(type_id: u32) -> EngineTrail {
    match type_id {
        // Amarr - golden engines (frigates, destroyers, battlecruisers)
        597 | 589 | 591 | 16236 | 24696 | 624 | 2006 | 11393 | 34317 | 12019 => {
            EngineTrail::amarr()
        }
        // Caldari - blue engines
        603 | 602 | 583 | 16238 | 24698 | 11379 | 11381 | 34828 | 621 => EngineTrail::caldari(),
        // Gallente - green engines
        593 | 594 | 608 | 16240 | 24700 | 12044 | 35683 => EngineTrail::gallente(),
        // Minmatar - rust engines
        587 | 585 | 598 | 11400 | 11993 | 11371 => EngineTrail::minmatar(),
        // EDENCOM — Vorton arc trails
        54731..=54733 => EngineTrail::edencom(),
        // Triglavian — entropic crimson
        47269 | 47270 | 47271 | 49710 | 49711 | 52250 | 52252 | 52254 => EngineTrail::triglavian(),
        // Guristas pirate (Gila) — violet
        17715 => EngineTrail::pirate(),
        _ => EngineTrail::amarr(),
    }
}

/// Get weapon type for faction based on type_id
pub(super) fn get_faction_weapon(type_id: u32) -> WeaponType {
    match type_id {
        // Amarr - Lasers (EM damage) - frigates, destroyers, battlecruisers
        597 | 589 | 591 | 16236 | 24696 | 624 => WeaponType::Laser,
        // Caldari - Railguns/Missiles (Kinetic/Explosive)
        603 | 16238 | 11381 => WeaponType::Railgun, // Merlin, Cormorant
        602 | 583 | 24698 => WeaponType::MissileLauncher, // Kestrel, Condor, Drake
        // Gallente - Drones/Blasters (Thermal)
        593 | 594 | 608 | 16240 | 24700 => WeaponType::Drone,
        // Minmatar - Autocannons
        585 | 587 | 598 => WeaponType::Autocannon,
        // Triglavian — Disintegrators (Damavik, Vedmak, Leshak, Kikimora, Drekavac, Nergal)
        47269 | 47270 | 47271 | 49710 | 49711 | 52250 => WeaponType::Disintegrator,
        // EDENCOM - Vorton projectors (chain lightning)
        54731..=54733 => WeaponType::Vorton,
        _ => WeaponType::Laser,
    }
}

/// Faction tint applied to ship sprites (multiplicative). White = untinted,
/// preserves full CCP render colors. We use pure white for invasion hulls so
/// Skybreaker/Nergal/Gila show their canonical art. Empire ships get a
/// very light faction wash so they read as alliance without washing out.
pub fn ship_sprite_tint(type_id: u32, faction: Faction) -> Color {
    match type_id {
        // Invasion-era hulls — show CCP art untinted so the ships are
        // instantly recognizable (EDENCOM, Triglavian, Pirate lineages).
        54731 | 54733 | 54732 | 47269 | 47270 | 47271 | 49710 | 49711 | 52250 | 17715 => {
            Color::WHITE
        }
        // Empire ships get a subtle ~80% → faction tint so the hull still
        // looks factional but detail stays readable.
        _ => {
            let c = faction.primary_color().to_srgba();
            Color::srgb(c.red * 0.3 + 0.7, c.green * 0.3 + 0.7, c.blue * 0.3 + 0.7)
        }
    }
}

/// Canon EVE weapon family for player ships. Hull type drives weapon type
/// (cross-faction invasion rosters fire correctly), with fallback to the
/// player's faction default for base empire ships not listed here.
pub fn get_player_weapon_type(type_id: u32, faction: Faction) -> WeaponType {
    match type_id {
        // EDENCOM — Vorton projectors (chain lightning)
        54731..=54733 => WeaponType::Vorton,
        // Triglavian + derived — Entropic disintegrators (ramping damage)
        47269 | 47270 | 47271 | 49710 | 49711 | 52250 => WeaponType::Disintegrator,
        // Caldari missile hulls (Kestrel, Condor, Hawk rockets, Jackdaw, Caracal, Drake)
        602 | 583 | 11379 | 34828 | 621 | 24698 => WeaponType::MissileLauncher,
        // Caldari hybrid hulls (Merlin, Cormorant) — railguns
        603 | 16238 | 11381 => WeaponType::Railgun,
        // Amarr laser hulls (Punisher, Executioner, Tormentor, Coercer,
        // Harbinger, Retribution AF, Confessor T3)
        597 | 589 | 591 | 16236 | 24696 | 11393 | 34317 => WeaponType::Laser,
        // Amarr missile hull (Sacrilege HAMs — canon exception)
        12019 => WeaponType::MissileLauncher,
        // Gallente drone/blaster hulls
        593 | 594 | 608 | 16240 | 24700 | 12044 | 12042 | 35683 => WeaponType::Drone,
        // Minmatar autocannon hulls (Rifter, Slasher, Breacher, Jaguar AF)
        587 | 585 | 598 | 11400 => WeaponType::Autocannon,
        // Minmatar Muninn — user-configured missile boat
        11993 => WeaponType::MissileLauncher,
        // Gila — pirate cruiser; no drones in this build, fires rapid missiles
        17715 => WeaponType::MissileLauncher,
        // Fall back to faction default for anything unmapped
        _ => match faction {
            Faction::Amarr => WeaponType::Laser,
            Faction::Caldari => WeaponType::MissileLauncher,
            Faction::Gallente => WeaponType::Drone,
            Faction::Minmatar => WeaponType::Autocannon,
        },
    }
}

/// Get rotation correction for ships with non-standard orientations from CCP renders
/// Returns additional rotation in radians to apply on top of base rotation
pub fn get_ship_rotation_correction(type_id: u32) -> f32 {
    static ROTATIONS: std::sync::OnceLock<std::collections::HashMap<u32, f32>> =
        std::sync::OnceLock::new();
    *ROTATIONS
        .get_or_init(|| {
            #[derive(serde::Deserialize)]
            struct Manifest {
                ships: Vec<Hull>,
            }
            #[derive(serde::Deserialize)]
            struct Hull {
                type_id: u32,
                sprite: Facing,
            }
            #[derive(serde::Deserialize)]
            struct Facing {
                rotation_correction_degrees: f32,
            }
            let manifest: Manifest =
                serde_json::from_str(include_str!("../../../assets/ships/ship_manifest.json"))
                    .expect("bundled hull orientation manifest");
            manifest
                .ships
                .into_iter()
                .map(|h| (h.type_id, h.sprite.rotation_correction_degrees.to_radians()))
                .collect()
        })
        .get(&type_id)
        .unwrap_or(&0.0)
}
