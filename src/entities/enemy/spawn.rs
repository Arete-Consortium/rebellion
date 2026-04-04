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

        // === TRIGLAVIAN ===
        47269 => ("Damavik", 80.0, 100.0, 150, ShipClass::Frigate), // Disintegrator frigate
        49710 => ("Kikimora", 100.0, 90.0, 200, ShipClass::Destroyer), // Disintegrator destroyer
        47271 => ("Vedmak", 200.0, 70.0, 350, ShipClass::Cruiser),  // Disintegrator cruiser
        49711 => ("Ikitursa", 280.0, 60.0, 450, ShipClass::Cruiser), // HAC
        47273 => ("Drekavac", 350.0, 50.0, 600, ShipClass::Battlecruiser), // BC
        47466 => ("Leshak", 600.0, 40.0, 1000, ShipClass::Battleship), // BS
        56756 => ("Xordazh", 2000.0, 20.0, 5000, ShipClass::Battleship), // World Ark (capital)

        // === EDENCOM ===
        56757 => ("Skybreaker", 90.0, 95.0, 180, ShipClass::Frigate), // Vorton frigate
        56759 => ("Thunderchild", 220.0, 65.0, 400, ShipClass::Cruiser), // Vorton cruiser
        56760 => ("Stormbringer", 550.0, 45.0, 900, ShipClass::Battleship), // Vorton BS

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

/// Spawn a specialized Kamikaze enemy (glowing, suicide rush)
pub fn spawn_kamikaze(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = 589; // Executioner - fast, aggressive
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Kamikaze,
        sprite,
        model_cache,
    );

    // Boost stats for kamikaze
    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Kamikaze".into(),
        health: 15.0, // Low health
        max_health: 15.0,
        speed: 180.0,     // Very fast
        score_value: 150, // Worth more
        is_boss: false,
        liberation_value: 1,
    });

    entity
}

/// Spawn a Weaver enemy (fast sine-wave harasser)
pub fn spawn_weaver(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = 602; // Kestrel - agile
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Weaver,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Weaver".into(),
        health: 25.0,
        max_health: 25.0,
        speed: 140.0, // Fast
        score_value: 120,
        is_boss: false,
        liberation_value: 1,
    });

    entity
}

/// Spawn a Sniper enemy (long-range, stationary)
pub fn spawn_sniper(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = 603; // Merlin - Caldari, railgun platform
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Sniper,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Sniper".into(),
        health: 35.0,
        max_health: 35.0,
        speed: 50.0, // Slow
        score_value: 130,
        is_boss: false,
        liberation_value: 1,
    });

    // Enhanced weapon for sniper
    commands.entity(entity).insert(EnemyWeapon {
        weapon_type: WeaponType::Railgun,
        fire_rate: 0.4,      // Slow but powerful
        damage: 25.0,        // High damage
        bullet_speed: 400.0, // Fast projectiles
        cooldown: 1.0,
        pattern: FiringPattern::Single,
    });

    entity
}

/// Spawn a Spawner enemy (deploys fighters)
pub fn spawn_spawner_enemy(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = 593; // Tristan - drone boat
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Spawner,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Carrier".into(),
        health: 80.0, // Tanky
        max_health: 80.0,
        speed: 40.0, // Very slow
        score_value: 200,
        is_boss: false,
        liberation_value: 3, // More crew
    });

    // Add spawner component
    commands.entity(entity).insert(EnemySpawner {
        spawn_rate: 4.0,
        spawn_timer: 2.0,
        spawn_type_id: 589, // Spawns Executioners
        max_spawned: 3,
        spawned_count: 0,
    });

    entity
}

/// Spawn a Tank enemy (heavy armor, slow)
pub fn spawn_tank(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = 597; // Punisher - heavily armored
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Tank,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Juggernaut".into(),
        health: 150.0, // Very tanky
        max_health: 150.0,
        speed: 35.0, // Very slow
        score_value: 250,
        is_boss: false,
        liberation_value: 2,
    });

    entity
}

// ============================================================================
// Triglavian Ships (Disintegrator beam weapons with ramping damage)
// ============================================================================

/// Triglavian ship type IDs
pub mod triglavian {
    pub const DAMAVIK: u32 = 47269; // Light frigate
    pub const VEDMAK: u32 = 47270; // Cruiser
    pub const DREKAVAC: u32 = 47271; // Battlecruiser
    pub const LESHAK: u32 = 47272; // Battleship
    pub const KIKIMORA: u32 = 47273; // Destroyer
}

/// Spawn a Raznaborg Damavik (light Triglavian frigate)
/// Fast, agile, moderate ramp (2.0x max)
pub fn spawn_damavik(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = triglavian::DAMAVIK;
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Disintegrator,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Raznaborg Damavik".into(),
        health: 120.0,
        max_health: 120.0,
        speed: 100.0, // Fast frigate
        score_value: 180,
        is_boss: false,
        liberation_value: 2,
    });

    // Disintegrator beam weapon (tuned for survivability)
    commands.entity(entity).insert(DisintegratorRamp::new(
        5.0, // Base damage per second (reduced from 8)
        2.0, // Max 2x multiplier
        6.0, // 6 seconds to max ramp
    ));

    // No standard weapon - uses disintegrator beam instead
    commands.entity(entity).remove::<EnemyWeapon>();

    entity
}

/// Spawn a Starving Damavik (fast, fragile variant)
/// Very fast, lower HP, quick ramp (1.8x max)
pub fn spawn_starving_damavik(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = triglavian::DAMAVIK;
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Disintegrator,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Starving Damavik".into(),
        health: 80.0, // Fragile
        max_health: 80.0,
        speed: 130.0, // Very fast
        score_value: 150,
        is_boss: false,
        liberation_value: 1,
    });

    commands.entity(entity).insert(DisintegratorRamp::new(
        4.0, // Lower base damage (reduced from 6)
        1.8, // Lower max multiplier
        4.0, // Faster ramp time
    ));

    commands.entity(entity).remove::<EnemyWeapon>();

    entity
}

/// Spawn a Harrowing Vedmak (heavy Triglavian cruiser)
/// Slow, tanky, high ramp (2.5x max)
pub fn spawn_vedmak(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = triglavian::VEDMAK;
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Disintegrator,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Harrowing Vedmak".into(),
        health: 400.0, // Heavy cruiser
        max_health: 400.0,
        speed: 60.0, // Slower
        score_value: 350,
        is_boss: false,
        liberation_value: 5,
    });

    commands.entity(entity).insert(DisintegratorRamp::new(
        9.0, // High base damage (reduced from 15)
        2.0, // Max multiplier (reduced from 2.5)
        8.0, // Longer ramp time
    ));

    commands.entity(entity).remove::<EnemyWeapon>();

    entity
}

/// Spawn a Blinding Vedmak (EWAR variant)
/// Medium stats, moderate ramp with debuff effect
pub fn spawn_blinding_vedmak(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = triglavian::VEDMAK;
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Disintegrator,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Blinding Vedmak".into(),
        health: 350.0,
        max_health: 350.0,
        speed: 70.0,
        score_value: 320,
        is_boss: false,
        liberation_value: 4,
    });

    commands.entity(entity).insert(DisintegratorRamp::new(
        7.0, // Moderate damage (reduced from 12)
        2.0, // Standard multiplier
        6.0,
    ));

    commands.entity(entity).remove::<EnemyWeapon>();

    entity
}

/// Spawn a Drekavac (Triglavian battlecruiser boss)
/// Very tanky, high damage, extreme ramp (3.0x max)
pub fn spawn_drekavac_boss(
    commands: &mut Commands,
    position: Vec2,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let type_id = triglavian::DREKAVAC;
    let entity = spawn_enemy(
        commands,
        type_id,
        position,
        EnemyBehavior::Disintegrator,
        sprite,
        model_cache,
    );

    commands.entity(entity).insert(EnemyStats {
        type_id,
        name: "Drekavac".into(),
        health: 800.0, // Boss-level HP
        max_health: 800.0,
        speed: 45.0, // Slow battlecruiser
        score_value: 1000,
        is_boss: true, // This is a boss!
        liberation_value: 10,
    });

    commands.entity(entity).insert(DisintegratorRamp::new(
        14.0, // High base damage (reduced from 25)
        2.5,  // High max multiplier (reduced from 3.0)
        10.0, // Long ramp time (counterplay: stay mobile)
    ));

    commands.entity(entity).remove::<EnemyWeapon>();

    entity
}
