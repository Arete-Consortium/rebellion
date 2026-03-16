//! Enemy Entities
//!
//! All enemy ship types, AI behaviors, and wave spawning.

#![allow(dead_code)]

use crate::assets::{ShipModelCache, ShipModelRotation};
use crate::core::*;
use crate::systems::EngineTrail;
use bevy::prelude::*;

use super::projectile::{PlayerProjectile, ProjectilePhysics};

/// Marker component for enemy entities
#[derive(Component, Debug)]
pub struct Enemy;

/// Enemy AI behavior type
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyBehavior {
    /// Moves straight down
    Linear,
    /// Weaves side to side
    Zigzag,
    /// Moves toward player
    Homing,
    /// Circles around a point
    Orbital,
    /// Stays at distance, strafes horizontally, fires long-range
    Sniper,
    /// Rushes toward player at high speed (suicide)
    Kamikaze,
    /// Fast sine-wave pattern, harassing
    Weaver,
    /// Slow, spawns fighter escorts
    Spawner,
    /// Heavy armor, slow advance, absorbs damage
    Tank,
    /// Triglavian disintegrator: tracks player, fires continuous beam with ramping damage
    Disintegrator,
}

impl EnemyBehavior {
    /// How strongly this enemy reacts to incoming projectiles (0.0 = ignores, 1.0 = maximum dodge)
    pub fn dodge_sensitivity(&self) -> f32 {
        match self {
            EnemyBehavior::Linear => 0.3,
            EnemyBehavior::Zigzag => 0.5,
            EnemyBehavior::Homing => 0.3,
            EnemyBehavior::Orbital => 0.5,
            EnemyBehavior::Sniper => 0.8,        // Self-preservation
            EnemyBehavior::Kamikaze => 0.0,      // Suicide, doesn't dodge
            EnemyBehavior::Weaver => 0.7,        // Nimble harasser
            EnemyBehavior::Spawner => 0.2,       // Heavy, slow to react
            EnemyBehavior::Tank => 0.1,          // Absorbs damage
            EnemyBehavior::Disintegrator => 0.5, // Moderate evasion
        }
    }

    /// How accurately this enemy leads its shots (0.0 = no lead, 1.0 = perfect prediction)
    pub fn aim_accuracy(&self) -> f32 {
        match self {
            EnemyBehavior::Linear => 0.2,
            EnemyBehavior::Zigzag => 0.4,
            EnemyBehavior::Homing => 0.5,
            EnemyBehavior::Orbital => 0.4,
            EnemyBehavior::Sniper => 0.9,   // Precision platform
            EnemyBehavior::Kamikaze => 0.0, // Doesn't shoot
            EnemyBehavior::Weaver => 0.3,
            EnemyBehavior::Spawner => 0.2,
            EnemyBehavior::Tank => 0.5,
            EnemyBehavior::Disintegrator => 0.6, // Beam weapon, moderate tracking
        }
    }
}

/// Enemy stats
#[derive(Component, Debug, Clone)]
pub struct EnemyStats {
    /// Ship type ID
    pub type_id: u32,
    /// Display name
    pub name: String,
    /// Current HP
    pub health: f32,
    /// Maximum HP
    pub max_health: f32,
    /// Movement speed
    pub speed: f32,
    /// Score value when destroyed
    pub score_value: u64,
    /// Is this a boss?
    pub is_boss: bool,
    /// Number of souls liberated when destroyed
    pub liberation_value: u32,
}

impl Default for EnemyStats {
    fn default() -> Self {
        Self {
            type_id: 597, // Punisher
            name: "Punisher".into(),
            health: 30.0,
            max_health: 30.0,
            speed: ENEMY_BASE_SPEED,
            score_value: POINTS_PER_KILL,
            is_boss: false,
            liberation_value: 1, // Each enemy carries 1 enslaved soul
        }
    }
}

/// Enemy weapon
#[derive(Component, Debug, Clone)]
pub struct EnemyWeapon {
    /// Weapon type (determines projectile visuals and damage type)
    pub weapon_type: WeaponType,
    /// Fire rate
    pub fire_rate: f32,
    /// Cooldown timer
    pub cooldown: f32,
    /// Bullet speed
    pub bullet_speed: f32,
    /// Damage per hit
    pub damage: f32,
    /// Firing pattern
    pub pattern: FiringPattern,
}

/// Enemy firing patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiringPattern {
    /// Single shot at player
    Single,
    /// 3-shot spread
    Spread3,
    /// 5-shot spread
    Spread5,
    /// Circular burst
    Circle,
    /// Aimed stream
    Stream,
}

impl Default for EnemyWeapon {
    fn default() -> Self {
        Self {
            weapon_type: WeaponType::Laser, // Default Amarr
            fire_rate: 1.0,
            cooldown: 1.0,
            bullet_speed: ENEMY_BULLET_SPEED,
            damage: 10.0,
            pattern: FiringPattern::Single,
        }
    }
}

/// AI state for behavior logic
#[derive(Component, Debug, Clone)]
pub struct EnemyAI {
    /// Current behavior
    pub behavior: EnemyBehavior,
    /// Timer for behavior patterns
    pub timer: f32,
    /// Phase for oscillating patterns
    pub phase: f32,
    /// Target position (for some behaviors)
    pub target: Vec2,
    /// Whether currently active (on screen)
    pub active: bool,
    /// Accumulated dodge/separation impulse from spatial awareness (reset each frame)
    pub dodge_impulse: Vec2,
}

impl Default for EnemyAI {
    fn default() -> Self {
        Self {
            behavior: EnemyBehavior::Linear,
            timer: 0.0,
            phase: 0.0,
            target: Vec2::ZERO,
            active: true,
            dodge_impulse: Vec2::ZERO,
        }
    }
}

/// Spawner component for enemies that deploy fighters
#[derive(Component, Debug)]
pub struct EnemySpawner {
    /// Time between spawns
    pub spawn_rate: f32,
    /// Spawn cooldown timer
    pub spawn_timer: f32,
    /// Type ID of spawned enemies
    pub spawn_type_id: u32,
    /// Max spawned at once
    pub max_spawned: u32,
    /// Currently spawned count
    pub spawned_count: u32,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            spawn_rate: 3.0,
            spawn_timer: 2.0,
            spawn_type_id: 589, // Executioner (small fighter)
            max_spawned: 4,
            spawned_count: 0,
        }
    }
}

/// Triglavian Disintegrator ramping damage component
/// Damage increases the longer the beam stays on target
#[derive(Component, Debug, Clone)]
pub struct DisintegratorRamp {
    /// Base damage per tick
    pub base_damage: f32,
    /// Maximum damage multiplier (1.0 = no ramp, 3.0 = 3x max)
    pub ramp_max: f32,
    /// Time to reach max ramp (seconds)
    pub ramp_time: f32,
    /// Time currently on target
    pub time_on_target: f32,
    /// Current damage multiplier (1.0 to ramp_max)
    pub current_mult: f32,
    /// Is beam currently active/firing
    pub beam_active: bool,
    /// Beam visual intensity (0.0 to 1.0)
    pub beam_intensity: f32,
}

impl Default for DisintegratorRamp {
    fn default() -> Self {
        Self {
            base_damage: 8.0,
            ramp_max: 2.0,
            ramp_time: 6.0,
            time_on_target: 0.0,
            current_mult: 1.0,
            beam_active: false,
            beam_intensity: 0.0,
        }
    }
}

impl DisintegratorRamp {
    /// Create a new disintegrator with specified parameters
    pub fn new(base_damage: f32, ramp_max: f32, ramp_time: f32) -> Self {
        Self {
            base_damage,
            ramp_max,
            ramp_time,
            ..Default::default()
        }
    }

    /// Update the ramp based on whether we're hitting the target
    pub fn update(&mut self, dt: f32, hitting_target: bool) {
        if hitting_target {
            self.time_on_target += dt;
            let ramp_progress = (self.time_on_target / self.ramp_time).min(1.0);
            self.current_mult = 1.0 + (self.ramp_max - 1.0) * ramp_progress;
            self.beam_active = true;
            self.beam_intensity = 0.3 + 0.7 * ramp_progress; // 30% to 100% intensity
        } else {
            // Reset ramp when not hitting
            self.time_on_target = 0.0;
            self.current_mult = 1.0;
            self.beam_active = false;
            self.beam_intensity = 0.0;
        }
    }

    /// Get current damage output
    pub fn current_damage(&self) -> f32 {
        self.base_damage * self.current_mult
    }

    /// Get ramp progress (0.0 to 1.0)
    pub fn ramp_progress(&self) -> f32 {
        (self.time_on_target / self.ramp_time).min(1.0)
    }
}

/// Bundle for spawning an enemy
#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub stats: EnemyStats,
    pub weapon: EnemyWeapon,
    pub ai: EnemyAI,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            enemy: Enemy,
            stats: EnemyStats::default(),
            weapon: EnemyWeapon::default(),
            ai: EnemyAI::default(),
            sprite: Sprite {
                color: COLOR_AMARR,
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 300.0, LAYER_ENEMIES),
        }
    }
}

// =============================================================================
// SPATIAL AWARENESS & PREDICTIVE AI
// =============================================================================

/// How far enemies detect incoming player projectiles
const DODGE_DETECTION_RADIUS: f32 = 100.0;
/// Base dodge force applied when evading projectiles
const DODGE_STRENGTH: f32 = 150.0;
/// Distance at which enemies repel from each other
const SEPARATION_RADIUS: f32 = 55.0;
/// Force applied to keep enemies apart
const SEPARATION_STRENGTH: f32 = 80.0;
/// Screen edge margin for soft boundary avoidance
const EDGE_AVOIDANCE_MARGIN: f32 = 40.0;
/// Force pushing enemies away from screen edges
const EDGE_PUSH_STRENGTH: f32 = 120.0;
/// Maximum total dodge impulse magnitude (prevents runaway forces)
const MAX_DODGE_IMPULSE: f32 = 300.0;
/// Radius within which enemies rally around leader units (Spawner/Tank)
const LEADER_RALLY_RADIUS: f32 = 150.0;
/// Cohesion force pulling escort enemies toward their leader
const LEADER_COHESION_STRENGTH: f32 = 40.0;

/// Tracks player position and velocity so enemies can predict movement
#[derive(Resource, Default)]
pub struct PlayerTracker {
    /// Current player position
    pub position: Vec2,
    /// Estimated player velocity (units/sec)
    pub velocity: Vec2,
    /// Previous frame position for velocity calculation
    prev_position: Vec2,
    /// Whether we've seen at least one frame
    initialized: bool,
}

/// Updates the player tracker with current position and derived velocity
fn update_player_tracker(
    time: Res<Time>,
    player_query: Query<&Transform, With<super::Player>>,
    mut tracker: ResMut<PlayerTracker>,
) {
    let dt = time.delta_secs();
    if let Ok(transform) = player_query.get_single() {
        let pos = transform.translation.truncate();
        if tracker.initialized && dt > 0.0 {
            tracker.velocity = (pos - tracker.prev_position) / dt;
        }
        tracker.prev_position = pos;
        tracker.position = pos;
        tracker.initialized = true;
    }
}

/// Computes spatial awareness for each enemy: projectile dodge, enemy separation,
/// edge avoidance, and coordinated leader-escort tactics.
/// Stores result in `EnemyAI.dodge_impulse` for the movement system to apply.
fn enemy_spatial_awareness(
    projectile_query: Query<(&Transform, &ProjectilePhysics), With<PlayerProjectile>>,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyAI), With<Enemy>>,
) {
    // Collect enemy positions and behaviors first (immutable pass)
    let enemy_data: Vec<(Entity, Vec2, EnemyBehavior)> = enemy_query
        .iter()
        .map(|(e, t, ai)| (e, t.translation.truncate(), ai.behavior))
        .collect();

    // Identify leader positions (Spawner and Tank enemies act as squad leaders)
    let leaders: Vec<Vec2> = enemy_data
        .iter()
        .filter(|(_, _, b)| matches!(b, EnemyBehavior::Spawner | EnemyBehavior::Tank))
        .map(|(_, pos, _)| *pos)
        .collect();

    // Collect projectile data
    let projectiles: Vec<(Vec2, Vec2)> = projectile_query
        .iter()
        .map(|(t, p)| (t.translation.truncate(), p.velocity))
        .collect();

    let half_w = SCREEN_WIDTH / 2.0;
    let half_h = SCREEN_HEIGHT / 2.0;

    for (entity, transform, mut ai) in enemy_query.iter_mut() {
        let pos = transform.translation.truncate();
        let sensitivity = ai.behavior.dodge_sensitivity();
        let mut impulse = Vec2::ZERO;

        if sensitivity > 0.0 {
            // 1. Projectile dodge — evade incoming player bullets
            for &(proj_pos, proj_vel) in &projectiles {
                let to_enemy = pos - proj_pos;
                let dist = to_enemy.length();

                if dist < DODGE_DETECTION_RADIUS && dist > 1.0 {
                    let proj_dir = proj_vel.normalize_or_zero();
                    let approach = proj_dir.dot(to_enemy.normalize_or_zero());

                    if approach > 0.2 {
                        let perpendicular = Vec2::new(-proj_dir.y, proj_dir.x);
                        let side = perpendicular.dot(to_enemy).signum();
                        let urgency = 1.0 - (dist / DODGE_DETECTION_RADIUS);
                        impulse += perpendicular * side * urgency * DODGE_STRENGTH * sensitivity;
                    }
                }
            }

            // 2. Separation — avoid stacking on top of other enemies
            for &(other_entity, other_pos, _) in &enemy_data {
                if other_entity == entity {
                    continue;
                }
                let diff = pos - other_pos;
                let dist = diff.length();
                if dist < SEPARATION_RADIUS && dist > 1.0 {
                    let push = diff.normalize_or_zero()
                        * (1.0 - dist / SEPARATION_RADIUS)
                        * SEPARATION_STRENGTH;
                    impulse += push;
                }
            }

            // 3. Screen edge avoidance
            if pos.x < -half_w + EDGE_AVOIDANCE_MARGIN {
                impulse.x += (1.0 - (pos.x + half_w) / EDGE_AVOIDANCE_MARGIN) * EDGE_PUSH_STRENGTH;
            }
            if pos.x > half_w - EDGE_AVOIDANCE_MARGIN {
                impulse.x -= (1.0 - (half_w - pos.x) / EDGE_AVOIDANCE_MARGIN) * EDGE_PUSH_STRENGTH;
            }
            if pos.y > half_h - EDGE_AVOIDANCE_MARGIN {
                impulse.y -= (1.0 - (half_h - pos.y) / EDGE_AVOIDANCE_MARGIN) * EDGE_PUSH_STRENGTH;
            }
        }

        // 4. Coordinated tactics — escort enemies rally near leaders
        // Non-leader enemies are gently pulled toward the nearest Spawner or Tank
        let is_leader = matches!(ai.behavior, EnemyBehavior::Spawner | EnemyBehavior::Tank);
        if !is_leader && !leaders.is_empty() {
            let mut nearest_leader: Option<Vec2> = None;
            let mut nearest_dist = LEADER_RALLY_RADIUS;
            for &leader_pos in &leaders {
                let dist = (leader_pos - pos).length();
                if dist < nearest_dist {
                    nearest_dist = dist;
                    nearest_leader = Some(leader_pos);
                }
            }
            if let Some(leader_pos) = nearest_leader {
                let to_leader = leader_pos - pos;
                let dist = to_leader.length();
                // Only pull if beyond comfortable escort distance (40 units)
                if dist > 40.0 {
                    let cohesion = to_leader.normalize_or_zero()
                        * (dist / LEADER_RALLY_RADIUS)
                        * LEADER_COHESION_STRENGTH;
                    impulse += cohesion;
                }
            }
        }

        ai.dodge_impulse = impulse.clamp_length_max(MAX_DODGE_IMPULSE);
    }
}

/// Enemy plugin
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerTracker>().add_systems(
            Update,
            (
                // Ordered pipeline: track player → compute awareness → move → shoot
                (
                    update_player_tracker,
                    enemy_spatial_awareness,
                    enemy_movement,
                    enemy_shooting,
                )
                    .chain(),
                // These can run in parallel
                update_enemy_ship_rotation,
                disintegrator_update,
                spawner_update,
                enemy_bounds_check,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Enemy movement based on AI behavior + spatial awareness dodge impulse
fn enemy_movement(
    time: Res<Time>,
    player_tracker: Res<PlayerTracker>,
    mut query: Query<
        (&mut Transform, &EnemyStats, &mut EnemyAI),
        (With<Enemy>, Without<super::Player>),
    >,
) {
    let dt = time.delta_secs();
    let player_pos = player_tracker.position;

    for (mut transform, stats, mut ai) in query.iter_mut() {
        ai.timer += dt;
        let pos = transform.translation.truncate();

        let velocity = match ai.behavior {
            EnemyBehavior::Linear => Vec2::new(0.0, -1.0) * stats.speed,
            EnemyBehavior::Zigzag => {
                let x = (ai.timer * 3.0 + ai.phase).sin() * stats.speed;
                Vec2::new(x, -stats.speed * 0.5)
            }
            EnemyBehavior::Homing => {
                let dir = (player_pos - pos).normalize_or_zero();
                dir * stats.speed
            }
            EnemyBehavior::Orbital => {
                let angle = ai.timer * 2.0 + ai.phase;
                let orbit_center = Vec2::new(0.0, 100.0);
                let target = orbit_center + Vec2::new(angle.cos(), angle.sin()) * 150.0;
                (target - pos).normalize_or_zero() * stats.speed
            }
            EnemyBehavior::Sniper => {
                // Stay at top, strafe
                let target_y = SCREEN_HEIGHT / 2.0 - 100.0;
                let y_diff = target_y - pos.y;
                let x = (ai.timer * 1.5 + ai.phase).sin() * stats.speed;
                Vec2::new(x, y_diff.signum() * stats.speed.min(y_diff.abs()))
            }
            EnemyBehavior::Kamikaze => {
                // Suicide rush toward player at 2x speed
                let dir = (player_pos - pos).normalize_or_zero();
                dir * stats.speed * 2.0
            }
            EnemyBehavior::Weaver => {
                // Fast sine-wave, wide amplitude, harassing movement
                let amplitude = 200.0;
                let frequency = 4.0;
                let x = (ai.timer * frequency + ai.phase).sin() * amplitude * dt * 2.0;
                Vec2::new(x, -stats.speed * 0.7)
            }
            EnemyBehavior::Spawner => {
                // Slow descent, stays in upper area
                let target_y = SCREEN_HEIGHT / 2.0 - 150.0;
                if pos.y > target_y {
                    Vec2::new(0.0, -stats.speed * 0.3)
                } else {
                    // Slow side-to-side drift once in position
                    let x = (ai.timer * 0.5).sin() * stats.speed * 0.3;
                    Vec2::new(x, 0.0)
                }
            }
            EnemyBehavior::Tank => {
                // Slow but relentless advance toward player
                let dir = (player_pos - pos).normalize_or_zero();
                // Mostly moves down, slight homing
                Vec2::new(dir.x * stats.speed * 0.3, -stats.speed * 0.4)
            }
            EnemyBehavior::Disintegrator => {
                // Triglavian: Maintains distance while tracking player
                // Optimal range: 150-250 units from player
                let to_player = player_pos - pos;
                let distance = to_player.length();
                let dir = to_player.normalize_or_zero();

                let optimal_range = 200.0;
                let approach_speed = if distance > optimal_range + 50.0 {
                    stats.speed * 0.8 // Close in
                } else if distance < optimal_range - 50.0 {
                    -stats.speed * 0.5 // Back off
                } else {
                    0.0 // At optimal range
                };

                // Strafe perpendicular to player direction
                let strafe = Vec2::new(-dir.y, dir.x) * (ai.timer * 2.0).sin() * stats.speed * 0.4;

                dir * approach_speed + strafe + Vec2::new(0.0, -stats.speed * 0.2)
            }
        };

        // Combine behavior velocity with spatial awareness (dodge + separation + edge avoidance)
        let total_velocity = velocity + ai.dodge_impulse;

        transform.translation.x += total_velocity.x * dt;
        transform.translation.y += total_velocity.y * dt;

        // Slight tilt based on horizontal movement (visual effect only)
        let tilt = (total_velocity.x / stats.speed.max(1.0)).clamp(-1.0, 1.0) * 0.2;
        transform.rotation = Quat::from_rotation_z(tilt);
    }
}

/// Enemy shooting system with predictive aiming
/// Enemies lead their shots based on player velocity — accuracy depends on behavior type
fn enemy_shooting(
    mut commands: Commands,
    time: Res<Time>,
    player_tracker: Res<PlayerTracker>,
    mut query: Query<(&Transform, &mut EnemyWeapon, &EnemyAI), With<Enemy>>,
) {
    let dt = time.delta_secs();
    let player_pos = player_tracker.position;
    let player_vel = player_tracker.velocity;

    for (transform, mut weapon, ai) in query.iter_mut() {
        if !ai.active {
            continue;
        }

        weapon.cooldown -= dt;
        if weapon.cooldown <= 0.0 {
            weapon.cooldown = 1.0 / weapon.fire_rate;

            let pos = transform.translation.truncate();

            // Predictive aiming: lead the shot based on player velocity
            let accuracy = ai.behavior.aim_accuracy();
            let distance = (player_pos - pos).length();
            let flight_time = distance / weapon.bullet_speed.max(1.0);
            let predicted_pos = player_pos + player_vel * flight_time * accuracy;

            let dir = (predicted_pos - pos).normalize_or_zero();

            // Spawn enemy projectile with correct weapon type
            super::projectile::spawn_enemy_projectile_typed(
                &mut commands,
                pos,
                dir,
                weapon.damage,
                weapon.bullet_speed,
                weapon.weapon_type,
            );
        }
    }
}

/// Triglavian disintegrator beam system
/// Handles continuous beam damage with ramping multiplier
fn disintegrator_update(
    time: Res<Time>,
    mut player_query: Query<
        (
            &Transform,
            &mut super::ShipStats,
            &super::PowerupEffects,
            &crate::systems::ManeuverState,
        ),
        With<super::Player>,
    >,
    mut enemy_query: Query<(&Transform, &mut DisintegratorRamp, &EnemyAI), With<Enemy>>,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let dt = time.delta_secs();

    let Ok((player_transform, mut player_stats, powerups, maneuver)) =
        player_query.get_single_mut()
    else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    // Check invulnerability
    let player_invulnerable = powerups.is_invulnerable() || maneuver.invincible;

    for (enemy_transform, mut disintegrator, ai) in enemy_query.iter_mut() {
        if !ai.active {
            disintegrator.update(dt, false);
            continue;
        }

        let enemy_pos = enemy_transform.translation.truncate();
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();

        // Check if player is within beam range (350 units max)
        let in_range = distance < 350.0;

        // Update ramping state
        disintegrator.update(dt, in_range);

        // Apply damage if beam is active
        if disintegrator.beam_active && !player_invulnerable {
            // Damage per second = base * mult, convert to per-frame damage
            let damage_per_frame = disintegrator.current_damage() * dt;

            // Apply damage directly to player
            let destroyed = player_stats.take_damage(damage_per_frame, DamageType::Thermal);

            // Send damage event for other systems to react
            damage_events.send(PlayerDamagedEvent {
                damage: damage_per_frame,
                damage_type: DamageType::Thermal,
                source_position: enemy_pos,
            });

            if destroyed {
                info!("Player destroyed by disintegrator beam!");
                next_state.set(GameState::GameOver);
            }
        }
    }
}

/// Remove enemies that go off screen
fn enemy_bounds_check(mut commands: Commands, query: Query<(Entity, &Transform), With<Enemy>>) {
    let margin = 100.0;
    for (entity, transform) in query.iter() {
        let pos = transform.translation;
        if pos.y < -SCREEN_HEIGHT / 2.0 - margin
            || pos.y > SCREEN_HEIGHT / 2.0 + margin
            || pos.x.abs() > SCREEN_WIDTH / 2.0 + margin
        {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Update 3D enemy rotation based on movement (banking/tilting)
fn update_enemy_ship_rotation(
    time: Res<Time>,
    mut query: Query<(&EnemyStats, &EnemyAI, &mut Transform, &ShipModelRotation), With<Enemy>>,
) {
    let dt = time.delta_secs();

    for (stats, ai, mut transform, model_rot) in query.iter_mut() {
        // Estimate velocity from AI behavior
        let velocity = match ai.behavior {
            EnemyBehavior::Linear => Vec2::new(0.0, -stats.speed),
            EnemyBehavior::Zigzag => {
                let x = (ai.timer * 3.0 + ai.phase).sin() * stats.speed;
                Vec2::new(x, -stats.speed * 0.5)
            }
            EnemyBehavior::Homing | EnemyBehavior::Kamikaze => {
                // These move toward player, estimate based on target
                let dir = (ai.target - transform.translation.truncate()).normalize_or_zero();
                dir * stats.speed
            }
            EnemyBehavior::Orbital => {
                let angle = ai.timer * 2.0 + ai.phase;
                Vec2::new(-angle.sin(), angle.cos()) * stats.speed * 0.5
            }
            EnemyBehavior::Sniper => {
                let x = (ai.timer * 1.5 + ai.phase).sin() * stats.speed;
                Vec2::new(x, 0.0)
            }
            EnemyBehavior::Weaver => {
                let x = (ai.timer * 4.0 + ai.phase).cos() * stats.speed;
                Vec2::new(x, -stats.speed * 0.7)
            }
            EnemyBehavior::Spawner => {
                let x = (ai.timer * 0.5).cos() * stats.speed * 0.3;
                Vec2::new(x, 0.0)
            }
            EnemyBehavior::Tank => Vec2::new(0.0, -stats.speed * 0.4),
            EnemyBehavior::Disintegrator => {
                // Triglavian ships strafe while tracking
                let strafe = (ai.timer * 2.0).sin() * stats.speed * 0.4;
                Vec2::new(strafe, -stats.speed * 0.2)
            }
        };

        let target_rotation = model_rot.calculate_rotation(velocity, stats.speed);
        transform.rotation = transform
            .rotation
            .slerp(target_rotation, (model_rot.smoothing * dt).min(1.0));
    }
}

/// Get faction color for enemy type
fn get_enemy_color(type_id: u32) -> Color {
    match type_id {
        // Amarr - Gold (frigates, destroyers, battlecruisers)
        597 | 589 | 591 | 16236 | 24690 => COLOR_AMARR,
        // Caldari - Steel Blue (frigates, destroyers, battlecruisers)
        603 | 602 | 583 | 16238 | 24688 => COLOR_CALDARI,
        // Gallente - Green (frigates, destroyers, battlecruisers)
        593 | 594 | 608 | 16242 | 24700 => COLOR_GALLENTE,
        // Minmatar - Rust (frigates)
        587 | 585 | 598 => COLOR_MINMATAR,
        // Triglavian - Crimson (Damavik, Vedmak, Drekavac)
        47269..=47273 => COLOR_TRIGLAVIAN,
        _ => Color::srgb(0.5, 0.5, 0.5),
    }
}

/// Get engine trail for faction based on type_id
fn get_faction_engine_trail(type_id: u32) -> EngineTrail {
    match type_id {
        // Amarr - golden engines (frigates, destroyers, battlecruisers)
        597 | 589 | 591 | 16236 | 24690 | 624 | 2006 | 11373 => EngineTrail::amarr(),
        // Caldari - blue engines (frigates, destroyers, battlecruisers)
        603 | 602 | 583 | 16238 | 24688 | 11381 | 11387 | 35683 => EngineTrail::caldari(),
        // Gallente - green engines (frigates, destroyers, battlecruisers)
        593 | 594 | 608 | 16242 | 24700 | 11371 | 35685 => EngineTrail::gallente(),
        // Minmatar - rust engines
        587 | 585 | 598 => EngineTrail::minmatar(),
        _ => EngineTrail::amarr(), // Default to Amarr (enemies)
    }
}

/// Get weapon type for faction based on type_id
fn get_faction_weapon(type_id: u32) -> WeaponType {
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
        // Triglavian - Disintegrators (ramping damage)
        47269 | 49710 | 47271 | 49711 | 47273 | 47466 | 56756 => WeaponType::Disintegrator,
        // EDENCOM - Vorton projectors (chain lightning)
        56757 | 56759 | 56760 => WeaponType::Vorton,
        _ => WeaponType::Laser,
    }
}

/// Get rotation correction for ships with non-standard orientations from CCP renders
/// Returns additional rotation in radians to apply on top of base rotation
pub fn get_ship_rotation_correction(type_id: u32) -> f32 {
    use std::f32::consts::FRAC_PI_2;
    match type_id {
        // === CALDARI === (bundled sprites face up, need 180° base only)
        // 602 => 0.0,        // Kestrel - faces up, no extra correction
        603 => -FRAC_PI_2,  // Merlin - faces left
        583 => -FRAC_PI_2,  // Condor - faces left
        11381 => FRAC_PI_2, // Hawk - assault frigate
        11387 => FRAC_PI_2, // Harpy - assault frigate
        35683 => FRAC_PI_2, // Jackdaw - tactical destroyer

        // === GALLENTE === (most render sideways)
        593 => FRAC_PI_2,   // Tristan - faces right
        594 => FRAC_PI_2,   // Incursus - faces right
        608 => FRAC_PI_2,   // Atron - faces right
        11373 => FRAC_PI_2, // Enyo - assault frigate
        11377 => FRAC_PI_2, // Ishkur - assault frigate
        35685 => FRAC_PI_2, // Hecate - tactical destroyer

        // === DESTROYERS ===
        16236 => FRAC_PI_2,  // Coercer (Amarr)
        16238 => FRAC_PI_2,  // Cormorant (Caldari)
        16242 => -FRAC_PI_2, // Catalyst (Gallente) - faces left

        // === BATTLECRUISERS ===
        24688 => FRAC_PI_2, // Drake (Caldari)
        24690 => FRAC_PI_2, // Harbinger (Amarr)
        24700 => FRAC_PI_2, // Myrmidon (Gallente)

        // === AMARR ===
        597 => std::f32::consts::PI, // Punisher - faces down, flip 180°
        591 => FRAC_PI_2,            // Tormentor - faces right
        // 589 (Executioner) - faces up

        // === MINMATAR ===
        587 => std::f32::consts::PI, // Rifter - faces down, flip 180°
        585 => std::f32::consts::PI, // Slasher - faces down, flip 180°
        // 598 (Breacher) - faces up, no rotation needed

        // === CARRIERS ===
        24483 => std::f32::consts::PI, // Nidhoggur (Minmatar) - needs 180° flip
        23915 => std::f32::consts::PI, // Chimera (Caldari) - needs 180° flip
        // 23757 (Archon), 23911 (Thanatos) - face correctly

        // Ships that already face up correctly
        _ => 0.0,
    }
}

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

    // Get rotation: 180° base (face down) + per-ship correction
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
fn spawner_update(
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
