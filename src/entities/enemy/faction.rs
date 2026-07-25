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
        // Triglavian - Crimson (Damavik, Vedmak, Leshak, Kikimora, Drekavac, Nergal)
        47269 | 47270 | 47271 | 49710 | 49711 | 52250 => COLOR_TRIGLAVIAN,
        _ => Color::srgb(0.5, 0.5, 0.5),
    }
}

/// Get engine trail for faction based on type_id
pub(super) fn get_faction_engine_trail(type_id: u32) -> EngineTrail {
    match type_id {
        // Amarr - golden engines (frigates, destroyers, battlecruisers)
        597 | 589 | 591 | 16236 | 24690 | 624 | 2006 | 11393 | 34317 | 12019 => {
            EngineTrail::amarr()
        }
        // Caldari - blue engines
        603 | 602 | 583 | 16238 | 24688 | 11381 | 11387 | 35683 | 621 => EngineTrail::caldari(),
        // Gallente - green engines
        593 | 594 | 608 | 16242 | 24700 | 11373 | 35685 => EngineTrail::gallente(),
        // Minmatar - rust engines
        587 | 585 | 598 | 11400 | 11993 | 11371 => EngineTrail::minmatar(),
        // EDENCOM — Vorton arc trails
        54731..=54733 => EngineTrail::edencom(),
        // Triglavian — entropic crimson
        47269 | 47270 | 47271 | 49710 | 49711 | 52250 | 52252 | 52254 => EngineTrail::triglavian(),
        // Guristas pirate (Gila) — violet
        17713 => EngineTrail::pirate(),
        _ => EngineTrail::amarr(),
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
        54731 | 54733 | 54732 | 47269 | 47270 | 47271 | 49710 | 49711 | 52250 | 17713 => {
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
        602 | 583 | 11381 | 35683 | 621 | 24688 => WeaponType::MissileLauncher,
        // Caldari hybrid hulls (Merlin, Cormorant) — railguns
        603 | 16238 => WeaponType::Railgun,
        // Amarr laser hulls (Punisher, Executioner, Tormentor, Coercer,
        // Harbinger, Retribution AF, Confessor T3)
        597 | 589 | 591 | 16236 | 24690 | 11393 | 34317 => WeaponType::Laser,
        // Amarr missile hull (Sacrilege HAMs — canon exception)
        12019 => WeaponType::MissileLauncher,
        // Gallente drone/blaster hulls
        593 | 594 | 608 | 16242 | 24700 | 11373 | 11377 | 35685 => WeaponType::Drone,
        // Minmatar autocannon hulls (Rifter, Slasher, Breacher, Jaguar AF)
        587 | 585 | 598 | 11400 => WeaponType::Autocannon,
        // Minmatar Muninn — user-configured missile boat
        11993 => WeaponType::MissileLauncher,
        // Gila — pirate cruiser; no drones in this build, fires rapid missiles
        17713 => WeaponType::MissileLauncher,
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
    use std::f32::consts::{FRAC_PI_2, PI};
    match type_id {
        // === CALDARI === (bundled sprites face up, need 180deg base only)
        602 => -FRAC_PI_2, // Kestrel — observed facing right, rotate -90° to face down
        603 => -FRAC_PI_2, // Merlin - faces left
        583 => -FRAC_PI_2, // Condor - faces left
        // Auto-detected via scripts/analyze_ship_orientation.py (PCA + CCP 3/4
        // render convention). Update with preview mode if a sprite rotates off.
        11381 => 0.0,       // Hawk — hand-drawn top-down, nose up
        11387 => FRAC_PI_2, // Harpy - legacy sprite
        35683 => 0.0,       // Jackdaw — hand-drawn top-down

        // === INVASION ROSTER === (custom sprites from eve-ship-sprites)
        // Triglavian + Nergal: natively nose-up, no correction.
        47270 => 0.0, // Vedmak
        47271 => 0.0, // Leshak
        49711 => 0.0, // Drekavac
        52250 => 0.0, // Nergal
        // EDENCOM sprites already top-down — no correction needed.
        // Previous -90° rotation made the hull face right in-game.
        54731 => 0.0, // Skybreaker
        54732 => 0.0, // Stormbringer
        54733 => 0.0, // Thunderchild
        // Kikimora same — native top-down.
        49710 => 0.0, // Kikimora
        // Amarr + Caldari cruisers have tapered bow at BOTTOM in sprite — flip.
        11393 => PI, // Retribution
        34317 => PI, // Confessor
        12019 => PI, // Sacrilege
        621 => PI,   // Caracal
        // Muninn top-heavy body with tail taper at bottom — nose up natively.
        11993 => 0.0, // Muninn
        // Gila procedural — drawn nose-up.
        17713 => 0.0, // Gila

        // === GALLENTE === (most render sideways)
        593 => -FRAC_PI_2,  // Tristan — observed facing up, flip 180°
        594 => FRAC_PI_2,   // Incursus - faces right
        608 => FRAC_PI_2,   // Atron - faces right
        11373 => FRAC_PI_2, // Enyo - assault frigate
        11377 => FRAC_PI_2, // Ishkur - assault frigate
        35685 => FRAC_PI_2, // Hecate - tactical destroyer

        // === DESTROYERS ===
        16236 => FRAC_PI_2, // Coercer (Amarr)
        16238 => FRAC_PI_2, // Cormorant (Caldari)
        16242 => PI,        // Catalyst (Gallente) — observed facing right, flip

        // === BATTLECRUISERS ===
        24688 => FRAC_PI_2, // Drake (Caldari)
        24690 => FRAC_PI_2, // Harbinger (Amarr)
        24700 => FRAC_PI_2, // Myrmidon (Gallente)

        // === AMARR ===
        597 => std::f32::consts::PI, // Punisher - faces down, flip 180deg
        591 => -FRAC_PI_2,           // Tormentor - refetched sprite; -90° per analyzer
        // 589 (Executioner) - faces up

        // === MINMATAR ===
        587 => std::f32::consts::PI, // Rifter - faces down, flip 180deg
        585 => std::f32::consts::PI, // Slasher - faces down, flip 180deg
        // 11400 Jaguar — refetched sprite naturally faces up; no correction
        // 598 (Breacher) - faces up, no rotation needed

        // === CARRIERS ===
        24483 => std::f32::consts::PI, // Nidhoggur (Minmatar) - needs 180deg flip
        23915 => std::f32::consts::PI, // Chimera (Caldari) - needs 180deg flip
        // 23757 (Archon), 23911 (Thanatos) - face correctly

        // Ships that already face up correctly
        _ => 0.0,
    }
}
