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
        597 | 589 | 591 | 16236 | 24690 => COLOR_AMARR,
        // Caldari - Steel Blue (frigates, destroyers, battlecruisers)
        603 | 602 | 583 | 16238 | 24688 => COLOR_CALDARI,
        // Gallente - Green (frigates, destroyers, battlecruisers)
        593 | 594 | 608 | 16242 | 24700 => COLOR_GALLENTE,
        // Minmatar - Rust (frigates)
        587 | 585 | 598 => COLOR_MINMATAR,
        // Triglavian - Crimson (Damavik, Vedmak, Drekavac)
        47269..=47273 => COLOR_TRIGLAVIAN,
        _ => Color::srgb(0.5, 0.5, 0.5),
    }
}

/// Get engine trail for faction based on type_id
pub(super) fn get_faction_engine_trail(type_id: u32) -> EngineTrail {
    match type_id {
        // Amarr - golden engines (frigates, destroyers, battlecruisers)
        597 | 589 | 591 | 16236 | 24690 | 624 | 2006 | 11373 => EngineTrail::amarr(),
        // Caldari - blue engines (frigates, destroyers, battlecruisers)
        603 | 602 | 583 | 16238 | 24688 | 11381 | 11387 | 35683 => EngineTrail::caldari(),
        // Gallente - green engines (frigates, destroyers, battlecruisers)
        593 | 594 | 608 | 16242 | 24700 | 11371 | 35685 => EngineTrail::gallente(),
        // Minmatar - rust engines
        587 | 585 | 598 => EngineTrail::minmatar(),
        _ => EngineTrail::amarr(), // Default to Amarr (enemies)
    }
}

/// Get weapon type for faction based on type_id
pub(super) fn get_faction_weapon(type_id: u32) -> WeaponType {
    match type_id {
        // Amarr - Lasers (EM damage) - frigates, destroyers, battlecruisers
        597 | 589 | 591 | 16236 | 24690 => WeaponType::Laser,
        // Caldari - Railguns/Missiles (Kinetic/Explosive)
        603 | 16238 => WeaponType::Railgun, // Merlin, Cormorant
        602 | 583 | 24688 => WeaponType::MissileLauncher, // Kestrel, Condor, Drake
        // Gallente - Drones/Blasters (Thermal)
        593 | 594 | 608 | 16242 | 24700 => WeaponType::Drone,
        // Minmatar - Autocannons
        585 | 587 | 598 => WeaponType::Autocannon,
        // Triglavian - Disintegrators (ramping damage)
        47269 | 49710 | 47271 | 49711 | 47273 | 47466 | 56756 => WeaponType::Disintegrator,
        // EDENCOM - Vorton projectors (chain lightning)
        56757 | 56759 | 56760 => WeaponType::Vorton,
        _ => WeaponType::Laser,
    }
}

/// Get rotation correction for ships with non-standard orientations from CCP renders
/// Returns additional rotation in radians to apply on top of base rotation
pub fn get_ship_rotation_correction(type_id: u32) -> f32 {
    use std::f32::consts::FRAC_PI_2;
    match type_id {
        // === CALDARI === (bundled sprites face up, need 180deg base only)
        // 602 => 0.0,        // Kestrel - faces up, no extra correction
        603 => -FRAC_PI_2,  // Merlin - faces left
        583 => -FRAC_PI_2,  // Condor - faces left
        11381 => FRAC_PI_2, // Hawk - assault frigate
        11387 => FRAC_PI_2, // Harpy - assault frigate
        35683 => FRAC_PI_2, // Jackdaw - tactical destroyer

        // === GALLENTE === (most render sideways)
        593 => FRAC_PI_2,   // Tristan - faces right
        594 => FRAC_PI_2,   // Incursus - faces right
        608 => FRAC_PI_2,   // Atron - faces right
        11373 => FRAC_PI_2, // Enyo - assault frigate
        11377 => FRAC_PI_2, // Ishkur - assault frigate
        35685 => FRAC_PI_2, // Hecate - tactical destroyer

        // === DESTROYERS ===
        16236 => FRAC_PI_2,  // Coercer (Amarr)
        16238 => FRAC_PI_2,  // Cormorant (Caldari)
        16242 => -FRAC_PI_2, // Catalyst (Gallente) - faces left

        // === BATTLECRUISERS ===
        24688 => FRAC_PI_2, // Drake (Caldari)
        24690 => FRAC_PI_2, // Harbinger (Amarr)
        24700 => FRAC_PI_2, // Myrmidon (Gallente)

        // === AMARR ===
        597 => std::f32::consts::PI, // Punisher - faces down, flip 180deg
        591 => FRAC_PI_2,            // Tormentor - faces right
        // 589 (Executioner) - faces up

        // === MINMATAR ===
        587 => std::f32::consts::PI, // Rifter - faces down, flip 180deg
        585 => std::f32::consts::PI, // Slasher - faces down, flip 180deg
        // 598 (Breacher) - faces up, no rotation needed

        // === CARRIERS ===
        24483 => std::f32::consts::PI, // Nidhoggur (Minmatar) - needs 180deg flip
        23915 => std::f32::consts::PI, // Chimera (Caldari) - needs 180deg flip
        // 23757 (Archon), 23911 (Thanatos) - face correctly

        // Ships that already face up correctly
        _ => 0.0,
    }
}
