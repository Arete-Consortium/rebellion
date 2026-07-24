//! Enemy Spawn Functions
//!
//! Spawn functions for each enemy type: generic, kamikaze, weaver, sniper, spawner, tank,
//! and all Triglavian variants.

#![allow(dead_code)]

use super::faction::*;
use super::types::*;
use crate::assets::ShipModelCache;
use crate::core::*;
use bevy::prelude::*;

/// Spawn a single enemy with 3D model, sprite, or fallback color
pub fn spawn_enemy(
    commands: &mut Commands,
    type_id: u32,
    position: Vec2,
    behavior: EnemyBehavior,
    sprite: Option<Handle<Image>>,
    _model_cache: Option<&ShipModelCache>,
) -> Entity {
    use crate::core::ShipClass;

    // Stats: (name, health, speed, score, ship_class)
    let (name, health, speed, score, ship_class) = match type_id {
        // === AMARR ===
        // Frigates
        597 => ("Punisher", 40.0, 80.0, 100, ShipClass::Frigate),
        589 => ("Executioner", 25.0, 120.0, 80, ShipClass::Frigate),
        591 => ("Tormentor", 35.0, 90.0, 90, ShipClass::Frigate),
        // Destroyer
        16236 => ("Coercer", 120.0, 65.0, 250, ShipClass::Destroyer),
        // Battlecruiser
        24690 => ("Harbinger", 400.0, 50.0, 500, ShipClass::Battlecruiser),

        // === CALDARI ===
        // Frigates
        603 => ("Merlin", 45.0, 70.0, 100, ShipClass::Frigate),
        602 => ("Kestrel", 30.0, 100.0, 90, ShipClass::Frigate),
        583 => ("Condor", 25.0, 130.0, 75, ShipClass::Frigate),
        // Destroyer
        16238 => ("Cormorant", 100.0, 70.0, 200, ShipClass::Destroyer),
        // Battlecruiser
        24688 => ("Drake", 450.0, 45.0, 500, ShipClass::Battlecruiser),

        // === GALLENTE ===
        // Frigates
        593 => ("Tristan", 35.0, 90.0, 100, ShipClass::Frigate),
        594 => ("Incursus", 40.0, 85.0, 95, ShipClass::Frigate),
        608 => ("Atron", 25.0, 130.0, 75, ShipClass::Frigate),
        // Destroyer
        16242 => ("Catalyst", 90.0, 75.0, 200, ShipClass::Destroyer),
        // Battlecruiser
        24700 => ("Myrmidon", 380.0, 55.0, 450, ShipClass::Battlecruiser),

        // === MINMATAR ===
        // Frigates
        587 => ("Rifter", 35.0, 100.0, 100, ShipClass::Frigate),
        585 => ("Slasher", 25.0, 130.0, 75, ShipClass::Frigate),
        598 => ("Breacher", 40.0, 90.0, 100, ShipClass::Frigate),

        // === TRIGLAVIAN === (verified CCP inventory_type IDs 2026-04)
        47269 => ("Damavik", 80.0, 100.0, 150, ShipClass::Frigate),
        47270 => ("Vedmak", 200.0, 70.0, 350, ShipClass::Cruiser),
        47271 => ("Leshak", 600.0, 40.0, 1000, ShipClass::Battleship),
        49710 => ("Kikimora", 100.0, 90.0, 200, ShipClass::Destroyer),
        49711 => ("Drekavac", 350.0, 50.0, 600, ShipClass::Battlecruiser),
        52250 => ("Nergal", 130.0, 110.0, 220, ShipClass::Frigate), // Assault frigate
        52252 => ("Ikitursa", 280.0, 60.0, 450, ShipClass::Cruiser), // HAC
        52254 => ("Draugur", 140.0, 95.0, 240, ShipClass::Destroyer), // Command destroyer

        // === EDENCOM === (verified CCP inventory_type IDs 2026-04)
        54731 => ("Skybreaker", 90.0, 95.0, 180, ShipClass::Frigate),
        54732 => ("Stormbringer", 550.0, 45.0, 900, ShipClass::Battleship),
        54733 => ("Thunderchild", 220.0, 65.0, 400, ShipClass::Cruiser),

        // Unknown - default to frigate size
        _ => ("Unknown", 30.0, 100.0, 50, ShipClass::Frigate),
    };

    // Get sprite size from ship class
    let sprite_size = ship_class.sprite_size();

    let base_color = get_enemy_color(type_id);
    let weapon_type = get_faction_weapon(type_id);

    // Configure weapon based on faction
    let weapon = EnemyWeapon {
        weapon_type,
        fire_rate: match weapon_type {
            WeaponType::Laser => 0.8,           // Amarr: Slower, harder hitting
            WeaponType::Railgun => 0.6,         // Caldari: Slow but powerful
            WeaponType::MissileLauncher => 0.5, // Caldari missiles: Slowest
            WeaponType::Drone => 1.2,           // Gallente: Fast drones
            WeaponType::Autocannon => 1.5,      // Minmatar: Fastest
            WeaponType::Disintegrator => 0.0, // Triglavian: Continuous beam (uses DisintegratorRamp)
            WeaponType::Vorton => 0.7,        // EDENCOM: Chain lightning
            _ => 1.0,
        },
        damage: match weapon_type {
            WeaponType::Laser => 12.0,
            WeaponType::Railgun => 18.0,
            WeaponType::MissileLauncher => 20.0,
            WeaponType::Drone => 8.0,
            WeaponType::Autocannon => 10.0,
            WeaponType::Disintegrator => 0.0, // Handled by DisintegratorRamp component
            WeaponType::Vorton => 15.0,       // Chain bounces deal less per hit
            _ => 10.0,
        },
        bullet_speed: match weapon_type {
            WeaponType::Laser => 280.0,           // Fast beams
            WeaponType::Railgun => 350.0,         // Fastest projectiles
            WeaponType::MissileLauncher => 180.0, // Slow missiles
            WeaponType::Drone => 200.0,           // Medium
            WeaponType::Autocannon => 250.0,      // Fast bullets
            WeaponType::Disintegrator => 0.0,     // Instant (beam)
            WeaponType::Vorton => 400.0,          // Fast lightning
            _ => 200.0,
        },
        cooldown: 0.5 + fastrand::f32() * 1.0, // Random initial delay
        pattern: FiringPattern::Single,
    };

    // Liberation value based on ship class
    let liberation = match type_id {
        20185 => 5, // Bestower (transport) - more slaves
        2006 => 3,  // Apocalypse - capital crew
        24690 => 2, // Harbinger/Absolution - larger crew
        24692 => 3, // Abaddon - battleship
        _ => 1,     // Regular frigates/cruisers
    };

    let stats = EnemyStats {
        type_id,
        name: name.into(),
        health,
        max_health: health,
        speed,
        score_value: score,
        is_boss: false,
        liberation_value: liberation,
    };

    let ai = EnemyAI {
        behavior,
        phase: fastrand::f32() * std::f32::consts::TAU,
        ..default()
    };

    // Get faction-appropriate engine trail (pointing up since enemies face down)
    let mut engine_trail = get_faction_engine_trail(type_id);
    engine_trail.offset = Vec2::new(0.0, 25.0); // Offset up since enemies face down

    // Get rotation: 180deg base (face down) + per-ship correction
    let base_rotation = std::f32::consts::PI; // Face down
    let correction = get_ship_rotation_correction(type_id);
    let total_rotation = base_rotation + correction;

    // Use sprites (2D camera compatible)
    if let Some(texture) = sprite {
        commands
            .spawn((
                Enemy,
                stats,
                weapon,
                ai,
                engine_trail,
                Sprite {
                    image: texture,
                    custom_size: Some(Vec2::splat(sprite_size)),
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, LAYER_ENEMIES)
                    .with_rotation(Quat::from_rotation_z(total_rotation)),
            ))
            .id()
    } else {
        // Color fallback - slightly smaller for non-square proportion
        commands
            .spawn((
                Enemy,
                stats,
                weapon,
                ai,
                engine_trail,
                Sprite {
                    color: base_color,
                    custom_size: Some(Vec2::new(sprite_size * 0.85, sprite_size)),
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, LAYER_ENEMIES),
            ))
            .id()
    }
}

/// Spawner update - spawns fighter escorts from Spawner enemies
pub(super) fn spawner_update(
    mut commands: Commands,
    time: Res<Time>,
    sprite_cache: Option<Res<crate::assets::ShipSpriteCache>>,
    model_cache: Option<Res<ShipModelCache>>,
    mut query: Query<(&Transform, &mut EnemySpawner), With<Enemy>>,
) {
    let dt = time.delta_secs();

    for (transform, mut spawner) in query.iter_mut() {
        spawner.spawn_timer -= dt;

        if spawner.spawn_timer <= 0.0 && spawner.spawned_count < spawner.max_spawned {
            spawner.spawn_timer = spawner.spawn_rate;
            spawner.spawned_count += 1;

            let pos = transform.translation.truncate();
            // Spawn fighters slightly offset from spawner
            let offset_x = (fastrand::f32() - 0.5) * 60.0;
            let spawn_pos = Vec2::new(pos.x + offset_x, pos.y - 30.0);

            let sprite = sprite_cache
                .as_ref()
                .and_then(|c| c.get(spawner.spawn_type_id));
            let model = model_cache.as_ref().map(|c| c.as_ref());

            spawn_enemy(
                &mut commands,
                spawner.spawn_type_id,
                spawn_pos,
                EnemyBehavior::Linear, // Spawned fighters use simple linear behavior
                sprite,
                model,
            );
        }
    }
}

// ============================================================================
// Enemy Variant System (data-driven specialized spawning)
// ============================================================================

/// Configuration for a specialized enemy variant.
/// Captures stat overrides, weapon overrides, and extra components.
pub struct EnemyVariantConfig {
    pub type_id: u32,
    pub behavior: EnemyBehavior,
    pub name: &'static str,
    pub health: f32,
    pub speed: f32,
    pub score_value: u64,
    pub is_boss: bool,
    pub liberation_value: u32,
    pub weapon_override: Option<EnemyWeapon>,
    pub spawner: Option<EnemySpawner>,
    pub disintegrator: Option<DisintegratorRamp>,
    pub remove_weapon: bool,
}

/// Predefined enemy variants
pub enum EnemyVariant {
    Kamikaze,
    Weaver,
    Sniper,
    Spawner,
    Tank,
    Damavik,
    StarvingDamavik,
    Vedmak,
    BlindingVedmak,
    Kikimora,
    Leshak,
    DrekavacBoss,
    ExecutionerElite,
    PunisherTank,
    RifterBerserker,
}

impl EnemyVariant {
    /// Get the configuration for this variant
    pub fn config(&self) -> EnemyVariantConfig {
        match self {
            Self::Kamikaze => EnemyVariantConfig {
                type_id: 589, // Executioner
                behavior: EnemyBehavior::Kamikaze,
                name: "Kamikaze",
                health: 15.0,
                speed: 180.0,
                score_value: 150,
                is_boss: false,
                liberation_value: 1,
                weapon_override: None,
                spawner: None,
                disintegrator: None,
                remove_weapon: false,
            },
            Self::Weaver => EnemyVariantConfig {
                type_id: 602, // Kestrel
                behavior: EnemyBehavior::Weaver,
                name: "Weaver",
                health: 25.0,
                speed: 140.0,
                score_value: 120,
                is_boss: false,
                liberation_value: 1,
                weapon_override: None,
                spawner: None,
                disintegrator: None,
                remove_weapon: false,
            },
            Self::Sniper => EnemyVariantConfig {
                type_id: 603, // Merlin
                behavior: EnemyBehavior::Sniper,
                name: "Sniper",
                health: 35.0,
                speed: 50.0,
                score_value: 130,
                is_boss: false,
                liberation_value: 1,
                weapon_override: Some(EnemyWeapon {
                    weapon_type: WeaponType::Railgun,
                    fire_rate: 0.4,
                    damage: 25.0,
                    bullet_speed: 400.0,
                    cooldown: 1.0,
                    pattern: FiringPattern::Single,
                }),
                spawner: None,
                disintegrator: None,
                remove_weapon: false,
            },
            Self::Spawner => EnemyVariantConfig {
                type_id: 593, // Tristan
                behavior: EnemyBehavior::Spawner,
                name: "Carrier",
                health: 80.0,
                speed: 40.0,
                score_value: 200,
                is_boss: false,
                liberation_value: 3,
                weapon_override: None,
                spawner: Some(EnemySpawner {
                    spawn_rate: 4.0,
                    spawn_timer: 2.0,
                    spawn_type_id: 589,
                    max_spawned: 3,
                    spawned_count: 0,
                }),
                disintegrator: None,
                remove_weapon: false,
            },
            Self::Tank => EnemyVariantConfig {
                type_id: 597, // Punisher
                behavior: EnemyBehavior::Tank,
                name: "Juggernaut",
                health: 150.0,
                speed: 35.0,
                score_value: 250,
                is_boss: false,
                liberation_value: 2,
                weapon_override: None,
                spawner: None,
                disintegrator: None,
                remove_weapon: false,
            },
            Self::Damavik => EnemyVariantConfig {
                type_id: triglavian::DAMAVIK,
                behavior: EnemyBehavior::Disintegrator,
                name: "Raznaborg Damavik",
                health: 120.0,
                speed: 100.0,
                score_value: 180,
                is_boss: false,
                liberation_value: 2,
                weapon_override: None,
                spawner: None,
                disintegrator: Some(DisintegratorRamp::new(5.0, 2.0, 6.0)),
                remove_weapon: true,
            },
            Self::StarvingDamavik => EnemyVariantConfig {
                type_id: triglavian::DAMAVIK,
                behavior: EnemyBehavior::Disintegrator,
                name: "Starving Damavik",
                health: 80.0,
                speed: 130.0,
                score_value: 150,
                is_boss: false,
                liberation_value: 1,
                weapon_override: None,
                spawner: None,
                disintegrator: Some(DisintegratorRamp::new(4.0, 1.8, 4.0)),
                remove_weapon: true,
            },
            Self::Vedmak => EnemyVariantConfig {
                type_id: triglavian::VEDMAK,
                behavior: EnemyBehavior::Disintegrator,
                name: "Harrowing Vedmak",
                health: 400.0,
                speed: 60.0,
                score_value: 350,
                is_boss: false,
                liberation_value: 5,
                weapon_override: None,
                spawner: None,
                disintegrator: Some(DisintegratorRamp::new(9.0, 2.0, 8.0)),
                remove_weapon: true,
            },
            Self::BlindingVedmak => EnemyVariantConfig {
                type_id: triglavian::VEDMAK,
                behavior: EnemyBehavior::Disintegrator,
                name: "Blinding Vedmak",
                health: 350.0,
                speed: 70.0,
                score_value: 320,
                is_boss: false,
                liberation_value: 4,
                weapon_override: None,
                spawner: None,
                disintegrator: Some(DisintegratorRamp::new(7.0, 2.0, 6.0)),
                remove_weapon: true,
            },
            Self::Kikimora => EnemyVariantConfig {
                type_id: triglavian::KIKIMORA,
                behavior: EnemyBehavior::Disintegrator,
                name: "Rapid Kikimora",
                health: 200.0,
                speed: 85.0,
                score_value: 280,
                is_boss: false,
                liberation_value: 3,
                weapon_override: None,
                spawner: None,
                disintegrator: Some(DisintegratorRamp::new(6.0, 1.5, 5.0)),
                remove_weapon: true,
            },
            Self::Leshak => EnemyVariantConfig {
                type_id: triglavian::LESHAK,
                behavior: EnemyBehavior::Disintegrator,
                name: "Siege Leshak",
                health: 600.0,
                speed: 40.0,
                score_value: 500,
                is_boss: false,
                liberation_value: 8,
                weapon_override: None,
                spawner: None,
                disintegrator: Some(DisintegratorRamp::new(12.0, 2.5, 10.0)),
                remove_weapon: true,
            },
            Self::DrekavacBoss => EnemyVariantConfig {
                type_id: triglavian::DREKAVAC,
                behavior: EnemyBehavior::Disintegrator,
                name: "Drekavac",
                health: 800.0,
                speed: 45.0,
                score_value: 1000,
                is_boss: true,
                liberation_value: 10,
                weapon_override: None,
                spawner: None,
                disintegrator: Some(DisintegratorRamp::new(14.0, 2.5, 10.0)),
                remove_weapon: true,
            },
            Self::ExecutionerElite => EnemyVariantConfig {
                type_id: 589, // Executioner
                behavior: EnemyBehavior::Homing,
                name: "Elite Executioner",
                health: 40.0,
                speed: 150.0,
                score_value: 200,
                is_boss: false,
                liberation_value: 1,
                weapon_override: Some(EnemyWeapon {
                    weapon_type: WeaponType::Laser,
                    fire_rate: 0.6,
                    damage: 18.0,
                    bullet_speed: 300.0,
                    cooldown: 0.3 + fastrand::f32() * 0.5,
                    pattern: FiringPattern::Single,
                }),
                spawner: None,
                disintegrator: None,
                remove_weapon: false,
            },
            Self::PunisherTank => EnemyVariantConfig {
                type_id: 597, // Punisher
                behavior: EnemyBehavior::Tank,
                name: "Heavy Punisher",
                health: 220.0,
                speed: 30.0,
                score_value: 300,
                is_boss: false,
                liberation_value: 2,
                weapon_override: Some(EnemyWeapon {
                    weapon_type: WeaponType::Laser,
                    fire_rate: 1.2,
                    damage: 15.0,
                    bullet_speed: 280.0,
                    cooldown: 0.5 + fastrand::f32() * 1.0,
                    pattern: FiringPattern::Single,
                }),
                spawner: None,
                disintegrator: None,
                remove_weapon: false,
            },
            Self::RifterBerserker => EnemyVariantConfig {
                type_id: 587, // Rifter
                behavior: EnemyBehavior::Zigzag,
                name: "Berserker Rifter",
                health: 60.0,
                speed: 140.0,
                score_value: 180,
                is_boss: false,
                liberation_value: 1,
                weapon_override: Some(EnemyWeapon {
                    weapon_type: WeaponType::Autocannon,
                    fire_rate: 1.8,
                    damage: 12.0,
                    bullet_speed: 260.0,
                    cooldown: 0.2 + fastrand::f32() * 0.3,
                    pattern: FiringPattern::Single,
                }),
                spawner: None,
                disintegrator: None,
                remove_weapon: false,
            },
        }
    }
}

/// Spawn a specialized enemy variant using data-driven config
pub fn spawn_variant(
    commands: &mut Commands,
    variant: EnemyVariant,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let config = variant.config();

    let entity = spawn_enemy(
        commands,
        config.type_id,
        position,
        config.behavior,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id: config.type_id,
        name: config.name.into(),
        health: config.health,
        max_health: config.health,
        speed: config.speed,
        score_value: config.score_value,
        is_boss: config.is_boss,
        liberation_value: config.liberation_value,
    });

    if let Some(weapon) = config.weapon_override {
        commands.entity(entity).insert(weapon);
    }

    if let Some(spawner) = config.spawner {
        commands.entity(entity).insert(spawner);
    }

    if let Some(disintegrator) = config.disintegrator {
        commands.entity(entity).insert(disintegrator);
    }

    if config.remove_weapon {
        commands.entity(entity).remove::<EnemyWeapon>();
    }

    entity
}

// ============================================================================
// Triglavian Ships (Disintegrator beam weapons with ramping damage)
// ============================================================================

/// Triglavian ship type IDs — verified CCP inventory_type IDs 2026-04.
pub mod triglavian {
    pub const DAMAVIK: u32 = 47269; // Light frigate
    pub const VEDMAK: u32 = 47270; // Cruiser
    pub const LESHAK: u32 = 47271; // Battleship
    pub const KIKIMORA: u32 = 49710; // Destroyer
    pub const DREKAVAC: u32 = 49711; // Battlecruiser
    pub const NERGAL: u32 = 52250; // Assault frigate
    pub const IKITURSA: u32 = 52252; // HAC
    pub const DRAUGUR: u32 = 52254; // Command destroyer
}

/// EDENCOM ship type IDs — verified CCP inventory_type IDs 2026-04.
pub mod edencom {
    pub const SKYBREAKER: u32 = 54731; // Frigate
    pub const STORMBRINGER: u32 = 54732; // Battleship
    pub const THUNDERCHILD: u32 = 54733; // Cruiser
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_variants() -> Vec<EnemyVariant> {
        vec![
            EnemyVariant::Kamikaze,
            EnemyVariant::Weaver,
            EnemyVariant::Sniper,
            EnemyVariant::Spawner,
            EnemyVariant::Tank,
            EnemyVariant::Damavik,
            EnemyVariant::StarvingDamavik,
            EnemyVariant::Vedmak,
            EnemyVariant::BlindingVedmak,
            EnemyVariant::Kikimora,
            EnemyVariant::Leshak,
            EnemyVariant::DrekavacBoss,
            EnemyVariant::ExecutionerElite,
            EnemyVariant::PunisherTank,
            EnemyVariant::RifterBerserker,
        ]
    }

    #[test]
    fn all_variants_have_positive_health() {
        for variant in all_variants() {
            let config = variant.config();
            assert!(config.health > 0.0, "{} has zero health", config.name);
        }
    }

    #[test]
    fn all_variants_have_positive_speed() {
        for variant in all_variants() {
            let config = variant.config();
            assert!(config.speed > 0.0, "{} has zero speed", config.name);
        }
    }

    #[test]
    fn all_variants_have_positive_score() {
        for variant in all_variants() {
            let config = variant.config();
            assert!(config.score_value > 0, "{} has zero score", config.name);
        }
    }

    #[test]
    fn all_variants_have_names() {
        for variant in all_variants() {
            let config = variant.config();
            assert!(!config.name.is_empty());
        }
    }

    #[test]
    fn only_drekavac_is_boss() {
        for variant in all_variants() {
            let config = variant.config();
            if config.name == "Drekavac" {
                assert!(config.is_boss, "Drekavac should be a boss");
            } else {
                assert!(!config.is_boss, "{} should not be a boss", config.name);
            }
        }
    }

    #[test]
    fn kamikaze_is_fast_and_fragile() {
        let config = EnemyVariant::Kamikaze.config();
        let tank = EnemyVariant::Tank.config();
        assert!(config.speed > tank.speed);
        assert!(config.health < tank.health);
    }

    #[test]
    fn tank_is_slow_and_tanky() {
        let config = EnemyVariant::Tank.config();
        assert!(config.health >= 150.0);
        assert!(config.speed <= 40.0);
    }

    #[test]
    fn sniper_has_weapon_override() {
        let config = EnemyVariant::Sniper.config();
        assert!(config.weapon_override.is_some());
    }

    #[test]
    fn spawner_has_spawner_component() {
        let config = EnemyVariant::Spawner.config();
        assert!(config.spawner.is_some());
    }

    #[test]
    fn triglavian_variants_have_disintegrator() {
        let trig_variants = vec![
            EnemyVariant::Damavik,
            EnemyVariant::StarvingDamavik,
            EnemyVariant::Vedmak,
            EnemyVariant::BlindingVedmak,
            EnemyVariant::Kikimora,
            EnemyVariant::Leshak,
            EnemyVariant::DrekavacBoss,
        ];
        for variant in trig_variants {
            let config = variant.config();
            assert!(
                config.disintegrator.is_some(),
                "{} should have disintegrator",
                config.name
            );
            assert!(
                config.remove_weapon,
                "{} should remove standard weapon",
                config.name
            );
            assert_eq!(config.behavior, EnemyBehavior::Disintegrator);
        }
    }

    #[test]
    fn standard_variants_no_disintegrator() {
        let standard = vec![
            EnemyVariant::Kamikaze,
            EnemyVariant::Weaver,
            EnemyVariant::Sniper,
            EnemyVariant::Spawner,
            EnemyVariant::Tank,
            EnemyVariant::ExecutionerElite,
            EnemyVariant::PunisherTank,
            EnemyVariant::RifterBerserker,
        ];
        for variant in standard {
            let config = variant.config();
            assert!(
                config.disintegrator.is_none(),
                "{} should not have disintegrator",
                config.name
            );
            assert!(!config.remove_weapon);
        }
    }

    #[test]
    fn executioner_elite_is_fast_with_laser() {
        let config = EnemyVariant::ExecutionerElite.config();
        assert!(config.speed > 100.0, "Elite Executioner should be fast");
        assert!(config.health > 25.0, "Elite Executioner should be tougher than base");
        let weapon = config.weapon_override.expect("should have weapon override");
        assert_eq!(weapon.weapon_type, WeaponType::Laser);
        assert!(weapon.damage > 10.0, "laser should be strong");
    }

    #[test]
    fn punisher_tank_is_slow_and_tanky() {
        let config = EnemyVariant::PunisherTank.config();
        assert!(config.health >= 200.0, "Heavy Punisher should be very tanky");
        assert!(config.speed <= 35.0, "Heavy Punisher should be slow");
        let weapon = config.weapon_override.expect("should have weapon override");
        assert_eq!(weapon.weapon_type, WeaponType::Laser);
    }

    #[test]
    fn rifter_berserker_is_fast_with_autocannon() {
        let config = EnemyVariant::RifterBerserker.config();
        assert!(config.speed > 100.0, "Berserker Rifter should be fast");
        assert!(config.health > 35.0, "Berserker should be tougher than base Rifter");
        let weapon = config.weapon_override.expect("should have weapon override");
        assert_eq!(weapon.weapon_type, WeaponType::Autocannon);
        assert!(weapon.fire_rate > 1.0, "autocannon should fire fast");
    }

    #[test]
    fn drekavac_boss_has_highest_health() {
        let boss = EnemyVariant::DrekavacBoss.config();
        for variant in all_variants() {
            let config = variant.config();
            assert!(
                boss.health >= config.health,
                "Drekavac should have highest health"
            );
        }
    }

    #[test]
    fn drekavac_boss_has_highest_score() {
        let boss = EnemyVariant::DrekavacBoss.config();
        for variant in all_variants() {
            let config = variant.config();
            assert!(
                boss.score_value >= config.score_value,
                "Drekavac should have highest score"
            );
        }
    }
}
