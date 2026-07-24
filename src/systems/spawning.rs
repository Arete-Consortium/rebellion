//! Enemy Spawning System
//!
//! Handles wave-based enemy spawning with carrier visuals.
//! Enemy waves launch from faction-appropriate carriers in the background.

use super::dialogue::{DialogueEvent, DialogueSystem};
use crate::assets::ShipModelCache;
use crate::core::*;
use crate::entities::{spawn_enemy, spawn_variant, EnemyBehavior, EnemyVariant};
use crate::games::caldari_gallente::LastStandState;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// Spawning plugin
pub struct SpawningPlugin;

impl Plugin for SpawningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveManager>()
            .add_systems(
                OnEnter(GameState::Playing),
                (reset_wave_manager, spawn_enemy_carrier)
                    .run_if(not_last_stand)
                    .run_if(not_abyssal),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_carrier)
            .add_systems(
                FixedUpdate,
                (
                    wave_spawning,
                    handle_spawn_events,
                    animate_carrier,
                    carrier_launch_fighters,
                    tick_carrier_flash,
                )
                    .run_if(in_state(GameState::Playing))
                    .run_if(not_last_stand)
                    .run_if(not_abyssal),
            );
    }
}

/// Run condition: Last Stand mode is NOT active
fn not_last_stand(last_stand: Option<Res<LastStandState>>) -> bool {
    last_stand.map(|ls| !ls.active).unwrap_or(true)
}

/// Run condition: Abyssal Depths is NOT active
fn not_abyssal(abyssal: Option<Res<crate::games::abyssal_depths::AbyssalState>>) -> bool {
    abyssal.map(|a| !a.active).unwrap_or(true)
}

/// Marker component for the enemy carrier in background
#[derive(Component)]
pub struct EnemyCarrier {
    /// Base Y position
    pub base_y: f32,
    /// Animation timer
    pub timer: f32,
    /// Warp-in progress (0.0 = warping, 1.0 = arrived)
    pub warp_progress: f32,
    /// Cooldown before next fighter launch
    pub launch_cooldown: Timer,
}

/// Brief flash/glow effect spawned at each fighter launch point.
#[derive(Component)]
pub struct CarrierLaunchFlash {
    pub life: f32,
    pub max: f32,
}

/// Spawn the enemy faction's carrier in the background
fn spawn_enemy_carrier(
    mut commands: Commands,
    session: Res<GameSession>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
) {
    let carrier_id = session.enemy_faction.carrier_type_id();
    let sprite = sprite_cache.get(carrier_id);

    // Position carrier in upper background
    let carrier_y = SCREEN_HEIGHT / 2.0 - 100.0;

    // Carrier size from constants
    let carrier_size = crate::core::SIZE_CARRIER;

    let mut entity = commands.spawn((
        EnemyCarrier {
            base_y: carrier_y,
            timer: 0.0,
            warp_progress: 0.0, // Start warping in
            launch_cooldown: Timer::from_seconds(4.5, TimerMode::Once),
        },
        Transform::from_xyz(0.0, carrier_y + 200.0, -50.0), // Start above screen, z=-50 for background
        Visibility::Visible,
        Name::new("EnemyCarrier"),
    ));

    // Add sprite - carrier faces down (rotated 180° + ship-specific correction)
    if let Some(texture) = sprite {
        // Get rotation: 180° base (face down) + per-ship correction
        let base_rotation = std::f32::consts::PI;
        let correction = crate::entities::get_ship_rotation_correction(carrier_id);
        let total_rotation = base_rotation + correction;

        entity.insert((Sprite {
            image: texture,
            color: Color::srgba(1.0, 1.0, 1.0, 0.0), // Start invisible for warp-in
            custom_size: Some(Vec2::splat(carrier_size)),
            ..default()
        },));
        entity.insert(
            Transform::from_xyz(0.0, carrier_y + 200.0, -50.0)
                .with_rotation(Quat::from_rotation_z(total_rotation)),
        );
    }

    info!(
        "Enemy {} carrier warping in!",
        session.enemy_faction.short_name()
    );
}

/// Animate the carrier — warp-in, then slow vertical scroll so the hull
/// treadmills past the player, selling "we're flying across the deck."
fn animate_carrier(
    time: Res<Time>,
    mut carrier_query: Query<(&mut EnemyCarrier, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    // Screen is 700 tall + carrier is 1800 tall — reset point keeps carrier
    // always covering the screen vertically.
    let carrier_half = crate::core::SIZE_CARRIER / 2.0;
    let screen_half = crate::core::SCREEN_HEIGHT / 2.0;
    // Scroll speed: slow and steady. Tuned so a full hull pass ≈ 45s.
    const HULL_SCROLL_SPEED: f32 = 40.0;

    for (mut carrier, mut transform, mut sprite) in carrier_query.iter_mut() {
        carrier.timer += dt;

        // Warp-in animation (first 2 seconds) — hull materializes into place,
        // centered so it covers the screen before the scroll begins.
        if carrier.warp_progress < 1.0 {
            carrier.warp_progress = (carrier.warp_progress + dt * 0.5).min(1.0);

            // Target = carrier centered so bow is above screen, mid-hull
            // visible. Bias up 0.25×half so the bow enters first.
            let target_y = carrier_half * 0.25;
            let start_y = target_y + 200.0;
            transform.translation.y =
                start_y + (target_y - start_y) * ease_out_cubic(carrier.warp_progress);
            carrier.base_y = target_y;

            let alpha = carrier.warp_progress * 0.9;
            let warp_tint = 1.0 - (1.0 - carrier.warp_progress) * 0.3;
            sprite.color = Color::srgba(warp_tint, warp_tint, 1.0, alpha);
        } else {
            // Continuous scroll downward so the hull treadmills past.
            transform.translation.y -= HULL_SCROLL_SPEED * dt;
            // Wrap: when carrier's top edge drops below screen bottom, loop
            // back up with bottom edge just above screen top.
            if transform.translation.y + carrier_half < -screen_half {
                transform.translation.y = screen_half + carrier_half;
            }
            sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.9);
        }
    }
}

/// Ease out cubic for smooth deceleration
fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// Pick the appropriate fighter frigate type_id for the enemy faction.
fn fighter_type_for(faction: Faction) -> u32 {
    match faction {
        Faction::Caldari => 603,  // Merlin
        Faction::Gallente => 594, // Incursus
        Faction::Amarr => 597,    // Punisher
        Faction::Minmatar => 587, // Rifter
    }
}

/// Periodically launch a WING of fighters from the background carrier —
/// multiple ships at once with staggered hangar-bay flashes.
fn carrier_launch_fighters(
    time: Res<Time>,
    mut commands: Commands,
    session: Res<GameSession>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    mut carriers: Query<(&Transform, &mut EnemyCarrier)>,
) {
    let delta = time.delta();
    for (carrier_t, mut carrier) in carriers.iter_mut() {
        // Wait until carrier has fully warped in
        if carrier.warp_progress < 1.0 {
            continue;
        }
        carrier.launch_cooldown.tick(delta);
        if !carrier.launch_cooldown.finished() {
            continue;
        }
        // Reset cooldown — 2.5-4.5 seconds between wings (was 5-8s for single).
        let next = 2.5 + fastrand::f32() * 2.0;
        carrier.launch_cooldown = Timer::from_seconds(next, TimerMode::Once);

        let fighter_type = fighter_type_for(session.enemy_faction);
        let sprite = sprite_cache.get(fighter_type);

        // Wing size: 2-4 fighters. Spread them across the carrier's hangar
        // footprint so it reads as simultaneous fleet deployment.
        let wing_size = 2 + fastrand::u32(0..3);
        let carrier_half = crate::core::SIZE_CARRIER * 0.35;
        for i in 0..wing_size {
            let t = if wing_size > 1 {
                i as f32 / (wing_size - 1) as f32
            } else {
                0.5
            };
            let offset_x = carrier_half * (t - 0.5);
            let jitter_x = (fastrand::f32() - 0.5) * 40.0;
            let jitter_y = fastrand::f32() * 30.0;
            // Hull is now the full-screen backdrop — launch fighters from
            // just above the screen top regardless of carrier scroll phase so
            // every wave stays visible to the player.
            let launch_y = crate::core::SCREEN_HEIGHT / 2.0 + 40.0 - jitter_y;
            let launch_pos = Vec2::new(carrier_t.translation.x + offset_x + jitter_x, launch_y);

            let _fighter_entity = crate::entities::enemy::spawn_enemy(
                &mut commands,
                fighter_type,
                launch_pos,
                crate::entities::enemy::EnemyBehavior::Homing,
                sprite.clone(),
                None,
            );

            // Hangar-bay flash at each launch point — big and warm
            commands.spawn((
                CarrierLaunchFlash {
                    life: 0.6,
                    max: 0.6,
                },
                Sprite {
                    color: Color::srgba(1.0, 0.85, 0.45, 1.0),
                    custom_size: Some(Vec2::splat(120.0)),
                    ..default()
                },
                Transform::from_xyz(launch_pos.x, launch_pos.y, -40.0),
            ));
        }

        info!(
            "Carrier launched wing of {} {} fighters",
            wing_size,
            session.enemy_faction.short_name()
        );
    }
}

/// Fade and expand the launch-flash sprite, then despawn.
fn tick_carrier_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<(Entity, &mut CarrierLaunchFlash, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (e, mut f, mut s, mut t) in q.iter_mut() {
        f.life -= dt;
        if f.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        let k = f.life / f.max; // 1.0 → 0.0
        s.color.set_alpha(k);
        t.scale = Vec3::splat(0.6 + (1.0 - k) * 1.4);
    }
}

/// Cleanup carrier when leaving playing state
fn cleanup_carrier(mut commands: Commands, carrier_query: Query<Entity, With<EnemyCarrier>>) {
    for entity in carrier_query.iter() {
        if let Some(ec) = commands.get_entity(entity) {
            ec.despawn_recursive();
        }
    }
}

/// Manages wave spawning
#[derive(Resource, Debug)]
pub struct WaveManager {
    /// Current wave number (within stage)
    pub wave: u32,
    /// Waves per stage before boss
    pub waves_per_stage: u32,
    /// Current stage (1-13)
    pub current_stage: u32,
    /// Enemies remaining in current wave
    pub enemies_remaining: u32,
    /// Time until next spawn
    pub spawn_timer: f32,
    /// Time between spawns
    pub spawn_interval: f32,
    /// Wave delay timer
    pub wave_delay: f32,
    /// Is currently in wave delay?
    pub in_delay: bool,
    /// Boss fight active (don't spawn waves)
    pub boss_active: bool,
    /// Stage complete, waiting for next
    pub stage_complete: bool,
    /// Endless mode active (infinite waves)
    pub endless_mode: bool,
    /// Mini-boss spawning (for endless mode)
    pub mini_boss_active: bool,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            wave: 0,
            waves_per_stage: 5, // 5 waves then boss
            current_stage: 1,
            enemies_remaining: 0,
            spawn_timer: 0.0,
            spawn_interval: 0.8,
            wave_delay: 0.0,
            in_delay: true,
            boss_active: false,
            stage_complete: false,
            endless_mode: false,
            mini_boss_active: false,
        }
    }
}

/// Wave definition
#[derive(Debug, Clone)]
pub struct WaveDefinition {
    pub enemy_count: u32,
    pub enemy_types: Vec<u32>,
    pub behaviors: Vec<EnemyBehavior>,
    pub spawn_pattern: SpawnPattern,
}

fn reset_wave_manager(
    mut manager: ResMut<WaveManager>,
    mut endless: ResMut<crate::core::EndlessMode>,
    mut dialogue_system: ResMut<DialogueSystem>,
    mut dialogue_events: EventWriter<DialogueEvent>,
) {
    let is_endless = endless.active;

    *manager = WaveManager {
        wave: 0,
        waves_per_stage: 5,
        current_stage: 1,
        in_delay: true,
        wave_delay: if is_endless { 2.0 } else { 3.0 },
        boss_active: false,
        stage_complete: false,
        endless_mode: is_endless,
        mini_boss_active: false,
        ..default()
    };

    // Reset dialogue system and trigger briefing
    dialogue_system.reset();

    if is_endless {
        // Start endless mode tracking
        endless.start();
        info!("ENDLESS MODE - Survive as long as you can!");
    } else {
        dialogue_events.send(DialogueEvent::stage_briefing(1));
        info!("Stage 1 - The Call begins!");
    }
}

/// Main wave spawning logic
fn wave_spawning(
    mut commands: Commands,
    time: Res<Time>,
    mut manager: ResMut<WaveManager>,
    mut endless: ResMut<crate::core::EndlessMode>,
    mut next_state: ResMut<NextState<GameState>>,
    _stage: Res<CurrentStage>,
    session: Res<crate::core::GameSession>,
    enemy_query: Query<Entity, With<crate::entities::Enemy>>,
    boss_query: Query<Entity, With<crate::entities::Boss>>,
    carrier_query: Query<&Transform, With<EnemyCarrier>>,
    mut wave_events: EventWriter<SpawnWaveEvent>,
    mut boss_spawn_events: EventWriter<super::boss::BossEntitySpawned>,
    mut boss_defeated_events: EventReader<super::boss::BossEntityDefeated>,
    mut dialogue_events: EventWriter<DialogueEvent>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    model_cache: Res<ShipModelCache>,
) {
    // Get carrier position for spawning enemies
    let carrier_pos = carrier_query
        .get_single()
        .map(|t| Vec2::new(t.translation.x, t.translation.y))
        .unwrap_or(Vec2::new(0.0, SCREEN_HEIGHT / 2.0 - 100.0));
    let dt = time.delta_secs();

    // Update endless mode timer
    if manager.endless_mode && endless.active {
        endless.time_survived += dt;
    }

    // Handle boss defeated - progress to next stage (or continue endless)
    for event in boss_defeated_events.read() {
        manager.boss_active = false;

        // ENDLESS MODE: Mini-boss defeated, continue waves
        if manager.endless_mode {
            manager.mini_boss_active = false;
            endless.mini_bosses_defeated += 1;
            manager.wave_delay = 2.0;
            manager.in_delay = true;
            info!(
                "ENDLESS: Mini-boss {} defeated! {} total",
                event.boss_name, endless.mini_bosses_defeated
            );
            continue;
        }

        // CAMPAIGN MODE: Stage complete
        manager.stage_complete = true;
        manager.wave_delay = 4.0; // Pause before next stage
        manager.in_delay = true;

        // Check for act completion
        let completed_stage = manager.current_stage;
        if completed_stage == 4 {
            dialogue_events.send(DialogueEvent::act_complete(1));
        } else if completed_stage == 9 {
            dialogue_events.send(DialogueEvent::act_complete(2));
        } else if completed_stage == 13 {
            dialogue_events.send(DialogueEvent::act_complete(3));
        }

        info!(
            "Stage {} complete! {} defeated!",
            manager.current_stage, event.boss_name
        );
    }

    // If boss is active, don't spawn waves
    if manager.boss_active || !boss_query.is_empty() {
        return;
    }

    // Stage complete - wait then advance
    if manager.stage_complete {
        manager.wave_delay -= dt;
        if manager.wave_delay <= 0.0 {
            manager.current_stage += 1;
            if manager.current_stage > 13 {
                info!("CAMPAIGN COMPLETE! The Elder Fleet has liberated the Minmatar people!");
                next_state.set(GameState::Victory);
                return;
            }
            manager.wave = 0;
            manager.stage_complete = false;
            manager.wave_delay = 3.0;

            // Trigger stage briefing
            dialogue_events.send(DialogueEvent::stage_briefing(manager.current_stage));

            info!("Stage {} begins!", manager.current_stage);
        }
        return;
    }

    // Handle wave delay
    if manager.in_delay {
        manager.wave_delay -= dt;
        if manager.wave_delay <= 0.0 {
            manager.in_delay = false;
            manager.wave += 1;

            // ENDLESS MODE: Infinite waves
            if manager.endless_mode {
                endless.next_wave();

                // Check for mini-boss every 10 waves
                if endless.is_mini_boss_wave() {
                    manager.mini_boss_active = true;
                    // Spawn a mini-boss (use stage-based boss with scaled stats)
                    let mini_boss_stage = ((endless.wave / 10) % 13).max(1);
                    boss_spawn_events.send(super::boss::BossEntitySpawned {
                        stage: mini_boss_stage,
                    });
                    info!("ENDLESS Wave {} - MINI-BOSS incoming!", endless.wave);
                    return;
                }

                // Setup endless wave with escalating difficulty
                let enemy_count = endless.wave_enemy_count();
                manager.enemies_remaining = enemy_count;
                manager.spawn_interval = (0.6 - endless.wave as f32 * 0.01).max(0.2);

                wave_events.send(SpawnWaveEvent {
                    wave_number: endless.wave,
                    enemy_count,
                    enemy_types: vec!["endless".to_string()],
                });

                info!(
                    "ENDLESS Wave {}: {} enemies ({}x escalation)",
                    endless.wave, enemy_count, endless.escalation
                );
                return;
            }

            // CAMPAIGN MODE: Check if time for boss
            if manager.wave > manager.waves_per_stage {
                manager.boss_active = true;
                boss_spawn_events.send(super::boss::BossEntitySpawned {
                    stage: manager.current_stage,
                });
                info!("WARNING: Boss incoming!");
                return;
            }

            // Setup new wave — arcade density (inspired by classic shmups).
            // Raise enemy_count 1.7× and halve spawn interval so the screen
            // stays busy and the pacing feels frantic instead of trickled.
            let wave_def = get_wave_definition(manager.current_stage, manager.wave);
            manager.enemies_remaining =
                ((wave_def.enemy_count as f32 * 1.7).ceil() as u32).max(wave_def.enemy_count);
            manager.spawn_interval = 0.22 + 0.18 / (manager.wave as f32).sqrt();

            wave_events.send(SpawnWaveEvent {
                wave_number: manager.wave,
                enemy_count: wave_def.enemy_count,
                enemy_types: wave_def
                    .enemy_types
                    .iter()
                    .map(|id| format!("{}", id))
                    .collect(),
            });

            // Wave incoming callout on significant waves (every 5th or last before boss)
            if manager.wave % 5 == 0 || manager.wave == manager.waves_per_stage {
                dialogue_events.send(DialogueEvent::combat_callout(
                    super::CombatCalloutType::WaveIncoming,
                ));
            }

            info!(
                "Stage {} Wave {}/{}: {} enemies",
                manager.current_stage, manager.wave, manager.waves_per_stage, wave_def.enemy_count
            );
        }
        return;
    }

    // Spawn enemies
    if manager.enemies_remaining > 0 {
        manager.spawn_timer -= dt;
        if manager.spawn_timer <= 0.0 {
            manager.spawn_timer = manager.spawn_interval;

            // Get wave definition for behaviors and patterns
            let wave_def = get_wave_definition(manager.current_stage, manager.wave);

            // Get random enemy from enemy faction using GameSession
            let enemy_def = session.random_enemy();
            let type_id = enemy_def.type_id;

            // Pick behavior based on stage progression
            let behavior_idx = fastrand::usize(..wave_def.behaviors.len());
            let behavior = wave_def.behaviors[behavior_idx];

            // Spawn position based on pattern - enemies launch from carrier
            let pos = match wave_def.spawn_pattern {
                SpawnPattern::Single | SpawnPattern::Random => {
                    // Spawn near carrier with random spread
                    let x = carrier_pos.x + fastrand::f32() * 200.0 - 100.0;
                    Vec2::new(x, carrier_pos.y - 50.0)
                }
                SpawnPattern::Line => {
                    // Line formation emanating from carrier
                    let spacing = 300.0 / (wave_def.enemy_count as f32 + 1.0);
                    let idx = wave_def.enemy_count - manager.enemies_remaining;
                    let x = carrier_pos.x + spacing * (idx as f32 + 1.0) - 150.0;
                    Vec2::new(x, carrier_pos.y - 40.0)
                }
                SpawnPattern::VFormation => {
                    // V formation launching from carrier bay
                    let idx = wave_def.enemy_count - manager.enemies_remaining;
                    let center_idx = wave_def.enemy_count / 2;
                    let offset = (idx as i32 - center_idx as i32) as f32;
                    let x = carrier_pos.x + offset * 50.0;
                    let y = carrier_pos.y - 30.0 - offset.abs() * 25.0;
                    Vec2::new(x, y)
                }
                SpawnPattern::Circle => {
                    // Circle around carrier
                    let angle = (manager.enemies_remaining as f32) / (wave_def.enemy_count as f32)
                        * std::f32::consts::TAU;
                    let x = carrier_pos.x + angle.cos() * 150.0;
                    let y = carrier_pos.y + angle.sin() * 80.0 - 20.0;
                    Vec2::new(x, y)
                }
                SpawnPattern::Swarm => {
                    // Swarm bursting from carrier bay
                    let x = carrier_pos.x + fastrand::f32() * 300.0 - 150.0;
                    let y = carrier_pos.y - 20.0 - fastrand::f32() * 60.0;
                    Vec2::new(x, y)
                }
            };

            let sprite = sprite_cache.get(type_id);

            // Use specialized spawn functions for special enemy types
            let entity = match behavior {
                EnemyBehavior::Kamikaze => spawn_variant(
                    &mut commands,
                    EnemyVariant::Kamikaze,
                    pos,
                    sprite,
                    Some(&model_cache),
                ),
                EnemyBehavior::Weaver => spawn_variant(
                    &mut commands,
                    EnemyVariant::Weaver,
                    pos,
                    sprite,
                    Some(&model_cache),
                ),
                EnemyBehavior::Sniper => spawn_variant(
                    &mut commands,
                    EnemyVariant::Sniper,
                    pos,
                    sprite,
                    Some(&model_cache),
                ),
                EnemyBehavior::Spawner => spawn_variant(
                    &mut commands,
                    EnemyVariant::Spawner,
                    pos,
                    sprite,
                    Some(&model_cache),
                ),
                EnemyBehavior::Tank => spawn_variant(
                    &mut commands,
                    EnemyVariant::Tank,
                    pos,
                    sprite,
                    Some(&model_cache),
                ),
                _ => spawn_enemy(
                    &mut commands,
                    type_id,
                    pos,
                    behavior,
                    sprite,
                    Some(&model_cache),
                ),
            };

            // Formation-pattern waves cycle back across the battlefield so
            // patrols keep pressure on the player instead of vanishing once.
            if matches!(
                wave_def.spawn_pattern,
                SpawnPattern::Line | SpawnPattern::VFormation | SpawnPattern::Circle
            ) {
                commands
                    .entity(entity)
                    .insert(crate::entities::enemy::CycleOnExit);
            }

            // Endless difficulty scaling — boost HP + damage by current
            // escalation so later waves actually hurt, not just arrive faster.
            if endless.active && endless.escalation > 1.0 {
                commands
                    .entity(entity)
                    .insert(crate::entities::enemy::EndlessScale(endless.escalation));
            }

            manager.enemies_remaining -= 1;
        }
    }

    // Check if wave complete
    if manager.enemies_remaining == 0 && enemy_query.is_empty() && !manager.in_delay {
        manager.in_delay = true;
        manager.wave_delay = WAVE_DELAY;
        info!("Wave {} complete!", manager.wave);
    }
}

/// Handle manual spawn events
fn handle_spawn_events(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnEnemyEvent>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    model_cache: Res<ShipModelCache>,
) {
    for event in spawn_events.read() {
        let type_id: u32 = event.enemy_type.parse().unwrap_or(597);
        let behavior = match event.spawn_pattern {
            SpawnPattern::Single => EnemyBehavior::Linear,
            SpawnPattern::Line => EnemyBehavior::Linear,
            SpawnPattern::VFormation => EnemyBehavior::Zigzag,
            SpawnPattern::Circle => EnemyBehavior::Orbital,
            SpawnPattern::Random => EnemyBehavior::Homing,
            SpawnPattern::Swarm => EnemyBehavior::Kamikaze,
        };

        let sprite = sprite_cache.get(type_id);
        spawn_enemy(
            &mut commands,
            type_id,
            event.position,
            behavior,
            sprite,
            Some(&model_cache),
        );
    }
}

// ============================================================================
// Data-driven wave configuration (loaded from config/waves_elder_fleet.json)
// ============================================================================

/// JSON schema for wave config file
#[derive(Deserialize)]
struct WaveConfigFile {
    stages: HashMap<String, StageConfig>,
}

#[derive(Deserialize, Clone)]
struct StageConfig {
    enemy_types: Vec<u32>,
    behaviors: Vec<String>,
}

/// Parsed wave config — loaded once from embedded JSON
struct WaveConfig {
    stages: HashMap<u32, StageConfig>,
}

fn load_wave_config() -> WaveConfig {
    let json = include_str!("../../config/waves_elder_fleet.json");
    match serde_json::from_str::<WaveConfigFile>(json) {
        Ok(file) => {
            let stages = file
                .stages
                .into_iter()
                .filter_map(|(k, v)| k.parse::<u32>().ok().map(|num| (num, v)))
                .collect();
            WaveConfig { stages }
        }
        Err(_) => WaveConfig {
            stages: HashMap::new(),
        },
    }
}

/// Thread-local cached wave config (loaded once on first use)
fn wave_config() -> &'static WaveConfig {
    use std::sync::OnceLock;
    static CONFIG: OnceLock<WaveConfig> = OnceLock::new();
    CONFIG.get_or_init(load_wave_config)
}

fn parse_behavior(name: &str) -> EnemyBehavior {
    match name {
        "Linear" => EnemyBehavior::Linear,
        "Zigzag" => EnemyBehavior::Zigzag,
        "Homing" => EnemyBehavior::Homing,
        "Orbital" => EnemyBehavior::Orbital,
        "Sniper" => EnemyBehavior::Sniper,
        "Kamikaze" => EnemyBehavior::Kamikaze,
        "Weaver" => EnemyBehavior::Weaver,
        "Spawner" => EnemyBehavior::Spawner,
        "Tank" => EnemyBehavior::Tank,
        "Disintegrator" => EnemyBehavior::Disintegrator,
        _ => EnemyBehavior::Linear,
    }
}

/// Get wave definition based on stage and wave number
fn get_wave_definition(stage: u32, wave: u32) -> WaveDefinition {
    let config = wave_config();
    // Base enemy count scales with stage and wave
    let base_count = 3 + wave + (stage / 2);

    // Load from config, fall back to stage 1 defaults
    let (enemy_types, behaviors) = if let Some(stage_cfg) = config.stages.get(&stage) {
        let types = stage_cfg.enemy_types.clone();
        let behaviors: Vec<EnemyBehavior> = stage_cfg
            .behaviors
            .iter()
            .map(|b| parse_behavior(b))
            .collect();
        (types, behaviors)
    } else {
        // Fallback for unconfigured stages
        (vec![597], vec![EnemyBehavior::Linear])
    };

    // Spawn patterns cycle with wave
    let spawn_pattern = match wave % 5 {
        1 => SpawnPattern::Single,
        2 => SpawnPattern::Line,
        3 => SpawnPattern::VFormation,
        4 => SpawnPattern::Random,
        0 => SpawnPattern::Swarm,
        _ => SpawnPattern::Single,
    };

    WaveDefinition {
        enemy_count: base_count.min(12 + stage / 2),
        enemy_types,
        behaviors,
        spawn_pattern,
    }
}
