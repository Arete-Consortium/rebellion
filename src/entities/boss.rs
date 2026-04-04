//! Boss Entities
//!
//! Stage bosses for the Elder Fleet campaign.

#![allow(dead_code)]

use super::player::Hitbox;
use crate::assets::{get_model_scale, ShipModelCache, ShipModelRotation};
use crate::core::*;
use bevy::prelude::*;

/// Marker component for bosses
#[derive(Component)]
pub struct Boss;

/// Boss data
#[derive(Component, Debug, Clone)]
pub struct BossData {
    /// Boss ID (1-13)
    pub id: u32,
    /// Stage number
    pub stage: u32,
    /// Display name
    pub name: String,
    /// Boss title
    pub title: String,
    /// Ship class name
    pub ship_class: String,
    /// Ship type ID (for sprite)
    pub type_id: u32,
    /// Maximum health
    pub max_health: f32,
    /// Current health
    pub health: f32,
    /// Current phase (1-5)
    pub current_phase: u32,
    /// Total phases
    pub total_phases: u32,
    /// Score value
    pub score_value: u64,
    /// Liberation value (souls freed on defeat)
    pub liberation_value: u32,
    /// Is stationary (stations/gates)
    pub stationary: bool,
    /// Intro dialogue
    pub dialogue_intro: String,
    /// Defeat dialogue
    pub dialogue_defeat: String,
    /// Is boss enraged (below 20% health)
    pub is_enraged: bool,
    /// Enrage threshold (default 0.2 = 20%)
    pub enrage_threshold: f32,
}

/// Boss health bar component
#[derive(Component)]
pub struct BossHealthBar;

/// Boss phase data
#[derive(Clone, Debug)]
pub struct BossPhase {
    pub phase_number: u32,
    pub health_threshold: f32,
    pub attack_pattern: String,
    pub spawns_escorts: bool,
    pub escort_count: u32,
}

/// Boss states for intro/battle/defeat sequences
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BossState {
    #[default]
    Intro, // Boss entering, name card showing
    Battle,          // Active combat
    PhaseTransition, // Changing phases
    Defeated,        // Death sequence playing
}

/// Boss movement pattern
#[derive(Component, Debug, Clone)]
pub struct BossMovement {
    pub pattern: MovementPattern,
    pub timer: f32,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementPattern {
    Stationary,
    Sweep,      // Side to side
    Strafe,     // Move and stop
    Aggressive, // Chase player
    Descend,    // For intro
}

impl Default for BossMovement {
    fn default() -> Self {
        Self {
            pattern: MovementPattern::Sweep,
            timer: 0.0,
            speed: 80.0,
        }
    }
}

/// Boss attack timer
#[derive(Component, Debug)]
pub struct BossAttack {
    pub pattern: String,
    pub fire_timer: f32,
    pub fire_rate: f32,
    pub burst_count: u32,
    pub burst_remaining: u32,
}

impl Default for BossAttack {
    fn default() -> Self {
        Self {
            pattern: "steady_beam".to_string(),
            fire_timer: 0.0,
            fire_rate: 0.8,
            burst_count: 3,
            burst_remaining: 0,
        }
    }
}

/// Boss bundle
#[derive(Bundle)]
pub struct BossBundle {
    pub boss: Boss,
    pub data: BossData,
    pub state: BossState,
    pub movement: BossMovement,
    pub attack: BossAttack,
    pub hitbox: Hitbox,
    pub sprite: Sprite,
    pub transform: Transform,
}

/// Load boss data from stage number
pub fn get_boss_for_stage(stage: u32) -> Option<BossData> {
    // Boss definitions based on config/bosses_campaign.json
    match stage {
        1 => Some(BossData {
            id: 1,
            stage: 1,
            name: "Slave Transport Overseer".to_string(),
            title: "Convoy Master Krador".to_string(),
            ship_class: "Bestower".to_string(),
            type_id: 1944,
            max_health: 500.0,
            health: 500.0,
            current_phase: 1,
            total_phases: 2,
            score_value: 5000,
            liberation_value: 50,
            stationary: false,
            dialogue_intro: "You dare attack an Imperial transport? Foolish rebel!".to_string(),
            dialogue_defeat: "The slaves... they're escaping...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        2 => Some(BossData {
            id: 2,
            stage: 2,
            name: "Patrol Commander".to_string(),
            title: "Commander Sarevok".to_string(),
            ship_class: "Navy Omen".to_string(),
            type_id: 2006,
            max_health: 800.0,
            health: 800.0,
            current_phase: 1,
            total_phases: 2,
            score_value: 7500,
            liberation_value: 0,
            stationary: false,
            dialogue_intro: "Another rebel scum. I've crushed dozens like you.".to_string(),
            dialogue_defeat: "Impossible... a frigate...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        3 => Some(BossData {
            id: 3,
            stage: 3,
            name: "Station Defense Battery".to_string(),
            title: "Holding Facility Defense Grid".to_string(),
            ship_class: "Orbital Platform".to_string(),
            type_id: 0,
            max_health: 1200.0,
            health: 1200.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 10000,
            liberation_value: 100,
            stationary: true,
            dialogue_intro: "Defense grid online. Eliminate hostile.".to_string(),
            dialogue_defeat: "Core breach... structural failure...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        4 => Some(BossData {
            id: 4,
            stage: 4,
            name: "Holder's Escort Fleet".to_string(),
            title: "Lord Holder Arzad's Guard".to_string(),
            ship_class: "Mixed".to_string(),
            type_id: 624, // Maller as primary
            max_health: 1500.0,
            health: 1500.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 15000,
            liberation_value: 75,
            stationary: false,
            dialogue_intro: "You attack a Holder's estate? You will burn for this heresy!".to_string(),
            dialogue_defeat: "My slaves... my property... all lost...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        5 => Some(BossData {
            id: 5,
            stage: 5,
            name: "Imperial Customs Commandant".to_string(),
            title: "Commandant Torash".to_string(),
            ship_class: "Prophecy".to_string(),
            type_id: 630,
            max_health: 2000.0,
            health: 2000.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 20000,
            liberation_value: 150,
            stationary: false,
            dialogue_intro: "No cargo passes without Imperial inspection. Surrender.".to_string(),
            dialogue_defeat: "The customs... will... be avenged...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        6 => Some(BossData {
            id: 6,
            stage: 6,
            name: "Inquisitor Vessel".to_string(),
            title: "Inquisitor Malkov".to_string(),
            ship_class: "Prophecy Variant".to_string(),
            type_id: 630,
            max_health: 2500.0,
            health: 2500.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 25000,
            liberation_value: 0,
            stationary: false,
            dialogue_intro: "Heretics! The Scriptures demand your purification!".to_string(),
            dialogue_defeat: "God... will not... forget this blasphemy...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        7 => Some(BossData {
            id: 7,
            stage: 7,
            name: "Navy Harbinger Strike Group".to_string(),
            title: "Strike Commander Venak".to_string(),
            ship_class: "Harbinger".to_string(),
            type_id: 24696,
            max_health: 3000.0,
            health: 3000.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 30000,
            liberation_value: 0,
            stationary: false,
            dialogue_intro: "Strike group, weapons free. Eliminate rebel contact.".to_string(),
            dialogue_defeat: "All ships... lost... how...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        8 => Some(BossData {
            id: 8,
            stage: 8,
            name: "Stargate Defense Grid".to_string(),
            title: "Gate Control: Sahtogas".to_string(),
            ship_class: "Infrastructure".to_string(),
            type_id: 0,
            max_health: 4000.0,
            health: 4000.0,
            current_phase: 1,
            total_phases: 4,
            score_value: 35000,
            liberation_value: 0,
            stationary: true,
            dialogue_intro: "Unauthorized vessel. Activating defense protocols.".to_string(),
            dialogue_defeat: "Gate control... offline... rebels... have breached...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        9 => Some(BossData {
            id: 9,
            stage: 9,
            name: "Amarr Battlestation Core".to_string(),
            title: "Holding Facility Theta-7".to_string(),
            ship_class: "Station".to_string(),
            type_id: 0,
            max_health: 6000.0,
            health: 6000.0,
            current_phase: 1,
            total_phases: 5,
            score_value: 50000,
            liberation_value: 500,
            stationary: true,
            dialogue_intro: "Station defense grid activated. All personnel to combat stations.".to_string(),
            dialogue_defeat: "Reactor critical... containment failing... the slaves are free...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        10 => Some(BossData {
            id: 10,
            stage: 10,
            name: "Imperial Navy Armageddon".to_string(),
            title: "Admiral Karsoth's Hammer".to_string(),
            ship_class: "Armageddon".to_string(),
            type_id: 643,
            max_health: 8000.0,
            health: 8000.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 75000,
            liberation_value: 200,
            stationary: false,
            dialogue_intro: "You face the might of the Imperial Navy. Prepare for oblivion.".to_string(),
            dialogue_defeat: "The Armageddon... falls... this cannot be...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        11 => Some(BossData {
            id: 11,
            stage: 11,
            name: "Amarr Carrier".to_string(),
            title: "Carrier Divine Providence".to_string(),
            ship_class: "Archon".to_string(),
            type_id: 23757,
            max_health: 10000.0,
            health: 10000.0,
            current_phase: 1,
            total_phases: 4,
            score_value: 100000,
            liberation_value: 300,
            stationary: false,
            dialogue_intro: "Launch all fighters. Annihilate the rebel frigate.".to_string(),
            dialogue_defeat: "Flight deck... compromised... she's going down...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        12 => Some(BossData {
            id: 12,
            stage: 12,
            name: "Lord Admiral's Apocalypse".to_string(),
            title: "Lord Admiral Vanir".to_string(),
            ship_class: "Apocalypse Navy Issue".to_string(),
            type_id: 24690,
            max_health: 12000.0,
            health: 12000.0,
            current_phase: 1,
            total_phases: 4,
            score_value: 150000,
            liberation_value: 0,
            stationary: false,
            dialogue_intro: "I have served the Empire for two hundred years. You will not take this day.".to_string(),
            dialogue_defeat: "My Emperor... I have... failed...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        13 => Some(BossData {
            id: 13,
            stage: 13,
            name: "Avatar-class Titan".to_string(),
            title: "Imperial Titan Empress's Wrath".to_string(),
            ship_class: "Avatar".to_string(),
            type_id: 11567,
            max_health: 25000.0,
            health: 25000.0,
            current_phase: 1,
            total_phases: 5,
            score_value: 500000,
            liberation_value: 1000,
            stationary: false,
            dialogue_intro: "A frigate? Against a Titan? Your courage is matched only by your foolishness.".to_string(),
            dialogue_defeat: "The Empress's Wrath... destroyed by... a single pilot... The Empire... will remember this day...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.15, // Titan enrages at 15%
        }),
        _ => None,
    }
}

/// Spawn a stage boss
pub fn spawn_boss(
    commands: &mut Commands,
    stage: u32,
    enemy_faction: crate::core::Faction,
    sprite_cache: Option<&crate::assets::ShipSpriteCache>,
    model_cache: Option<&ShipModelCache>,
) -> bool {
    use crate::systems::boss::{BossDroneSpawner, DroneSpawnPattern};

    let Some(boss_data) = get_boss_for_stage(stage) else {
        return false;
    };

    // Get faction-appropriate drone types
    let fast_drone = enemy_faction.fighter_type_id();
    let tough_drone = enemy_faction.tough_fighter_type_id();

    // Get boss scale based on ship class
    let scale_mult = match boss_data.ship_class.as_str() {
        "Bestower" | "Orbital Platform" => 2.0,
        "Navy Omen" | "Prophecy" | "Prophecy Variant" | "Maller" => 2.5,
        "Harbinger" | "Infrastructure" => 3.0,
        "Station" | "Armageddon" => 4.0,
        "Archon" | "Apocalypse Navy Issue" => 5.0,
        "Avatar" => 7.0,
        _ => 2.0,
    };

    let size = 64.0 * scale_mult;
    let stationary = boss_data.stationary;

    let movement = if stationary {
        BossMovement {
            pattern: MovementPattern::Stationary,
            ..default()
        }
    } else {
        BossMovement {
            pattern: MovementPattern::Descend,
            timer: 0.0,
            speed: 80.0,
        }
    };

    // Drone spawner for carrier/station bosses (uses faction-appropriate ships)
    let drone_spawner = match stage {
        // Stage 3 - Orbital Platform (first station)
        3 => Some(BossDroneSpawner {
            spawn_interval: 8.0,
            spawn_timer: 4.0,
            drones_per_wave: 2,
            drone_type_id: fast_drone,
            max_drones: 4,
            active_drones: 0,
            pattern: DroneSpawnPattern::FromSides,
        }),
        // Stage 8 - Stargate Defense
        8 => Some(BossDroneSpawner {
            spawn_interval: 6.0,
            spawn_timer: 3.0,
            drones_per_wave: 3,
            drone_type_id: fast_drone,
            max_drones: 6,
            active_drones: 0,
            pattern: DroneSpawnPattern::FromSides,
        }),
        // Stage 9 - Battlestation (major drone spawner)
        9 => Some(BossDroneSpawner {
            spawn_interval: 5.0,
            spawn_timer: 2.0,
            drones_per_wave: 4,
            drone_type_id: tough_drone, // Tougher fighters
            max_drones: 8,
            active_drones: 0,
            pattern: DroneSpawnPattern::Surround,
        }),
        // Stage 11 - Carrier (dedicated drone boat)
        11 => Some(BossDroneSpawner {
            spawn_interval: 4.0,
            spawn_timer: 2.0,
            drones_per_wave: 5,
            drone_type_id: fast_drone,
            max_drones: 10,
            active_drones: 0,
            pattern: DroneSpawnPattern::VFormation,
        }),
        // Stage 13 - Titan (light drone support)
        13 => Some(BossDroneSpawner {
            spawn_interval: 7.0,
            spawn_timer: 5.0,
            drones_per_wave: 3,
            drone_type_id: tough_drone, // Tougher fighters
            max_drones: 6,
            active_drones: 0,
            pattern: DroneSpawnPattern::Flanking,
        }),
        _ => None,
    };

    // Spawn at top of screen
    let start_y = SCREEN_HEIGHT / 2.0 + size;

    // Try 3D model first
    if boss_data.type_id > 0 {
        if let Some(cache) = model_cache {
            if let Some(scene_handle) = cache.get(boss_data.type_id) {
                let model_rot = ShipModelRotation::new_boss();
                let model_scale = get_model_scale(boss_data.type_id) * size;

                let mut entity_commands = commands.spawn((
                    Boss,
                    boss_data,
                    BossState::Intro,
                    movement,
                    BossAttack::default(),
                    Hitbox {
                        radius: size / 2.0 * 0.8,
                    },
                    model_rot.clone(),
                    SceneRoot(scene_handle),
                    Transform::from_xyz(0.0, start_y, 0.0)
                        .with_scale(Vec3::splat(model_scale))
                        .with_rotation(model_rot.base_rotation),
                ));

                if let Some(spawner) = drone_spawner {
                    entity_commands.insert(spawner);
                }

                return true;
            }
        }
    }

    // Fallback to sprite
    let sprite = if boss_data.type_id > 0 {
        if let Some(cache) = sprite_cache {
            if let Some(texture) = cache.get(boss_data.type_id) {
                Sprite {
                    image: texture,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                }
            } else {
                Sprite {
                    color: Color::srgb(0.9, 0.7, 0.3), // Amarr gold
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                }
            }
        } else {
            Sprite {
                color: Color::srgb(0.9, 0.7, 0.3),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        }
    } else {
        // Station/infrastructure - red square
        Sprite {
            color: Color::srgb(0.8, 0.2, 0.2),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        }
    };

    let mut entity_commands = commands.spawn(BossBundle {
        boss: Boss,
        data: boss_data,
        state: BossState::Intro,
        movement,
        attack: BossAttack::default(),
        hitbox: Hitbox {
            radius: size / 2.0 * 0.8,
        },
        sprite,
        // Ship renders face UP, rotate 180° to face DOWN
        transform: Transform::from_xyz(0.0, start_y, LAYER_ENEMIES)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
    });

    if let Some(spawner) = drone_spawner {
        entity_commands.insert(spawner);
    }

    true
}

/// Get phase health threshold
pub fn get_phase_threshold(phase: u32, total_phases: u32) -> f32 {
    match (phase, total_phases) {
        (1, _) => 1.0,
        (2, 2) => 0.4,
        (2, 3) => 0.6,
        (2, 4) => 0.7,
        (2, 5) => 0.75,
        (3, 3) => 0.3,
        (3, 4) => 0.4,
        (3, 5) => 0.5,
        (4, 4) => 0.15,
        (4, 5) => 0.25,
        (5, 5) => 0.05,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_13_stages_have_bosses() {
        for stage in 1..=13 {
            assert!(
                get_boss_for_stage(stage).is_some(),
                "Stage {} missing boss",
                stage
            );
        }
    }

    #[test]
    fn stage_0_has_no_boss() {
        assert!(get_boss_for_stage(0).is_none());
    }

    #[test]
    fn stage_14_has_no_boss() {
        assert!(get_boss_for_stage(14).is_none());
    }

    #[test]
    fn boss_ids_match_stages() {
        for stage in 1..=13 {
            let boss = get_boss_for_stage(stage).unwrap();
            assert_eq!(boss.id, stage);
            assert_eq!(boss.stage, stage);
        }
    }

    #[test]
    fn boss_health_increases_through_campaign() {
        let first = get_boss_for_stage(1).unwrap();
        let last = get_boss_for_stage(13).unwrap();
        assert!(last.max_health > first.max_health);
    }

    #[test]
    fn boss_score_increases_through_campaign() {
        let first = get_boss_for_stage(1).unwrap();
        let last = get_boss_for_stage(13).unwrap();
        assert!(last.score_value > first.score_value);
    }

    #[test]
    fn boss_health_equals_max_on_spawn() {
        for stage in 1..=13 {
            let boss = get_boss_for_stage(stage).unwrap();
            assert_eq!(boss.health, boss.max_health);
        }
    }

    #[test]
    fn boss_names_not_empty() {
        for stage in 1..=13 {
            let boss = get_boss_for_stage(stage).unwrap();
            assert!(!boss.name.is_empty(), "Stage {} boss has no name", stage);
            assert!(!boss.title.is_empty(), "Stage {} boss has no title", stage);
        }
    }

    #[test]
    fn boss_dialogue_not_empty() {
        for stage in 1..=13 {
            let boss = get_boss_for_stage(stage).unwrap();
            assert!(
                !boss.dialogue_intro.is_empty(),
                "Stage {} boss has no intro dialogue",
                stage
            );
            assert!(
                !boss.dialogue_defeat.is_empty(),
                "Stage {} boss has no defeat dialogue",
                stage
            );
        }
    }

    #[test]
    fn boss_starts_not_enraged() {
        for stage in 1..=13 {
            let boss = get_boss_for_stage(stage).unwrap();
            assert!(!boss.is_enraged);
        }
    }

    #[test]
    fn boss_enrage_threshold_valid() {
        for stage in 1..=13 {
            let boss = get_boss_for_stage(stage).unwrap();
            assert!(
                boss.enrage_threshold > 0.0 && boss.enrage_threshold < 1.0,
                "Stage {} enrage threshold out of range",
                stage
            );
        }
    }

    #[test]
    fn final_boss_is_avatar_titan() {
        let boss = get_boss_for_stage(13).unwrap();
        assert!(
            boss.name.contains("Avatar"),
            "Final boss should be Avatar Titan"
        );
    }

    #[test]
    fn final_boss_has_most_phases() {
        let final_boss = get_boss_for_stage(13).unwrap();
        for stage in 1..13 {
            let boss = get_boss_for_stage(stage).unwrap();
            assert!(final_boss.total_phases >= boss.total_phases);
        }
    }

    // Phase threshold tests
    #[test]
    fn phase_1_always_100_percent() {
        for total in 2..=5 {
            assert_eq!(get_phase_threshold(1, total), 1.0);
        }
    }

    #[test]
    fn phase_thresholds_decrease() {
        // For a 3-phase boss: phase 1 > phase 2 > phase 3
        assert!(get_phase_threshold(1, 3) > get_phase_threshold(2, 3));
        assert!(get_phase_threshold(2, 3) > get_phase_threshold(3, 3));
    }

    #[test]
    fn phase_thresholds_positive() {
        for total in 2..=5 {
            for phase in 1..=total {
                let threshold = get_phase_threshold(phase, total);
                assert!(
                    threshold > 0.0,
                    "Phase {}/{} threshold should be positive",
                    phase,
                    total
                );
            }
        }
    }

    // BossState tests
    #[test]
    fn boss_state_default_is_intro() {
        assert_eq!(BossState::default(), BossState::Intro);
    }

    // BossMovement tests
    #[test]
    fn boss_movement_default_sweep() {
        let movement = BossMovement::default();
        assert_eq!(movement.pattern, MovementPattern::Sweep);
        assert!(movement.speed > 0.0);
    }

    // BossAttack tests
    #[test]
    fn boss_attack_default_values() {
        let attack = BossAttack::default();
        assert!(attack.fire_rate > 0.0);
        assert!(attack.burst_count > 0);
    }
}
