//! Ship definitions, classes, and per-faction ship pools.

/// Player ship definition
#[derive(Debug, Clone, Copy)]
pub struct ShipDef {
    pub type_id: u32,
    pub name: &'static str,
    pub class: ShipClass,
    pub role: &'static str,
    pub health: f32,
    pub speed: f32,
    pub fire_rate: f32,
    pub damage: f32,
    pub special: &'static str,
    pub unlock_stage: u32, // 0 = always available
}

/// Enemy ship definition
#[derive(Debug, Clone, Copy)]
pub struct EnemyShipDef {
    pub type_id: u32,
    pub name: &'static str,
    pub class: ShipClass,
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
    pub spawn_weight: u32,
    pub score: u32,
}

/// Ship class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipClass {
    Frigate,
    AssaultFrigate,
    Interceptor,
    Destroyer,
    TacticalDestroyer,
    Cruiser,
    Battlecruiser,
    Battleship,
}

impl ShipClass {
    pub fn name(&self) -> &'static str {
        match self {
            ShipClass::Frigate => "Frigate",
            ShipClass::AssaultFrigate => "Assault Frigate",
            ShipClass::Interceptor => "Interceptor",
            ShipClass::Destroyer => "Destroyer",
            ShipClass::TacticalDestroyer => "Tactical Destroyer",
            ShipClass::Cruiser => "Cruiser",
            ShipClass::Battlecruiser => "Battlecruiser",
            ShipClass::Battleship => "Battleship",
        }
    }

    /// Get sprite size for this ship class (in pixels)
    pub fn sprite_size(&self) -> f32 {
        use crate::core::constants::*;
        match self {
            ShipClass::Frigate => SIZE_FRIGATE,
            ShipClass::AssaultFrigate => SIZE_ASSAULT_FRIGATE,
            ShipClass::Interceptor => SIZE_INTERCEPTOR,
            ShipClass::Destroyer => SIZE_DESTROYER,
            ShipClass::TacticalDestroyer => SIZE_TACTICAL_DESTROYER,
            ShipClass::Cruiser => SIZE_CRUISER,
            ShipClass::Battlecruiser => SIZE_BATTLECRUISER,
            ShipClass::Battleship => SIZE_BATTLESHIP,
        }
    }
}

// ============================================================================
// MINMATAR SHIPS
// ============================================================================

pub const MINMATAR_SHIPS: &[ShipDef] = &[
    ShipDef {
        type_id: 587,
        name: "Rifter",
        class: ShipClass::Frigate,
        role: "Balanced Brawler",
        health: 100.0,
        speed: 350.0,
        fire_rate: 8.0,
        damage: 10.0,
        special: "Overdrive: +50% speed burst",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 585,
        name: "Slasher",
        class: ShipClass::Frigate,
        role: "Fast Interceptor",
        health: 70.0,
        speed: 420.0,
        fire_rate: 10.0,
        damage: 7.0,
        special: "Afterburner: Invulnerable dash",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 598,
        name: "Breacher",
        class: ShipClass::Frigate,
        role: "Rocket Specialist",
        health: 110.0,
        speed: 320.0,
        fire_rate: 4.0,
        damage: 18.0,
        special: "Rocket Barrage: Triple spread",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11371,
        name: "Wolf",
        class: ShipClass::AssaultFrigate,
        role: "Heavy Autocannon",
        health: 150.0,
        speed: 340.0,
        fire_rate: 12.0,
        damage: 15.0,
        special: "Gyrostabilizer: +100% fire rate",
        unlock_stage: 4, // Unlocks after Act 1
    },
    ShipDef {
        type_id: 11400,
        name: "Jaguar",
        class: ShipClass::AssaultFrigate,
        role: "Rocket Swarm",
        health: 140.0,
        speed: 380.0,
        fire_rate: 3.0,
        damage: 25.0,
        special: "Rocket Swarm: Tracking missiles",
        unlock_stage: 9, // Unlocks after Act 2
    },
];

pub const MINMATAR_ENEMIES: &[EnemyShipDef] = &[
    EnemyShipDef {
        type_id: 587,
        name: "Rifter",
        class: ShipClass::Frigate,
        health: 50.0,
        speed: 180.0,
        damage: 8.0,
        spawn_weight: 30,
        score: 100,
    },
    EnemyShipDef {
        type_id: 585,
        name: "Slasher",
        class: ShipClass::Frigate,
        health: 35.0,
        speed: 220.0,
        damage: 5.0,
        spawn_weight: 25,
        score: 75,
    },
    EnemyShipDef {
        type_id: 598,
        name: "Breacher",
        class: ShipClass::Frigate,
        health: 60.0,
        speed: 150.0,
        damage: 12.0,
        spawn_weight: 20,
        score: 125,
    },
];

// ============================================================================
// AMARR SHIPS
// ============================================================================

pub const AMARR_SHIPS: &[ShipDef] = &[
    ShipDef {
        type_id: 589,
        name: "Executioner",
        class: ShipClass::Frigate,
        role: "Laser Interceptor",
        health: 90.0,
        speed: 380.0,
        fire_rate: 6.0,
        damage: 12.0,
        special: "Scorch: Extended laser range",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 597,
        name: "Punisher",
        class: ShipClass::Frigate,
        role: "Armored Brawler",
        health: 140.0,
        speed: 300.0,
        fire_rate: 5.0,
        damage: 14.0,
        special: "Armor Hardener: -50% damage",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 591,
        name: "Tormentor",
        class: ShipClass::Frigate,
        role: "Drone Support",
        health: 100.0,
        speed: 340.0,
        fire_rate: 7.0,
        damage: 10.0,
        special: "Deploy Drone: Autonomous fighter",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11186,
        name: "Crusader",
        class: ShipClass::Interceptor,
        role: "Fast Strike",
        health: 80.0,
        speed: 450.0,
        fire_rate: 10.0,
        damage: 8.0,
        special: "Microwarpdrive: Extreme speed",
        unlock_stage: 4, // Unlocks after Act 1
    },
    ShipDef {
        type_id: 11184,
        name: "Malediction",
        class: ShipClass::Interceptor,
        role: "Rocket Interceptor",
        health: 75.0,
        speed: 440.0,
        fire_rate: 5.0,
        damage: 15.0,
        special: "Tackle: Slow enemies on hit",
        unlock_stage: 9, // Unlocks after Act 2
    },
];

pub const AMARR_ENEMIES: &[EnemyShipDef] = &[
    EnemyShipDef {
        type_id: 589,
        name: "Executioner",
        class: ShipClass::Frigate,
        health: 45.0,
        speed: 200.0,
        damage: 10.0,
        spawn_weight: 30,
        score: 100,
    },
    EnemyShipDef {
        type_id: 597,
        name: "Punisher",
        class: ShipClass::Frigate,
        health: 80.0,
        speed: 140.0,
        damage: 12.0,
        spawn_weight: 25,
        score: 150,
    },
    EnemyShipDef {
        type_id: 591,
        name: "Tormentor",
        class: ShipClass::Frigate,
        health: 55.0,
        speed: 170.0,
        damage: 8.0,
        spawn_weight: 20,
        score: 100,
    },
    EnemyShipDef {
        type_id: 16236,
        name: "Coercer",
        class: ShipClass::Destroyer,
        health: 120.0,
        speed: 120.0,
        damage: 18.0,
        spawn_weight: 15,
        score: 250,
    },
    EnemyShipDef {
        type_id: 24690,
        name: "Harbinger",
        class: ShipClass::Battlecruiser,
        health: 400.0,
        speed: 80.0,
        damage: 30.0,
        spawn_weight: 5,
        score: 500,
    },
];

// ============================================================================
// CALDARI SHIPS
// ============================================================================

pub const CALDARI_SHIPS: &[ShipDef] = &[
    ShipDef {
        type_id: 602,
        name: "Kestrel",
        class: ShipClass::Frigate,
        role: "Missile Boat",
        health: 95.0,
        speed: 340.0,
        fire_rate: 4.0,
        damage: 16.0,
        special: "Salvo: 4 missiles at once",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 603,
        name: "Merlin",
        class: ShipClass::Frigate,
        role: "Shield Brawler",
        health: 120.0,
        speed: 310.0,
        fire_rate: 6.0,
        damage: 11.0,
        special: "Shield Boost: Instant regen",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 583,
        name: "Condor",
        class: ShipClass::Frigate,
        role: "Fast Tackler",
        health: 70.0,
        speed: 420.0,
        fire_rate: 5.0,
        damage: 12.0,
        special: "Warp Disruptor: Slow enemies",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11381,
        name: "Hawk",
        class: ShipClass::AssaultFrigate,
        role: "Assault Missile",
        health: 130.0,
        speed: 330.0,
        fire_rate: 5.0,
        damage: 20.0,
        special: "Assault Launchers: +50% damage",
        unlock_stage: 4, // Unlocks after Act 1
    },
    ShipDef {
        type_id: 11387,
        name: "Harpy",
        class: ShipClass::AssaultFrigate,
        role: "Railgun Sniper",
        health: 110.0,
        speed: 350.0,
        fire_rate: 3.0,
        damage: 28.0,
        special: "Optimal Range: Bonus at distance",
        unlock_stage: 4, // Unlocks after Act 1
    },
    ShipDef {
        type_id: 35683,
        name: "Jackdaw",
        class: ShipClass::TacticalDestroyer,
        role: "Mode Switcher",
        health: 180.0,
        speed: 300.0,
        fire_rate: 6.0,
        damage: 22.0,
        special: "Mode Switch: Defense/Speed/Sniper",
        unlock_stage: 9, // Unlocks after Act 2
    },
];

pub const CALDARI_ENEMIES: &[EnemyShipDef] = &[
    EnemyShipDef {
        type_id: 602,
        name: "Kestrel",
        class: ShipClass::Frigate,
        health: 50.0,
        speed: 170.0,
        damage: 12.0,
        spawn_weight: 30,
        score: 100,
    },
    EnemyShipDef {
        type_id: 603,
        name: "Merlin",
        class: ShipClass::Frigate,
        health: 70.0,
        speed: 150.0,
        damage: 9.0,
        spawn_weight: 25,
        score: 125,
    },
    EnemyShipDef {
        type_id: 583,
        name: "Condor",
        class: ShipClass::Frigate,
        health: 40.0,
        speed: 220.0,
        damage: 8.0,
        spawn_weight: 25,
        score: 75,
    },
    EnemyShipDef {
        type_id: 16238,
        name: "Cormorant",
        class: ShipClass::Destroyer,
        health: 100.0,
        speed: 130.0,
        damage: 15.0,
        spawn_weight: 12,
        score: 200,
    },
    EnemyShipDef {
        type_id: 24688,
        name: "Drake",
        class: ShipClass::Battlecruiser,
        health: 450.0,
        speed: 70.0,
        damage: 25.0,
        spawn_weight: 5,
        score: 500,
    },
];

// ============================================================================
// GALLENTE SHIPS
// ============================================================================

pub const GALLENTE_SHIPS: &[ShipDef] = &[
    ShipDef {
        type_id: 593,
        name: "Tristan",
        class: ShipClass::Frigate,
        role: "Drone Boat",
        health: 100.0,
        speed: 340.0,
        fire_rate: 6.0,
        damage: 8.0,
        special: "Drones: 2 autonomous fighters",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 594,
        name: "Incursus",
        class: ShipClass::Frigate,
        role: "Armor Brawler",
        health: 130.0,
        speed: 320.0,
        fire_rate: 8.0,
        damage: 10.0,
        special: "Armor Repair: Heal over time",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 608,
        name: "Atron",
        class: ShipClass::Frigate,
        role: "Blaster Interceptor",
        health: 75.0,
        speed: 410.0,
        fire_rate: 12.0,
        damage: 6.0,
        special: "Close Range: +100% damage in melee",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11373,
        name: "Enyo",
        class: ShipClass::AssaultFrigate,
        role: "Heavy Blaster",
        health: 150.0,
        speed: 310.0,
        fire_rate: 10.0,
        damage: 14.0,
        special: "Void: Maximum damage ammo",
        unlock_stage: 4, // Unlocks after Act 1
    },
    ShipDef {
        type_id: 11377,
        name: "Ishkur",
        class: ShipClass::AssaultFrigate,
        role: "Assault Drones",
        health: 140.0,
        speed: 330.0,
        fire_rate: 5.0,
        damage: 10.0,
        special: "Heavy Drones: 3 strong fighters",
        unlock_stage: 4, // Unlocks after Act 1
    },
    ShipDef {
        type_id: 35685,
        name: "Hecate",
        class: ShipClass::TacticalDestroyer,
        role: "Mode Switcher",
        health: 160.0,
        speed: 320.0,
        fire_rate: 10.0,
        damage: 18.0,
        special: "Mode Switch: Defense/Speed/Sniper",
        unlock_stage: 9, // Unlocks after Act 2
    },
];

pub const GALLENTE_ENEMIES: &[EnemyShipDef] = &[
    EnemyShipDef {
        type_id: 593,
        name: "Tristan",
        class: ShipClass::Frigate,
        health: 55.0,
        speed: 170.0,
        damage: 7.0,
        spawn_weight: 30,
        score: 100,
    },
    EnemyShipDef {
        type_id: 594,
        name: "Incursus",
        class: ShipClass::Frigate,
        health: 75.0,
        speed: 160.0,
        damage: 9.0,
        spawn_weight: 25,
        score: 125,
    },
    EnemyShipDef {
        type_id: 608,
        name: "Atron",
        class: ShipClass::Frigate,
        health: 40.0,
        speed: 220.0,
        damage: 6.0,
        spawn_weight: 25,
        score: 75,
    },
    EnemyShipDef {
        type_id: 16242,
        name: "Catalyst",
        class: ShipClass::Destroyer,
        health: 90.0,
        speed: 140.0,
        damage: 20.0,
        spawn_weight: 12,
        score: 200,
    },
    EnemyShipDef {
        type_id: 24700,
        name: "Myrmidon",
        class: ShipClass::Battlecruiser,
        health: 380.0,
        speed: 85.0,
        damage: 22.0,
        spawn_weight: 5,
        score: 450,
    },
];
