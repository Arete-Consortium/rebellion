//! Enemy Components, Types, and Bundles
//!
//! All shared types for enemy entities: markers, stats, weapons, AI state, and bundles.

#![allow(dead_code)]

use crate::core::*;
use bevy::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    // EnemyBehavior tests
    #[test]
    fn behavior_dodge_sensitivity_ranges() {
        let behaviors = [
            EnemyBehavior::Linear,
            EnemyBehavior::Zigzag,
            EnemyBehavior::Homing,
            EnemyBehavior::Orbital,
            EnemyBehavior::Sniper,
            EnemyBehavior::Kamikaze,
            EnemyBehavior::Weaver,
            EnemyBehavior::Spawner,
            EnemyBehavior::Tank,
            EnemyBehavior::Disintegrator,
        ];
        for b in &behaviors {
            let ds = b.dodge_sensitivity();
            assert!(
                (0.0..=1.0).contains(&ds),
                "{:?} dodge_sensitivity {} out of range",
                b,
                ds
            );
        }
    }

    #[test]
    fn behavior_kamikaze_never_dodges() {
        assert_eq!(EnemyBehavior::Kamikaze.dodge_sensitivity(), 0.0);
    }

    #[test]
    fn behavior_kamikaze_never_aims() {
        assert_eq!(EnemyBehavior::Kamikaze.aim_accuracy(), 0.0);
    }

    #[test]
    fn behavior_sniper_has_high_accuracy() {
        assert!(EnemyBehavior::Sniper.aim_accuracy() > 0.8);
    }

    #[test]
    fn behavior_aim_accuracy_ranges() {
        let behaviors = [
            EnemyBehavior::Linear,
            EnemyBehavior::Zigzag,
            EnemyBehavior::Homing,
            EnemyBehavior::Orbital,
            EnemyBehavior::Sniper,
            EnemyBehavior::Kamikaze,
            EnemyBehavior::Weaver,
            EnemyBehavior::Spawner,
            EnemyBehavior::Tank,
            EnemyBehavior::Disintegrator,
        ];
        for b in &behaviors {
            let aa = b.aim_accuracy();
            assert!(
                (0.0..=1.0).contains(&aa),
                "{:?} aim_accuracy {} out of range",
                b,
                aa
            );
        }
    }

    // EnemyStats defaults
    #[test]
    fn enemy_stats_default_not_boss() {
        let stats = EnemyStats::default();
        assert!(!stats.is_boss);
    }

    #[test]
    fn enemy_stats_default_health_equals_max() {
        let stats = EnemyStats::default();
        assert_eq!(stats.health, stats.max_health);
    }

    #[test]
    fn enemy_stats_default_positive_values() {
        let stats = EnemyStats::default();
        assert!(stats.health > 0.0);
        assert!(stats.speed > 0.0);
        assert!(stats.score_value > 0);
    }

    // EnemyAI defaults
    #[test]
    fn enemy_ai_default_linear() {
        let ai = EnemyAI::default();
        assert_eq!(ai.behavior, EnemyBehavior::Linear);
        assert!(ai.active);
    }

    // DisintegratorRamp tests
    #[test]
    fn disintegrator_new_sets_params() {
        let ramp = DisintegratorRamp::new(10.0, 3.0, 5.0);
        assert_eq!(ramp.base_damage, 10.0);
        assert_eq!(ramp.ramp_max, 3.0);
        assert_eq!(ramp.ramp_time, 5.0);
    }

    #[test]
    fn disintegrator_starts_at_base_damage() {
        let ramp = DisintegratorRamp::new(10.0, 3.0, 5.0);
        assert_eq!(ramp.current_damage(), 10.0);
        assert_eq!(ramp.current_mult, 1.0);
    }

    #[test]
    fn disintegrator_ramp_increases_on_target() {
        let mut ramp = DisintegratorRamp::new(10.0, 3.0, 5.0);
        ramp.update(2.5, true); // Half ramp time
        assert!(ramp.current_mult > 1.0);
        assert!(ramp.current_damage() > 10.0);
        assert!(ramp.beam_active);
    }

    #[test]
    fn disintegrator_ramp_maxes_out() {
        let mut ramp = DisintegratorRamp::new(10.0, 3.0, 5.0);
        ramp.update(10.0, true); // Way past ramp time
        assert_eq!(ramp.current_mult, 3.0);
        assert_eq!(ramp.current_damage(), 30.0);
    }

    #[test]
    fn disintegrator_ramp_resets_off_target() {
        let mut ramp = DisintegratorRamp::new(10.0, 3.0, 5.0);
        ramp.update(5.0, true);
        assert!(ramp.current_mult > 1.0);
        ramp.update(0.1, false);
        assert_eq!(ramp.current_mult, 1.0);
        assert!(!ramp.beam_active);
    }

    #[test]
    fn disintegrator_ramp_progress_clamped() {
        let mut ramp = DisintegratorRamp::new(10.0, 2.0, 6.0);
        ramp.update(100.0, true);
        assert_eq!(ramp.ramp_progress(), 1.0);
    }

    #[test]
    fn disintegrator_beam_intensity_scales() {
        let mut ramp = DisintegratorRamp::new(10.0, 2.0, 6.0);
        ramp.update(0.0, true);
        let start_intensity = ramp.beam_intensity;
        ramp.update(6.0, true);
        assert!(ramp.beam_intensity > start_intensity);
    }

    // EnemyWeapon defaults
    #[test]
    fn enemy_weapon_default_laser() {
        let weapon = EnemyWeapon::default();
        assert_eq!(weapon.weapon_type, WeaponType::Laser);
        assert_eq!(weapon.pattern, FiringPattern::Single);
    }

    // EnemySpawner defaults
    #[test]
    fn enemy_spawner_default_values() {
        let spawner = EnemySpawner::default();
        assert!(spawner.spawn_rate > 0.0);
        assert!(spawner.max_spawned > 0);
        assert_eq!(spawner.spawned_count, 0);
    }
}
