//! Triglavian Invasion Campaign
//!
//! 9-mission campaign across contested systems.

use super::ships::*;
use crate::assets::ShipSpriteCache;
use crate::core::{DamageType, GameState, LAYER_ENEMIES};
use crate::entities::boss::{Boss, BossAttack, BossData, BossMovement, BossState, MovementPattern};
use crate::entities::projectile::ProjectilePhysics;
use crate::entities::Hitbox;
use crate::entities::{spawn_enemy, spawn_variant, EnemyBehavior, EnemyVariant};
use bevy::prelude::*;

/// Campaign state resource
#[derive(Resource, Default)]
pub struct TriglavianCampaignState {
    pub current_mission: u32,
    pub current_wave: u32,
    pub waves_in_mission: u32,
    pub enemies_remaining: u32,
    pub mission_complete: bool,
    pub boss_spawned: bool,
}

impl TriglavianCampaignState {
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

        // Waves per mission (increases with difficulty)
        self.waves_in_mission = match mission {
            0..=2 => 3,
            3..=5 => 4,
            _ => 5,
        };
    }
}

// =============================================================================
// MISSION DEFINITIONS
// =============================================================================

/// Mission information
#[derive(Clone, Debug)]
pub struct MissionInfo {
    pub name: &'static str,
    pub system: &'static str,
    pub description: &'static str,
    pub boss_type_id: u32,
    pub boss_name: &'static str,
}

/// Get mission info for EDENCOM campaign
pub fn edencom_missions() -> Vec<MissionInfo> {
    vec![
        MissionInfo {
            name: "First Contact",
            system: "Niarja",
            description: "Triglavian scouts have been spotted. Intercept and eliminate.",
            boss_type_id: triglavian::VEDMAK,
            boss_name: "Raznaborg Vedmak",
        },
        MissionInfo {
            name: "Stellar Transmuter",
            system: "Raravoss",
            description: "Prevent the Triglavians from deploying their stellar harvester.",
            boss_type_id: triglavian::DREKAVAC,
            boss_name: "Perun Drekavac",
        },
        MissionInfo {
            name: "Liminality Rising",
            system: "Sakenta",
            description: "The system is approaching Final Liminality. Hold the line.",
            boss_type_id: triglavian::LESHAK,
            boss_name: "Veles Leshak",
        },
        MissionInfo {
            name: "Pochven's Edge",
            system: "Vale",
            description: "Systems are falling to Pochven. We must not lose Vale.",
            boss_type_id: triglavian::IKITURSA,
            boss_name: "Svarog Ikitursa",
        },
        MissionInfo {
            name: "Proving Ground",
            system: "Kuharah",
            description: "The Collective seeks to prove their might. Show them ours.",
            boss_type_id: triglavian::DREKAVAC,
            boss_name: "Proving Drekavac",
        },
        MissionInfo {
            name: "Gate Defense",
            system: "Otela",
            description: "Defend the stargate from Triglavian assault.",
            boss_type_id: triglavian::LESHAK,
            boss_name: "Siege Leshak",
        },
        MissionInfo {
            name: "World Ark Assault",
            system: "Kino",
            description: "A World Ark has been detected. This is our chance.",
            boss_type_id: triglavian::XORDAZH,
            boss_name: "Xordazh World Ark",
        },
        MissionInfo {
            name: "Zorya's Domain",
            system: "Pochven Gate",
            description: "Push into Pochven and strike at the heart of the invasion.",
            boss_type_id: triglavian::LESHAK,
            boss_name: "Zorya's Champion",
        },
        MissionInfo {
            name: "The Final Proving",
            system: "Triglavian Stronghold",
            description: "End the invasion. Victory or death.",
            boss_type_id: triglavian::XORDAZH,
            boss_name: "Zorya Triglav",
        },
    ]
}

/// Get mission info for Triglavian campaign
pub fn triglavian_missions() -> Vec<MissionInfo> {
    vec![
        MissionInfo {
            name: "Glory to the Collective",
            system: "Niarja",
            description: "Prove yourself worthy by destroying EDENCOM scouts.",
            boss_type_id: edencom::SKYBREAKER,
            boss_name: "EDENCOM Vanguard",
        },
        MissionInfo {
            name: "Stellar Manipulation",
            system: "Raravoss",
            description: "Protect the Stellar Transmuter from EDENCOM interference.",
            boss_type_id: edencom::THUNDERCHILD,
            boss_name: "EDENCOM Commander",
        },
        MissionInfo {
            name: "Embrace Liminality",
            system: "Sakenta",
            description: "Push the system toward Final Liminality.",
            boss_type_id: edencom::STORMBRINGER,
            boss_name: "EDENCOM Battlegroup",
        },
        MissionInfo {
            name: "Claim for Pochven",
            system: "Vale",
            description: "Vale will join Pochven. Eliminate all resistance.",
            boss_type_id: edencom::THUNDERCHILD,
            boss_name: "Imperial Thunderchild",
        },
        MissionInfo {
            name: "The Flow of Vyraj",
            system: "Kuharah",
            description: "Demonstrate superiority in the Proving.",
            boss_type_id: edencom::STORMBRINGER,
            boss_name: "CONCORD Stormbringer",
        },
        MissionInfo {
            name: "Gate Seizure",
            system: "Otela",
            description: "Capture the stargate for Collective use.",
            boss_type_id: edencom::APOCALYPSE,
            boss_name: "Amarr Dreadnought",
        },
        MissionInfo {
            name: "Empire's Fall",
            system: "Kino",
            description: "Break the empire fleet defending this system.",
            boss_type_id: edencom::RAVEN,
            boss_name: "Caldari Fleet Commander",
        },
        MissionInfo {
            name: "Weaving Pochven",
            system: "New Eden Core",
            description: "The wormhole network must grow. Destroy all obstacles.",
            boss_type_id: edencom::MEGATHRON,
            boss_name: "Gallente Titan Escort",
        },
        MissionInfo {
            name: "Totality",
            system: "Jita",
            description: "All of New Eden shall join Pochven.",
            boss_type_id: edencom::STORMBRINGER,
            boss_name: "EDENCOM Supreme Commander",
        },
    ]
}

// =============================================================================
// CAMPAIGN SYSTEMS
// =============================================================================

/// Start a Triglavian campaign mission
pub fn start_trig_mission(
    mut state: ResMut<TriglavianCampaignState>,
    active: Res<crate::games::ActiveModule>,
) {
    let mission = state.current_mission;
    state.start_mission(mission);

    let faction = active.player_faction.as_deref().unwrap_or("edencom");
    let missions = if faction == "edencom" {
        edencom_missions()
    } else {
        triglavian_missions()
    };

    if let Some(info) = missions.get(mission as usize) {
        info!(
            "Starting mission {}: {} - {}",
            mission + 1,
            info.name,
            info.system
        );
    }
}

/// Update mission state
pub fn update_trig_mission(
    mut state: ResMut<TriglavianCampaignState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Check for mission complete (all waves done)
    if state.current_wave >= state.waves_in_mission
        && state.enemies_remaining == 0
        && !state.boss_spawned
    {
        // Transition to boss fight
        state.boss_spawned = true;
        next_state.set(GameState::BossIntro);
    }
}

/// Check if current wave is complete
pub fn check_trig_wave_complete(
    mut state: ResMut<TriglavianCampaignState>,
    enemies: Query<Entity, With<crate::entities::Enemy>>,
) {
    state.enemies_remaining = enemies.iter().count() as u32;
}

/// Spawn next wave of enemies
pub fn spawn_trig_wave(
    mut commands: Commands,
    mut state: ResMut<TriglavianCampaignState>,
    active: Res<crate::games::ActiveModule>,
    sprite_cache: Res<ShipSpriteCache>,
    enemies: Query<Entity, With<crate::entities::Enemy>>,
    windows: Query<&Window>,
) {
    // Only spawn if no enemies and wave not complete
    if state.enemies_remaining > 0 || state.current_wave >= state.waves_in_mission {
        return;
    }

    // Don't spawn if there are still enemies alive
    if !enemies.is_empty() {
        state.enemies_remaining = enemies.iter().count() as u32;
        return;
    }

    let faction = active.player_faction.as_deref().unwrap_or("edencom");
    let spawn_weights = if faction == "edencom" {
        triglavian_spawn_weights()
    } else {
        edencom_spawn_weights()
    };

    // Calculate total weight for random selection
    let total_weight: u32 = spawn_weights.iter().map(|(_, w)| w).sum();

    // Calculate enemies for this wave
    let base_count = 4 + state.current_mission;
    let wave_bonus = state.current_wave;
    let enemy_count = (base_count + wave_bonus).min(12); // Cap at 12 enemies per wave

    info!(
        "Spawning wave {}/{} with {} enemies",
        state.current_wave + 1,
        state.waves_in_mission,
        enemy_count
    );

    // Get window dimensions for spawn positioning
    let window = windows.single();
    let width = window.width();
    let height = window.height();
    let spawn_y = height / 2.0 + 50.0; // Spawn above screen

    // Spawn enemies
    for i in 0..enemy_count {
        // Weighted random selection
        let roll = fastrand::u32(0..total_weight);
        let mut cumulative = 0;
        let mut selected_type_id = spawn_weights[0].0;

        for (type_id, weight) in &spawn_weights {
            cumulative += weight;
            if roll < cumulative {
                selected_type_id = *type_id;
                break;
            }
        }

        // Position spread across screen
        let spread = width * 0.8;
        let start_x = -spread / 2.0;
        let x = start_x + (i as f32 / enemy_count as f32) * spread + fastrand::f32() * 40.0 - 20.0;
        let y = spawn_y + fastrand::f32() * 100.0;
        let pos = Vec2::new(x, y);

        let sprite = sprite_cache.get(selected_type_id);

        // Use specialized spawn functions for Triglavian ships (they have DisintegratorRamp)
        if faction == "edencom" {
            // Player is EDENCOM, enemies are Triglavian
            match selected_type_id {
                triglavian::DAMAVIK => {
                    spawn_variant(&mut commands, EnemyVariant::Damavik, pos, sprite, None);
                }
                triglavian::VEDMAK => {
                    spawn_variant(&mut commands, EnemyVariant::Vedmak, pos, sprite, None);
                }
                _ => {
                    // Generic enemy spawn for other Triglavian ships
                    spawn_enemy(
                        &mut commands,
                        selected_type_id,
                        pos,
                        EnemyBehavior::Disintegrator,
                        sprite,
                        None,
                    );
                }
            }
        } else {
            // Player is Triglavian, enemies are EDENCOM/Empire
            let behavior = match selected_type_id {
                // EDENCOM ships - chain lightning behavior (use Sniper for now)
                edencom::SKYBREAKER | edencom::THUNDERCHILD | edencom::STORMBRINGER => {
                    EnemyBehavior::Sniper
                }
                // Empire frigates - aggressive
                edencom::RIFTER | edencom::MERLIN | edencom::PUNISHER | edencom::INCURSUS => {
                    EnemyBehavior::Zigzag
                }
                // Empire cruisers - steady
                edencom::STABBER | edencom::CARACAL | edencom::OMEN | edencom::THORAX => {
                    EnemyBehavior::Linear
                }
                // Empire battleships - tanky
                _ => EnemyBehavior::Tank,
            };
            spawn_enemy(&mut commands, selected_type_id, pos, behavior, sprite, None);
        }
    }

    state.current_wave += 1;
    state.enemies_remaining = enemy_count;
}

// =============================================================================
// BOSS SYSTEMS
// =============================================================================

/// Spawn mission boss
pub fn spawn_trig_boss(
    mut commands: Commands,
    state: Res<TriglavianCampaignState>,
    active: Res<crate::games::ActiveModule>,
    sprite_cache: Res<ShipSpriteCache>,
    windows: Query<&Window>,
    existing_bosses: Query<Entity, With<Boss>>,
) {
    // Don't spawn if boss already exists
    if !existing_bosses.is_empty() {
        return;
    }

    let faction = active.player_faction.as_deref().unwrap_or("edencom");
    let missions = if faction == "edencom" {
        edencom_missions()
    } else {
        triglavian_missions()
    };

    let Some(info) = missions.get(state.current_mission as usize) else {
        return;
    };

    info!("Spawning boss: {} ({})", info.boss_name, info.boss_type_id);

    // Get boss stats based on type
    let (health, score, scale) = match info.boss_type_id {
        // Triglavian bosses (when player is EDENCOM)
        triglavian::VEDMAK => (400.0, 800, 2.0),
        triglavian::DREKAVAC => (600.0, 1200, 2.5),
        triglavian::LESHAK => (1000.0, 2000, 3.0),
        triglavian::IKITURSA => (500.0, 1000, 2.2),
        triglavian::XORDAZH => (3000.0, 5000, 5.0), // World Ark - massive
        // EDENCOM/Empire bosses (when player is Triglavian)
        edencom::SKYBREAKER => (300.0, 600, 1.8),
        edencom::THUNDERCHILD => (500.0, 1000, 2.5),
        edencom::STORMBRINGER => (900.0, 1800, 3.0),
        edencom::APOCALYPSE => (800.0, 1600, 3.5),
        edencom::RAVEN => (700.0, 1400, 3.0),
        edencom::MEGATHRON => (750.0, 1500, 3.2),
        _ => (500.0, 1000, 2.5),
    };

    // Get window dimensions for spawn positioning
    let window = windows.single();
    let spawn_y = window.height() / 2.0 + 100.0; // Start above screen

    let sprite = sprite_cache.get(info.boss_type_id);
    let size = 64.0 * scale;

    // Create boss entity
    let mut entity = commands.spawn((
        Boss,
        BossData {
            id: state.current_mission + 1,
            stage: state.current_mission + 1,
            name: info.boss_name.to_string(),
            title: info.name.to_string(),
            ship_class: get_ship_class_name(info.boss_type_id).to_string(),
            type_id: info.boss_type_id,
            max_health: health,
            health,
            current_phase: 1,
            total_phases: 3,
            score_value: score,
            liberation_value: 10,
            stationary: info.boss_type_id == triglavian::XORDAZH, // World Ark is stationary
            dialogue_intro: format!("{} has engaged!", info.boss_name),
            dialogue_defeat: format!("{} has been destroyed!", info.boss_name),
            is_enraged: false,
            enrage_threshold: 0.2,
        },
        BossState::Intro,
        BossMovement {
            pattern: MovementPattern::Descend,
            timer: 0.0,
            speed: 80.0,
        },
        BossAttack::default(),
        Hitbox { radius: size / 2.0 },
        Transform::from_xyz(0.0, spawn_y, LAYER_ENEMIES),
    ));

    // Add sprite
    if let Some(image) = sprite {
        entity.insert(Sprite {
            image,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        });
    } else {
        // Fallback colored square
        entity.insert(Sprite {
            color: if faction == "edencom" {
                Color::srgb(0.8, 0.2, 0.2) // Triglavian red
            } else {
                Color::srgb(0.2, 0.6, 0.9) // EDENCOM blue
            },
            custom_size: Some(Vec2::splat(size)),
            ..default()
        });
    }

    // Add disintegrator for Triglavian bosses
    if faction == "edencom" {
        entity.insert(crate::entities::DisintegratorRamp::new(
            12.0, // Base damage (boss-level)
            2.5,  // Max multiplier
            8.0,  // Ramp time
        ));
    }
}

/// Get ship class name from type_id
fn get_ship_class_name(type_id: u32) -> &'static str {
    match type_id {
        triglavian::DAMAVIK => "Frigate",
        triglavian::KIKIMORA => "Destroyer",
        triglavian::VEDMAK | triglavian::IKITURSA => "Cruiser",
        triglavian::DREKAVAC => "Battlecruiser",
        triglavian::LESHAK => "Battleship",
        triglavian::XORDAZH => "World Ark",
        edencom::SKYBREAKER => "Frigate",
        edencom::THUNDERCHILD => "Cruiser",
        edencom::STORMBRINGER | edencom::APOCALYPSE | edencom::RAVEN | edencom::MEGATHRON => {
            "Battleship"
        }
        _ => "Unknown",
    }
}

// =============================================================================
// BOSS INTRO SEQUENCE
// =============================================================================

/// Boss intro sequence — descend boss into position then start fight
pub fn trig_boss_intro(
    time: Res<Time>,
    mut boss_query: Query<(&mut Transform, &mut BossState, &BossData), With<Boss>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: Local<f32>,
) {
    let dt = time.delta_secs();
    let mut boss_count = 0;

    for (mut transform, mut state, data) in boss_query.iter_mut() {
        boss_count += 1;

        // Descend boss to battle position
        let target_y = 200.0;
        if transform.translation.y > target_y {
            transform.translation.y -= 100.0 * dt;
        }

        // After 2 seconds, start fight
        if *timer > 2.0 {
            *timer = 0.0;
            *state = BossState::Battle;
            next_state.set(GameState::BossFight);
            info!("Triglavian Boss battle started: {}", data.name);
        }
    }

    if boss_count > 0 {
        *timer += dt;
    }
}

/// Despawn Triglavian boss intro UI (placeholder — no UI yet)
pub fn despawn_trig_boss_intro(_commands: Commands) {
    // Intentionally empty; intro UI will be added in future update
}

// =============================================================================
// BOSS UPDATE
// =============================================================================

/// Update boss behavior during BossFight
pub fn update_trig_boss(
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

        // -----------------------------------------------------------------
        // PHASE TRANSITIONS
        // -----------------------------------------------------------------
        let phase_threshold = 1.0 - (data.current_phase as f32 / data.total_phases as f32);

        if health_percent <= phase_threshold && data.current_phase < data.total_phases {
            data.current_phase += 1;
            movement.speed *= 1.2;
            attack.fire_rate *= 0.85;
            info!(
                "{} entering phase {}!",
                data.name, data.current_phase
            );
        }

        // -----------------------------------------------------------------
        // ENRAGE
        // -----------------------------------------------------------------
        if !data.is_enraged && health_percent <= data.enrage_threshold {
            data.is_enraged = true;
            movement.speed *= 1.5;
            attack.fire_rate *= 0.6;
            info!("{} is ENRAGED!", data.name);
        }

        // -----------------------------------------------------------------
        // MOVEMENT
        // -----------------------------------------------------------------
        if data.stationary {
            // World Ark stays in place
            continue;
        }

        movement.timer += dt;
        let _sweep_speed = movement.speed * (1.0 + data.current_phase as f32 * 0.15);
        let offset = (movement.timer * 0.5).sin() * 200.0;
        transform.translation.x = offset;

        // Clamp to screen bounds
        let half_screen = crate::core::SCREEN_WIDTH / 2.0 - 120.0;
        transform.translation.x = transform.translation.x.clamp(-half_screen, half_screen);

        // -----------------------------------------------------------------
        // ATTACK
        // -----------------------------------------------------------------
        attack.fire_timer += dt;
        if attack.fire_timer >= attack.fire_rate {
            attack.fire_timer = 0.0;

            let dir = (player_pos - pos).normalize_or_zero();
            let phase = data.current_phase;
            let is_enraged = data.is_enraged;

            // Choose attack pattern based on boss type and phase
            match data.type_id {
                // World Ark — ring attack
                triglavian::XORDAZH => {
                    let count = 16 + (phase * 4) as usize;
                    for i in 0..count {
                        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                        let bullet_dir = Vec2::new(angle.cos(), angle.sin());
                        spawn_trig_projectile(
                            &mut commands,
                            pos,
                            bullet_dir,
                            120.0,
                            12.0 + phase as f32 * 3.0,
                            Color::srgb(0.8, 0.2, 0.2),
                        );
                    }
                }

                // Leshak / Ikitursa — spread shot
                triglavian::LESHAK | triglavian::IKITURSA => {
                    let bullet_count = if is_enraged { 9 } else { 5 };
                    let base_angle = dir.y.atan2(dir.x);
                    for i in 0..bullet_count {
                        let angle_offset = (i as f32 - (bullet_count - 1) as f32 / 2.0) * 0.2;
                        let angle = base_angle + angle_offset;
                        let bullet_dir = Vec2::new(angle.cos(), angle.sin());
                        spawn_trig_projectile(
                            &mut commands,
                            pos + bullet_dir * 40.0,
                            bullet_dir,
                            220.0 + phase as f32 * 30.0,
                            18.0 + phase as f32 * 4.0,
                            Color::srgb(0.9, 0.3, 0.1),
                        );
                    }
                }

                // Vedmak / Drekavac — steady aimed shot
                _ => {
                    let projectile_speed = 250.0 + phase as f32 * 40.0;
                    let damage = 15.0 + phase as f32 * 5.0;
                    spawn_trig_projectile(
                        &mut commands,
                        pos + dir * 40.0,
                        dir,
                        projectile_speed,
                        damage,
                        Color::srgb(1.0, 0.5, 0.2),
                    );
                }
            }
        }
    }
}

/// Spawn a Triglavian boss projectile
fn spawn_trig_projectile(
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

/// Check if boss is defeated
pub fn check_trig_boss_defeated(
    mut state: ResMut<TriglavianCampaignState>,
    mut next_state: ResMut<NextState<GameState>>,
    bosses: Query<Entity, With<crate::entities::boss::Boss>>,
) {
    if bosses.is_empty() && state.boss_spawned {
        state.mission_complete = true;
        state.current_mission += 1;

        // Check for campaign complete
        let total_missions = 9;
        if state.current_mission >= total_missions {
            next_state.set(GameState::Victory);
        } else {
            next_state.set(GameState::StageComplete);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edencom_missions_has_nine_entries() {
        let missions = edencom_missions();
        assert_eq!(missions.len(), 9);
    }

    #[test]
    fn triglavian_missions_has_nine_entries() {
        let missions = triglavian_missions();
        assert_eq!(missions.len(), 9);
    }

    #[test]
    fn campaign_state_resets_correctly() {
        let mut state = TriglavianCampaignState::default();
        state.current_mission = 5;
        state.current_wave = 3;
        state.waves_in_mission = 4;
        state.enemies_remaining = 10;
        state.mission_complete = true;
        state.boss_spawned = true;

        state.reset();

        assert_eq!(state.current_mission, 0);
        assert_eq!(state.current_wave, 0);
        assert_eq!(state.waves_in_mission, 0);
        assert_eq!(state.enemies_remaining, 0);
        assert!(!state.mission_complete);
        assert!(!state.boss_spawned);
    }

    #[test]
    fn start_mission_sets_waves_based_on_difficulty() {
        let mut state = TriglavianCampaignState::default();

        // Early missions: 3 waves
        state.start_mission(0);
        assert_eq!(state.waves_in_mission, 3);
        state.start_mission(2);
        assert_eq!(state.waves_in_mission, 3);

        // Mid missions: 4 waves
        state.start_mission(3);
        assert_eq!(state.waves_in_mission, 4);
        state.start_mission(5);
        assert_eq!(state.waves_in_mission, 4);

        // Late missions: 5 waves
        state.start_mission(6);
        assert_eq!(state.waves_in_mission, 5);
        state.start_mission(8);
        assert_eq!(state.waves_in_mission, 5);
    }

    #[test]
    fn mission_names_are_unique() {
        let edencom = edencom_missions();
        let triglavian = triglavian_missions();

        let edencom_names: Vec<&str> = edencom.iter().map(|m| m.name).collect();
        let triglavian_names: Vec<&str> = triglavian.iter().map(|m| m.name).collect();

        // Within each faction, names should be unique
        let mut unique = edencom_names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), edencom_names.len());

        let mut unique = triglavian_names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), triglavian_names.len());
    }

    #[test]
    fn boss_type_ids_are_valid() {
        let edencom = edencom_missions();
        let triglavian = triglavian_missions();

        for mission in &edencom {
            assert!(
                get_ship_class_name(mission.boss_type_id) != "Unknown",
                "Unknown boss type_id {} for mission {}",
                mission.boss_type_id,
                mission.name
            );
        }

        for mission in &triglavian {
            assert!(
                get_ship_class_name(mission.boss_type_id) != "Unknown",
                "Unknown boss type_id {} for mission {}",
                mission.boss_type_id,
                mission.name
            );
        }
    }
}
