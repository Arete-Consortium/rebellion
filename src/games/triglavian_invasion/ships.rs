//! Triglavian Invasion Ship Definitions
//!
//! Ship type IDs for both factions.

use bevy::prelude::*;

/// Ship pool for Triglavian Invasion module
#[derive(Resource, Default)]
pub struct TriglavianShips {
    pub initialized: bool,
}

// =============================================================================
// TRIGLAVIAN COLLECTIVE SHIPS
// Ship type IDs
// =============================================================================

/// Triglavian ship type IDs — verified via CCP ESI /universe/ids 2026-04.
pub mod triglavian {
    // Frigates
    pub const DAMAVIK: u32 = 47269;  // T1 Frigate
    pub const NERGAL: u32 = 52250;   // Assault Frigate

    // Destroyers
    pub const KIKIMORA: u32 = 49710; // T1 Destroyer
    pub const DRAUGUR: u32 = 52254;  // Command Destroyer

    // Cruisers
    pub const VEDMAK: u32 = 47270;   // T1 Cruiser
    pub const IKITURSA: u32 = 52252; // Heavy Assault Cruiser
    pub const RODIVA: u32 = 52253;   // Logistics Cruiser

    // Battlecruisers
    pub const DREKAVAC: u32 = 49711; // T1 Battlecruiser

    // Battleships
    pub const LESHAK: u32 = 47271;   // T1 Battleship
    pub const XORDAZH: u32 = 56756;  // World Ark (capital; unverified)

    // Special
    pub const ZIRNITRA: u32 = 52907; // Dreadnought

    // Drones
    pub const SVAROG: u32 = 47276; // Heavy drone platform
}

// =============================================================================
// EDENCOM SHIPS
// Ship type IDs
// =============================================================================

/// EDENCOM ship type IDs — verified via CCP ESI /universe/ids 2026-04.
pub mod edencom {
    // Frigates
    pub const SKYBREAKER: u32 = 54731; // Frigate

    // Cruisers
    pub const THUNDERCHILD: u32 = 54733; // Cruiser

    // Battleships
    pub const STORMBRINGER: u32 = 54732; // Battleship

    // Empire ships commonly used with EDENCOM
    // Amarr
    pub const PUNISHER: u32 = 597;
    pub const OMEN: u32 = 2006;
    pub const APOCALYPSE: u32 = 642;

    // Caldari
    pub const MERLIN: u32 = 603;
    pub const CARACAL: u32 = 621;
    pub const RAVEN: u32 = 638;

    // Gallente
    pub const INCURSUS: u32 = 592;
    pub const THORAX: u32 = 627;
    pub const MEGATHRON: u32 = 641;

    // Minmatar
    pub const RIFTER: u32 = 587;
    pub const STABBER: u32 = 622;
    pub const TEMPEST: u32 = 639;
}

// =============================================================================
// SHIP STATS
// =============================================================================

/// Get base stats for a Triglavian ship
pub fn get_triglavian_stats(type_id: u32) -> ShipStats {
    match type_id {
        triglavian::DAMAVIK => ShipStats {
            health: 80.0,
            speed: 200.0,
            fire_rate: 0.8,
            damage: 8.0,
        },
        triglavian::NERGAL => ShipStats {
            health: 120.0,
            speed: 180.0,
            fire_rate: 0.6,
            damage: 15.0,
        },
        triglavian::KIKIMORA => ShipStats {
            health: 100.0,
            speed: 170.0,
            fire_rate: 0.7,
            damage: 12.0,
        },
        triglavian::VEDMAK => ShipStats {
            health: 200.0,
            speed: 140.0,
            fire_rate: 1.0,
            damage: 20.0,
        },
        triglavian::IKITURSA => ShipStats {
            health: 280.0,
            speed: 120.0,
            fire_rate: 0.9,
            damage: 25.0,
        },
        triglavian::DREKAVAC => ShipStats {
            health: 350.0,
            speed: 100.0,
            fire_rate: 1.2,
            damage: 30.0,
        },
        triglavian::LESHAK => ShipStats {
            health: 600.0,
            speed: 70.0,
            fire_rate: 1.5,
            damage: 50.0,
        },
        triglavian::XORDAZH => ShipStats {
            health: 2000.0,
            speed: 30.0,
            fire_rate: 2.0,
            damage: 80.0,
        },
        _ => ShipStats::default(),
    }
}

/// Get base stats for an EDENCOM ship
pub fn get_edencom_stats(type_id: u32) -> ShipStats {
    match type_id {
        edencom::SKYBREAKER => ShipStats {
            health: 90.0,
            speed: 190.0,
            fire_rate: 0.5,
            damage: 10.0,
        },
        edencom::THUNDERCHILD => ShipStats {
            health: 220.0,
            speed: 130.0,
            fire_rate: 0.8,
            damage: 22.0,
        },
        edencom::STORMBRINGER => ShipStats {
            health: 550.0,
            speed: 75.0,
            fire_rate: 1.2,
            damage: 45.0,
        },
        // Empire ships
        edencom::RIFTER | edencom::MERLIN | edencom::PUNISHER | edencom::INCURSUS => ShipStats {
            health: 70.0,
            speed: 210.0,
            fire_rate: 0.6,
            damage: 7.0,
        },
        edencom::STABBER | edencom::CARACAL | edencom::OMEN | edencom::THORAX => ShipStats {
            health: 180.0,
            speed: 140.0,
            fire_rate: 0.9,
            damage: 18.0,
        },
        edencom::TEMPEST | edencom::RAVEN | edencom::APOCALYPSE | edencom::MEGATHRON => ShipStats {
            health: 500.0,
            speed: 80.0,
            fire_rate: 1.3,
            damage: 40.0,
        },
        _ => ShipStats::default(),
    }
}

/// Basic ship stats structure
#[derive(Clone, Debug)]
pub struct ShipStats {
    pub health: f32,
    pub speed: f32,
    pub fire_rate: f32,
    pub damage: f32,
}

impl Default for ShipStats {
    fn default() -> Self {
        Self {
            health: 100.0,
            speed: 150.0,
            fire_rate: 1.0,
            damage: 10.0,
        }
    }
}

// =============================================================================
// ENEMY SPAWN WEIGHTS
// =============================================================================

/// Enemy spawn weights for Triglavian enemies (when playing as EDENCOM)
pub fn triglavian_spawn_weights() -> Vec<(u32, u32)> {
    vec![
        (triglavian::DAMAVIK, 50),  // Common frigate
        (triglavian::KIKIMORA, 25), // Uncommon destroyer
        (triglavian::VEDMAK, 15),   // Rare cruiser
        (triglavian::DREKAVAC, 8),  // Rare battlecruiser
        (triglavian::LESHAK, 2),    // Very rare battleship
    ]
}

/// Enemy spawn weights for EDENCOM enemies (when playing as Triglavian)
pub fn edencom_spawn_weights() -> Vec<(u32, u32)> {
    vec![
        (edencom::RIFTER, 15),
        (edencom::MERLIN, 15),
        (edencom::PUNISHER, 10),
        (edencom::INCURSUS, 10),
        (edencom::SKYBREAKER, 20), // Common EDENCOM frigate
        (edencom::STABBER, 8),
        (edencom::CARACAL, 8),
        (edencom::THUNDERCHILD, 10), // EDENCOM cruiser
        (edencom::STORMBRINGER, 4),  // Rare EDENCOM battleship
    ]
}

// =============================================================================
// PLAYER SHIP PROGRESSION
// =============================================================================

/// Player ship progression for EDENCOM
pub fn edencom_player_ships() -> Vec<PlayerShip> {
    vec![
        PlayerShip {
            type_id: edencom::SKYBREAKER,
            name: "Skybreaker",
            unlock_stage: 0, // Starting ship
        },
        PlayerShip {
            type_id: edencom::THUNDERCHILD,
            name: "Thunderchild",
            unlock_stage: 3,
        },
        PlayerShip {
            type_id: edencom::STORMBRINGER,
            name: "Stormbringer",
            unlock_stage: 6,
        },
    ]
}

/// Player ship progression for Triglavian
pub fn triglavian_player_ships() -> Vec<PlayerShip> {
    vec![
        PlayerShip {
            type_id: triglavian::DAMAVIK,
            name: "Damavik",
            unlock_stage: 0, // Starting ship
        },
        PlayerShip {
            type_id: triglavian::VEDMAK,
            name: "Vedmak",
            unlock_stage: 3,
        },
        PlayerShip {
            type_id: triglavian::LESHAK,
            name: "Leshak",
            unlock_stage: 6,
        },
    ]
}

/// Player ship definition
#[derive(Clone, Debug)]
pub struct PlayerShip {
    pub type_id: u32,
    pub name: &'static str,
    pub unlock_stage: u32,
}
