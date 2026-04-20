//! Custom Game Events
//!
//! Events for decoupled communication between systems.

#![allow(dead_code)]

use bevy::prelude::*;

/// Player took damage
#[derive(Event)]
pub struct PlayerDamagedEvent {
    pub damage: f32,
    pub damage_type: DamageType,
    pub source_position: Vec2,
}

/// Which defensive layer absorbed damage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageLayer {
    Shield,
    Armor,
    Hull,
}

/// Visual effect request for layer-specific damage
#[derive(Event)]
pub struct DamageLayerEvent {
    /// Position where damage occurred
    pub position: Vec2,
    /// Which layer was hit
    pub layer: DamageLayer,
    /// Amount of damage dealt to this layer
    pub damage: f32,
    /// Direction damage came from (for directional effects)
    pub direction: Vec2,
}

/// Enemy was destroyed
#[derive(Event)]
pub struct EnemyDestroyedEvent {
    pub position: Vec2,
    pub enemy_type: String,
    pub score_value: u64,
    pub was_boss: bool,
}

/// Player fired weapon
#[derive(Event)]
pub struct PlayerFireEvent {
    pub position: Vec2,
    pub direction: Vec2,
    pub weapon_type: WeaponType,
    pub bullet_color: Color,
    pub damage: f32,
    /// Number of projectiles to spawn (1 = normal, 3+ = burst)
    pub burst_count: u32,
    /// Spread angle in radians for burst fire (0 = parallel)
    pub spread_angle: f32,
    /// Ammo type (for autocannons)
    pub ammo_type: AmmoType,
    // Mod-driven overrides (from Inventory)
    pub crit_chance_override: Option<f32>,
    pub crit_mult_override: Option<f32>,
    pub pierce: u32,
    pub homing: f32,
    pub burn_dps: f32,
    pub chain_targets: u32,
}

/// Spawn enemy event
#[derive(Event)]
pub struct SpawnEnemyEvent {
    pub enemy_type: String,
    pub position: Vec2,
    pub spawn_pattern: SpawnPattern,
}

/// Spawn wave event
#[derive(Event)]
pub struct SpawnWaveEvent {
    pub wave_number: u32,
    pub enemy_count: u32,
    pub enemy_types: Vec<String>,
}

/// Stage completed
#[derive(Event)]
pub struct StageCompleteEvent {
    pub stage_number: u32,
    pub score: u64,
    pub time_taken: f32,
    pub refugees_rescued: u32,
}

/// Boss defeated
#[derive(Event)]
pub struct BossDefeatedEvent {
    pub boss_type: String,
    pub position: Vec2,
    pub score_value: u64,
}

/// Collectible picked up
#[derive(Event)]
pub struct CollectiblePickedUpEvent {
    pub collectible_type: CollectibleType,
    pub position: Vec2,
    pub value: u32,
}

/// Pickup visual effect request (separate from pickup logic for visual effects system)
#[derive(Event)]
pub struct PickupEffectEvent {
    pub position: Vec2,
    pub collectible_type: CollectibleType,
    pub color: Color,
}

/// Salt Miner mode activated
#[derive(Event)]
pub struct SaltMinerActivatedEvent;

/// Salt Miner mode ended
#[derive(Event)]
pub struct SaltMinerEndedEvent;

/// Screen shake request
#[derive(Event)]
pub struct ScreenShakeEvent {
    pub intensity: f32,
    pub duration: f32,
}

/// Explosion effect
#[derive(Event)]
pub struct ExplosionEvent {
    pub position: Vec2,
    pub size: ExplosionSize,
    pub color: Color,
}

/// Play sound effect
#[derive(Event)]
pub struct PlaySoundEvent {
    pub sound: SoundType,
    pub volume: f32,
}

// =============================================================================
// SUPPORTING TYPES
// =============================================================================

/// Damage types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    EM,        // Lasers, smartbombs
    Thermal,   // Lasers, some missiles
    Kinetic,   // Projectiles, railguns
    Explosive, // Missiles, artillery
}

/// Weapon types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    Autocannon,
    Artillery,
    Laser,
    Railgun,
    MissileLauncher,
    Drone,
    Disintegrator, // Triglavian entropic disintegrator (ramping damage)
    Vorton,        // EDENCOM vorton projector (chain lightning)
}

/// Ammo types for Minmatar autocannons (affects damage and fire rate)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(clippy::upper_case_acronyms)] // "EMP" is standard name as damage type name
pub enum AmmoType {
    /// Balanced ammo - no bonuses or penalties
    #[default]
    Sabot,
    /// Anti-shield ammo - +50% vs shields, -30% vs armor
    EMP,
    /// Anti-armor ammo - -30% vs shields, +50% vs armor
    Plasma,
    /// High-alpha ammo - +30% damage, -30% fire rate
    Fusion,
    /// Fast-tracking ammo - -10% damage, +40% fire rate
    Barrage,
}

impl AmmoType {
    /// Display name for HUD
    pub fn name(&self) -> &'static str {
        match self {
            AmmoType::Sabot => "SABOT",
            AmmoType::EMP => "EMP",
            AmmoType::Plasma => "PLASMA",
            AmmoType::Fusion => "FUSION",
            AmmoType::Barrage => "BARRAGE",
        }
    }

    /// Full display name
    pub fn full_name(&self) -> &'static str {
        match self {
            AmmoType::Sabot => "Titanium Sabot",
            AmmoType::EMP => "EMP",
            AmmoType::Plasma => "Phased Plasma",
            AmmoType::Fusion => "Fusion",
            AmmoType::Barrage => "Barrage",
        }
    }

    /// Projectile/tracer color
    pub fn color(&self) -> bevy::prelude::Color {
        use bevy::prelude::Color;
        match self {
            AmmoType::Sabot => Color::srgb(0.7, 0.7, 0.7), // Silver/gray
            AmmoType::EMP => Color::srgb(0.2, 0.6, 1.0),   // Blue
            AmmoType::Plasma => Color::srgb(1.0, 0.5, 0.0), // Orange
            AmmoType::Fusion => Color::srgb(1.0, 0.2, 0.2), // Red
            AmmoType::Barrage => Color::srgb(0.9, 0.8, 0.2), // Yellow
        }
    }

    /// Damage multiplier vs shields
    pub fn shield_mult(&self) -> f32 {
        match self {
            AmmoType::Sabot => 1.0,
            AmmoType::EMP => 1.5,
            AmmoType::Plasma => 0.7,
            AmmoType::Fusion => 1.3,
            AmmoType::Barrage => 0.9,
        }
    }

    /// Damage multiplier vs armor
    pub fn armor_mult(&self) -> f32 {
        match self {
            AmmoType::Sabot => 1.0,
            AmmoType::EMP => 0.7,
            AmmoType::Plasma => 1.5,
            AmmoType::Fusion => 1.3,
            AmmoType::Barrage => 0.9,
        }
    }

    /// Fire rate multiplier (higher = faster)
    pub fn fire_rate_mult(&self) -> f32 {
        match self {
            AmmoType::Sabot => 1.0,
            AmmoType::EMP => 1.0,
            AmmoType::Plasma => 1.0,
            AmmoType::Fusion => 0.7,
            AmmoType::Barrage => 1.4,
        }
    }

    /// Cycle to next ammo type
    pub fn next(&self) -> AmmoType {
        match self {
            AmmoType::Sabot => AmmoType::EMP,
            AmmoType::EMP => AmmoType::Plasma,
            AmmoType::Plasma => AmmoType::Fusion,
            AmmoType::Fusion => AmmoType::Barrage,
            AmmoType::Barrage => AmmoType::Sabot,
        }
    }

    /// Cycle to previous ammo type
    pub fn prev(&self) -> AmmoType {
        match self {
            AmmoType::Sabot => AmmoType::Barrage,
            AmmoType::EMP => AmmoType::Sabot,
            AmmoType::Plasma => AmmoType::EMP,
            AmmoType::Fusion => AmmoType::Plasma,
            AmmoType::Barrage => AmmoType::Fusion,
        }
    }

    /// Get ammo type from number key (1-5)
    pub fn from_number(n: u8) -> Option<AmmoType> {
        match n {
            1 => Some(AmmoType::Sabot),
            2 => Some(AmmoType::EMP),
            3 => Some(AmmoType::Plasma),
            4 => Some(AmmoType::Fusion),
            5 => Some(AmmoType::Barrage),
            _ => None,
        }
    }
}

/// Enemy spawn patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnPattern {
    Single,
    Line,
    VFormation,
    Circle,
    Random,
    Swarm,
}

/// Collectible types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectibleType {
    LiberationPod, // Freed slave - collect to liberate
    Credits,
    ShieldBoost,
    ArmorRepair,
    HullRepair,
    CapacitorCharge,
    Overdrive,       // Temporary speed boost
    DamageBoost,     // Temporary damage boost
    Invulnerability, // Temporary invincibility
    Nanite,          // Reduces weapon heat
    ExtraLife,
    SkillPointDrop, // SP currency drop
    // Persistent stacking weapon mods (additive, wipe on ship death)
    ScatterLauncher, // +projectile count, +spread, -dmg/shot
    RailSpike,       // +damage, -fire rate, +pierce
    PlasmaLance,     // +damage, +crit, +burn
    HomingSwarm,     // +projectile count, +homing
    VortonProjector,       // +chain targets, +damage
}

/// Upgrade types purchasable with Skill Points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Upgrade {
    /// +20 max shields
    ShieldBoost1,
    /// +40 max shields
    ShieldBoost2,
    /// +30 max armor
    ArmorPlate1,
    /// +60 max armor
    ArmorPlate2,
    /// +10% fire rate
    Gyrostabilizer1,
    /// +20% fire rate
    Gyrostabilizer2,
    /// +5 rocket capacity
    ExpandedRocketBay,
    /// +10% damage
    DamageAmplifier1,
    /// +20% damage
    DamageAmplifier2,
    /// +10% speed
    Afterburner,
    /// +25 max capacitor
    CapacitorBattery,
    /// Faster shield regen
    ShieldBooster,
}

impl Upgrade {
    /// Get all upgrades in purchase order
    pub fn all() -> &'static [Upgrade] {
        &[
            Upgrade::ShieldBoost1,
            Upgrade::ArmorPlate1,
            Upgrade::Gyrostabilizer1,
            Upgrade::ExpandedRocketBay,
            Upgrade::Afterburner,
            Upgrade::DamageAmplifier1,
            Upgrade::CapacitorBattery,
            Upgrade::ShieldBooster,
            Upgrade::ShieldBoost2,
            Upgrade::ArmorPlate2,
            Upgrade::Gyrostabilizer2,
            Upgrade::DamageAmplifier2,
        ]
    }

    /// Display name
    pub fn name(&self) -> &'static str {
        match self {
            Upgrade::ShieldBoost1 => "Reinforced Shielding I",
            Upgrade::ShieldBoost2 => "Reinforced Shielding II",
            Upgrade::ArmorPlate1 => "Alloy Armor Plating I",
            Upgrade::ArmorPlate2 => "Alloy Armor Plating II",
            Upgrade::Gyrostabilizer1 => "Gyrostabilizer I",
            Upgrade::Gyrostabilizer2 => "Gyrostabilizer II",
            Upgrade::ExpandedRocketBay => "Expanded Rocket Bay",
            Upgrade::DamageAmplifier1 => "Damage Amplifier I",
            Upgrade::DamageAmplifier2 => "Damage Amplifier II",
            Upgrade::Afterburner => "Afterburner",
            Upgrade::CapacitorBattery => "Capacitor Battery",
            Upgrade::ShieldBooster => "Shield Booster",
        }
    }

    /// Description
    pub fn description(&self) -> &'static str {
        match self {
            Upgrade::ShieldBoost1 => "Increase max shields by 20 points.",
            Upgrade::ShieldBoost2 => "Increase max shields by 40 more points.",
            Upgrade::ArmorPlate1 => "Reinforce armor with plates for +30 armor.",
            Upgrade::ArmorPlate2 => "Heavy armor plating for +60 more armor.",
            Upgrade::Gyrostabilizer1 => "Improve autocannon gyros for 10% faster fire rate.",
            Upgrade::Gyrostabilizer2 => "Advanced gyros for 20% more fire rate.",
            Upgrade::ExpandedRocketBay => "Install expanded rocket bays for 5 extra rockets.",
            Upgrade::DamageAmplifier1 => "Heat sinks allow 10% more damage output.",
            Upgrade::DamageAmplifier2 => "Overclocked amplifiers for 20% more damage.",
            Upgrade::Afterburner => "Microwarpdrive boost for 10% faster movement.",
            Upgrade::CapacitorBattery => "Extra battery banks for +25 capacitor.",
            Upgrade::ShieldBooster => "Active shield hardener for faster shield regen.",
        }
    }

    /// Cost in skill points
    pub fn cost(&self) -> u32 {
        match self {
            Upgrade::ShieldBoost1 => 50,
            Upgrade::ShieldBoost2 => 150,
            Upgrade::ArmorPlate1 => 50,
            Upgrade::ArmorPlate2 => 150,
            Upgrade::Gyrostabilizer1 => 75,
            Upgrade::Gyrostabilizer2 => 200,
            Upgrade::ExpandedRocketBay => 50,
            Upgrade::DamageAmplifier1 => 100,
            Upgrade::DamageAmplifier2 => 250,
            Upgrade::Afterburner => 75,
            Upgrade::CapacitorBattery => 60,
            Upgrade::ShieldBooster => 100,
        }
    }

    /// Prerequisite upgrade (if any)
    pub fn requires(&self) -> Option<Upgrade> {
        match self {
            Upgrade::ShieldBoost2 => Some(Upgrade::ShieldBoost1),
            Upgrade::ArmorPlate2 => Some(Upgrade::ArmorPlate1),
            Upgrade::Gyrostabilizer2 => Some(Upgrade::Gyrostabilizer1),
            Upgrade::DamageAmplifier2 => Some(Upgrade::DamageAmplifier1),
            _ => None,
        }
    }
}

/// Skill Points resource - persistent currency
#[derive(Resource, Debug, Clone, Default)]
pub struct SkillPoints {
    /// Current available SP
    pub available: u32,
}

/// Explosion sizes for visual effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionSize {
    Tiny,    // Bullet impact
    Small,   // Frigate explosion
    Medium,  // Cruiser explosion
    Large,   // Battleship explosion
    Massive, // Boss explosion
}

/// Sound effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundType {
    // Weapons
    Autocannon,
    Artillery,
    Laser,
    Missile,

    // Impacts
    ShieldHit,
    ArmorHit,
    HullHit,

    // Explosions
    SmallExplosion,
    MediumExplosion,
    LargeExplosion,

    // UI
    MenuSelect,
    MenuConfirm,
    MenuBack,

    // Gameplay
    PowerUp,
    Liberation, // Soul liberated
    SaltMinerActivate,
    Warning,
    Victory,
    GameOver,
}

/// Plugin to register all events
pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDamagedEvent>()
            .add_event::<DamageLayerEvent>()
            .add_event::<EnemyDestroyedEvent>()
            .add_event::<PlayerFireEvent>()
            .add_event::<SpawnEnemyEvent>()
            .add_event::<SpawnWaveEvent>()
            .add_event::<StageCompleteEvent>()
            .add_event::<BossDefeatedEvent>()
            .add_event::<CollectiblePickedUpEvent>()
            .add_event::<PickupEffectEvent>()
            .add_event::<SaltMinerActivatedEvent>()
            .add_event::<SaltMinerEndedEvent>()
            .add_event::<ScreenShakeEvent>()
            .add_event::<ExplosionEvent>()
            .add_event::<PlaySoundEvent>();
    }
}
