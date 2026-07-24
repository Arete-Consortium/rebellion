//! Elder Fleet Campaign Systems
//!
//! Minmatar Republic vs Amarr Empire — 5-mission campaign.
//! Patterned after the Triglavian campaign with Elder Fleet-specific
//! enemy pools, boss behaviours, and mission flow.

use crate::assets::ShipSpriteCache;
use crate::core::{DamageType, Difficulty, GameState, LAYER_ENEMIES};
use crate::entities::boss::{Boss, BossAttack, BossData, BossMovement, BossState, MovementPattern};
use crate::entities::projectile::ProjectilePhysics;
use crate::entities::{spawn_enemy, spawn_variant, EnemyBehavior, EnemyStats, EnemyVariant, Hitbox};
use bevy::prelude::*;

// =============================================================================
// CAMPAIGN STATE
// =============================================================================

/// Elder Fleet campaign state resource
#[derive(Resource, Default)]
pub struct ElderFleetCampaignState {
    pub current_mission: u32,
    pub current_wave: u32,
    pub waves_in_mission: u32,
    pub enemies_remaining: u32,
    pub mission_complete: bool,
    pub boss_spawned: bool,
}

impl ElderFleetCampaignState {
    pub fn reset(&mut self) {
        self.current_mission = 0;
        self.current_wave = 0;
        self.waves_in_mission = 0;
        self.enemies_remaining = 0;
        self.mission_complete = false;
        self.boss_spawned = false;
    }

    pub fn start_mission(&mut self, mission: u32) {
        self.current_mission = mission;
        self.current_wave = 0;
        self.enemies_remaining = 0;
        self.mission_complete = false;
        self.boss_spawned = false;

        self.waves_in_mission = match mission {
            0 => 3,
            1 => 3,
            2 => 4,
            3 => 4,
            _ => 5,
        };
    }
}

// =============================================================================
// MISSION DEFINITIONS
// =============================================================================

/// Elder Fleet mission information
#[derive(Clone, Debug)]
pub struct EFMissionInfo {
    pub name: &'static str,
    pub system: &'static str,
    pub description: &'static str,
    pub boss_name: &'static str,
    pub boss_health: f32,
    pub boss_phases: u32,
    pub boss_type_id: u32,
}

/// Minmatar campaign missions (player = Minmatar, enemies = Amarr)
pub fn minmatar_missions() -> Vec<EFMissionInfo> {
    vec![
        EFMissionInfo {
            name: "First Blood",
            system: "Arzad",
            description: "Your first strike against Amarr slavers. Prove the Republic's resolve.",
            boss_name: "Squadron Leader",
            boss_health: 300.0,
            boss_phases: 2,
            boss_type_id: 597, // Punisher
        },
        EFMissionInfo {
            name: "Slave Revolt",
            system: "Hedion",
            description: "Liberate a slave transport before it reaches the processing hub.",
            boss_name: "Holder's Champion",
            boss_health: 400.0,
            boss_phases: 2,
            boss_type_id: 589, // Executioner
        },
        EFMissionInfo {
            name: "Station Assault",
            system: "Neran",
            description: "A fortified orbital station blocks our supply lines. Take it down.",
            boss_name: "Station Commander",
            boss_health: 600.0,
            boss_phases: 3,
            boss_type_id: 603, // Maller
        },
        EFMissionInfo {
            name: "Imperial Response",
            system: "Varkal",
            description: "The Empire sends a battlecruiser task force. Stand your ground.",
            boss_name: "Harbinger Captain",
            boss_health: 800.0,
            boss_phases: 3,
            boss_type_id: 624, // Harbinger
        },
        EFMissionInfo {
            name: "Empire's End",
            system: "Amarr Prime",
            description: "Strike at the heart of the Empire. End the occupation.",
            boss_name: "Imperial Admiral",
            boss_health: 1200.0,
            boss_phases: 4,
            boss_type_id: 643, // Apocalypse
        },
    ]
}

/// Amarr campaign missions (player = Amarr, enemies = Minmatar)
pub fn amarr_missions() -> Vec<EFMissionInfo> {
    vec![
        EFMissionInfo {
            name: "Insurrection Suppression",
            system: "Arzad",
            description: "Crush the Minmatar raiders before the revolt spreads.",
            boss_name: "Rifter Berserker",
            boss_health: 300.0,
            boss_phases: 2,
            boss_type_id: 587, // Rifter
        },
        EFMissionInfo {
            name: "Convoy Defense",
            system: "Hedion",
            description: "Protect the slave transport from Minmatar pirates.",
            boss_name: "Wolf Assault Lead",
            boss_health: 400.0,
            boss_phases: 2,
            boss_type_id: 11371, // Wolf
        },
        EFMissionInfo {
            name: "Border Reclamation",
            system: "Neran",
            description: "Reclaim the border station from Minmatar insurgents.",
            boss_name: "Stabber Raid Captain",
            boss_health: 600.0,
            boss_phases: 3,
            boss_type_id: 602, // Stabber
        },
        EFMissionInfo {
            name: "Heretic's Bane",
            system: "Varkal",
            description: "A Minmatar strike force threatens sacred ground.",
            boss_name: "Hurricane Warlord",
            boss_health: 800.0,
            boss_phases: 3,
            boss_type_id: 625, // Hurricane
        },
        EFMissionInfo {
            name: "Purity Restored",
            system: "Minmatar Border",
            description: "Drive the heretics from imperial space. Restore divine order.",
            boss_name: "Republic Fleet Admiral",
            boss_health: 1200.0,
            boss_phases: 4,
            boss_type_id: 639, // Tempest
        },
    ]
}

// =============================================================================
// CAMPAIGN SYSTEMS
// =============================================================================

/// Start an Elder Fleet mission
pub fn start_ef_mission(
    mut state: ResMut<ElderFleetCampaignState>,
    active: Res<crate::games::ActiveModule>,
) {
    let mission = state.current_mission;
    state.start_mission(mission);

    let faction = active.player_faction.as_deref().unwrap_or("minmatar");
    let missions = if faction == "minmatar" {
        minmatar_missions()
    } else {
        amarr_missions()
    };

    if let Some(info) = missions.get(mission as usize) {
        info!(
            "Elder Fleet: Starting mission {}: {} - {}",
            mission + 1,
            info.name,
            info.system
        );
    }
}

/// Update Elder Fleet mission state
pub fn update_ef_mission(
    mut state: ResMut<ElderFleetCampaignState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Check for mission complete (all waves done)
    if state.current_wave >= state.waves_in_mission
        && state.enemies_remaining == 0
        && !state.boss_spawned
    {
        state.boss_spawned = true;
        next_state.set(GameState::BossIntro);
    }
}

/// Check if current wave is complete
pub fn check_ef_wave_complete(
    mut state: ResMut<ElderFleetCampaignState>,
    enemies: Query<Entity, With<crate::entities::Enemy>>,
) {
    state.enemies_remaining = enemies.iter().count() as u32;
}

/// Spawn next wave of enemies for Elder Fleet
pub fn spawn_ef_wave(
    mut commands: Commands,
    mut state: ResMut<ElderFleetCampaignState>,
    active: Res<crate::games::ActiveModule>,
    sprite_cache: Res<ShipSpriteCache>,
    enemies: Query<Entity, With<crate::entities::Enemy>>,
) {
    if state.enemies_remaining > 0 || state.current_wave >= state.waves_in_mission {
        return;
    }

    if !enemies.is_empty() {
        state.enemies_remaining = enemies.iter().count() as u32;
        return;
    }

    let faction = active.player_faction.as_deref().unwrap_or("minmatar");

    // Enemy pools per faction
    let (enemy_types, variant_pool) = if faction == "minmatar" {
        // Player Minmatar → enemies Amarr
        (
            vec![597, 589, 591, 603], // Punisher, Executioner, Tormentor, Maller
            vec![
                EnemyVariant::PunisherTank,
                EnemyVariant::ExecutionerElite,
            ],
        )
    } else {
        // Player Amarr → enemies Minmatar
        (
            vec![587, 585, 598, 602], // Rifter, Slasher, Breacher, Stabber
            vec![EnemyVariant::RifterBerserker],
        )
    };

    let base_count = 4 + state.current_mission;
    let enemy_count = (base_count + state.current_wave).min(12);

    info!(
        "Elder Fleet: Spawning wave {}/{} with {} enemies",
        state.current_wave + 1,
        state.waves_in_mission,
        enemy_count
    );

    let width = crate::core::SCREEN_WIDTH;
    let height = crate::core::SCREEN_HEIGHT;
    let spawn_y = height / 2.0 + 50.0;

    for i in 0..enemy_count {
        let spread = width * 0.8;
        let start_x = -spread / 2.0;
        let x = start_x + (i as f32 / enemy_count as f32) * spread + fastrand::f32() * 40.0 - 20.0;
        let y = spawn_y + fastrand::f32() * 100.0;
        let pos = Vec2::new(x, y);

        // Chance to spawn a variant instead of base enemy
        let roll = fastrand::u32(0..100);
        let variant_threshold = 15; // 15% chance for variant

        if roll < variant_threshold && !variant_pool.is_empty() {
            let variant = variant_pool[fastrand::usize(..variant_pool.len())];
            let sprite = sprite_cache.get(variant.config().type_id);
            spawn_variant(&mut commands, variant, pos, sprite, None);
        } else {
            let type_id = enemy_types[fastrand::usize(..enemy_types.len())];
            let sprite = sprite_cache.get(type_id);

            let behavior = match fastrand::u32(0..4) {
                0 => EnemyBehavior::Linear,
                1 => EnemyBehavior::Zigzag,
                2 => EnemyBehavior::Homing,
                _ => EnemyBehavior::Weaver,
            };

            spawn_enemy(&mut commands, type_id, pos, behavior, sprite, None);
        }
    }

    state.current_wave += 1;
    state.enemies_remaining = enemy_count;
}

// =============================================================================
// BOSS SYSTEMS
// =============================================================================

/// Spawn Elder Fleet boss for current mission
pub fn spawn_ef_boss(
    mut commands: Commands,
    state: Res<ElderFleetCampaignState>,
    active: Res<crate::games::ActiveModule>,
    sprite_cache: Res<ShipSpriteCache>,
    difficulty: Res<Difficulty>,
    existing_bosses: Query<Entity, With<Boss>>,
) {
    if !existing_bosses.is_empty() {
        return;
    }

    let faction = active.player_faction.as_deref().unwrap_or("minmatar");
    let missions = if faction == "minmatar" {
        minmatar_missions()
    } else {
        amarr_missions()
    };

    let Some(info) = missions.get(state.current_mission as usize) else {
        return;
    };

    info!("Elder Fleet: Spawning boss {}", info.boss_name);

    let health = info.boss_health * difficulty.enemy_health_mult();
    let size = 100.0;

    let sprite = sprite_cache.get(info.boss_type_id);
    let boss_color = if faction == "minmatar" {
        Color::srgb(1.0, 0.84, 0.0) // Amarr gold
    } else {
        Color::srgb(0.71, 0.39, 0.20) // Minmatar rust
    };

    commands.spawn((
        Boss,
        BossData {
            id: state.current_mission + 1,
            stage: state.current_mission + 1,
            name: info.boss_name.to_string(),
            title: info.name.to_string(),
            ship_class: get_ef_ship_class(info.boss_type_id).to_string(),
            type_id: info.boss_type_id,
            max_health: health,
            health,
            current_phase: 1,
            total_phases: info.boss_phases,
            score_value: (info.boss_health * 2.0) as u64,
            liberation_value: 10,
            stationary: false,
            dialogue_intro: format!("{} has engaged!", info.boss_name),
            dialogue_defeat: format!("{} has been destroyed!", info.boss_name),
            is_enraged: false,
            enrage_threshold: 0.25,
        },
        BossState::Intro,
        BossMovement {
            pattern: MovementPattern::Descend,
            timer: 0.0,
            speed: 80.0,
        },
        BossAttack::default(),
        Hitbox { radius: size / 2.0 },
        Transform::from_xyz(0.0, 300.0, LAYER_ENEMIES),
        EnemyStats {
            type_id: info.boss_type_id,
            name: info.boss_name.to_string(),
            health,
            max_health: health,
            speed: 80.0,
            score_value: (info.boss_health * 2.0) as u64,
            is_boss: true,
            liberation_value: 10,
        },
        Sprite {
            color: if sprite.is_none() { boss_color } else { Color::WHITE },
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
    ));
}

/// Update Elder Fleet boss behaviour during BossFight
pub fn update_ef_boss(
    time: Res<Time>,
    mut boss_query: Query<
        (
            &mut Transform,
            &mut BossData,
            &mut BossMovement,
            &mut BossAttack,
            &mut BossState,
        ),
        With<Boss>,
    >,
    player_query: Query<&Transform, (With<crate::entities::Player>, Without<Boss>)>,
    mut commands: Commands,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, mut data, mut movement, mut attack, state) in boss_query.iter_mut() {
        if *state != BossState::Battle {
            continue;
        }

        let pos = transform.translation.truncate();
        let dt = time.delta_secs();
        let health_percent = data.health / data.max_health;

        // Phase transitions
        let phase_threshold = 1.0 - (data.current_phase as f32 / data.total_phases as f32);
        if health_percent <= phase_threshold && data.current_phase < data.total_phases {
            data.current_phase += 1;
            movement.speed *= 1.2;
            attack.fire_rate *= 0.85;
            info!("{} entering phase {}!", data.name, data.current_phase);
        }

        // Enrage
        if !data.is_enraged && health_percent <= data.enrage_threshold {
            data.is_enraged = true;
            movement.speed *= 1.5;
            attack.fire_rate *= 0.6;
            info!("{} is ENRAGED!", data.name);
        }

        // Movement — sweep across screen
        movement.timer += dt;
        let offset = (movement.timer * 0.5).sin() * 200.0;
        transform.translation.x = offset;

        let half_screen = crate::core::SCREEN_WIDTH / 2.0 - 120.0;
        transform.translation.x = transform.translation.x.clamp(-half_screen, half_screen);

        // Attack
        attack.fire_timer += dt;
        if attack.fire_timer >= attack.fire_rate {
            attack.fire_timer = 0.0;

            let dir = (player_pos - pos).normalize_or_zero();
            let phase = data.current_phase;
            let is_enraged = data.is_enraged;

            // Spread shot pattern (all Elder Fleet bosses)
            let bullet_count = if is_enraged { 9 } else { 5 + phase as usize };
            let base_angle = dir.y.atan2(dir.x);

            for i in 0..bullet_count {
                let angle_offset = (i as f32 - (bullet_count - 1) as f32 / 2.0) * 0.18;
                let angle = base_angle + angle_offset;
                let bullet_dir = Vec2::new(angle.cos(), angle.sin());
                let projectile_speed = 220.0 + phase as f32 * 30.0;
                let damage = 12.0 + phase as f32 * 4.0;

                let color = if data.type_id == 643 || data.type_id == 639 {
                    // Apocalypse / Tempest — heavier shot
                    Color::srgb(1.0, 0.2, 0.2)
                } else {
                    Color::srgb(1.0, 0.5, 0.2)
                };

                spawn_ef_projectile(
                    &mut commands,
                    pos + bullet_dir * 40.0,
                    bullet_dir,
                    projectile_speed,
                    damage,
                    color,
                );
            }
        }
    }
}

/// Spawn Elder Fleet boss projectile
fn spawn_ef_projectile(
    commands: &mut Commands,
    pos: Vec2,
    dir: Vec2,
    speed: f32,
    damage: f32,
    color: Color,
) {
    commands.spawn((
        crate::entities::EnemyProjectile,
        crate::entities::ProjectileDamage {
            damage,
            damage_type: DamageType::Thermal,
            crit_chance: 0.05,
            crit_multiplier: 1.5,
            ammo_type: crate::core::AmmoType::default(),
        },
        ProjectilePhysics {
            velocity: dir * speed,
            lifetime: 4.0,
        },
        Sprite {
            color,
            custom_size: Some(Vec2::new(8.0, 16.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 9.0),
    ));
}

/// Check if Elder Fleet boss is defeated
pub fn check_ef_boss_defeated(
    mut state: ResMut<ElderFleetCampaignState>,
    mut next_state: ResMut<NextState<GameState>>,
    bosses: Query<Entity, With<Boss>>,
) {
    if bosses.is_empty() && state.boss_spawned {
        state.mission_complete = true;
        state.current_mission += 1;

        let total_missions = 5;
        if state.current_mission >= total_missions {
            next_state.set(GameState::Victory);
        } else {
            next_state.set(GameState::StageComplete);
        }
    }
}

// =============================================================================
// HELPERS
// =============================================================================

fn get_ef_ship_class(type_id: u32) -> &'static str {
    match type_id {
        587 => "Rifter Frigate",
        585 => "Slasher Interceptor",
        598 => "Breacher Assault",
        602 => "Stabber Cruiser",
        11371 => "Wolf Assault Frigate",
        625 => "Hurricane Battlecruiser",
        639 => "Tempest Battleship",
        597 => "Punisher Frigate",
        589 => "Executioner Interceptor",
        591 => "Tormentor Assault",
        603 => "Maller Cruiser",
        624 => "Harbinger Battlecruiser",
        643 => "Apocalypse Battleship",
        _ => "Unknown",
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minmatar_missions_has_five_entries() {
        let missions = minmatar_missions();
        assert_eq!(missions.len(), 5);
    }

    #[test]
    fn amarr_missions_has_five_entries() {
        let missions = amarr_missions();
        assert_eq!(missions.len(), 5);
    }

    #[test]
    fn campaign_state_resets_correctly() {
        let mut state = ElderFleetCampaignState::default();
        state.current_mission = 3;
        state.current_wave = 4;
        state.boss_spawned = true;

        state.reset();

        assert_eq!(state.current_mission, 0);
        assert_eq!(state.current_wave, 0);
        assert!(!state.boss_spawned);
    }

    #[test]
    fn start_mission_sets_waves_correctly() {
        let mut state = ElderFleetCampaignState::default();

        state.start_mission(0);
        assert_eq!(state.waves_in_mission, 3);

        state.start_mission(2);
        assert_eq!(state.waves_in_mission, 4);

        state.start_mission(4);
        assert_eq!(state.waves_in_mission, 5);
    }

    #[test]
    fn get_ef_ship_class_maps_correctly() {
        assert_eq!(get_ef_ship_class(587), "Rifter Frigate");
        assert_eq!(get_ef_ship_class(597), "Punisher Frigate");
        assert_eq!(get_ef_ship_class(643), "Apocalypse Battleship");
        assert_eq!(get_ef_ship_class(9999), "Unknown");
    }
}
