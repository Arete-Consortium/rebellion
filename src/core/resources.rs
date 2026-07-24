//! Shared Game Resources
//!
//! Global state that persists across systems.

#![allow(dead_code)]

use bevy::prelude::*;

/// Player score and combo system
#[derive(Debug, Clone, Resource)]
pub struct ScoreSystem {
    /// Current score
    pub score: u64,
    /// Current multiplier (1.0 - 99.9)
    pub multiplier: f32,
    /// Kill chain count
    pub chain: u32,
    /// Time remaining to maintain chain
    pub chain_timer: f32,
    /// Maximum chain time
    pub max_chain_time: f32,
    /// Style points earned
    pub style_points: u32,
    /// No damage bonus active
    pub no_damage_bonus: bool,
    /// Souls liberated count (Elder Fleet campaign)
    pub souls_liberated: u32,
}

impl Default for ScoreSystem {
    fn default() -> Self {
        Self {
            score: 0,
            multiplier: 1.0,
            chain: 0,
            chain_timer: 0.0,
            max_chain_time: crate::core::constants::CHAIN_TIMEOUT,
            style_points: 0,
            no_damage_bonus: true,
            souls_liberated: 0,
        }
    }
}

impl ScoreSystem {
    /// Add points with current multiplier
    pub fn add_score(&mut self, base_points: u64) {
        let final_points = (base_points as f32 * self.multiplier) as u64;
        self.score += final_points;
    }

    /// Register a kill and extend chain
    pub fn on_kill(&mut self, base_points: u64) {
        self.chain += 1;
        self.chain_timer = self.max_chain_time;
        self.multiplier = (1.0 + self.chain as f32 * 0.1).min(99.9);
        self.add_score(base_points);
    }

    /// Update chain timer (call each frame)
    pub fn update(&mut self, dt: f32) {
        if self.chain > 0 {
            self.chain_timer -= dt;
            if self.chain_timer <= 0.0 {
                self.chain = 0;
                self.multiplier = 1.0;
            }
        }
    }

    /// Get style grade based on average multiplier
    pub fn get_grade(&self) -> StyleGrade {
        match self.multiplier {
            m if m >= 50.0 => StyleGrade::SSS,
            m if m >= 20.0 => StyleGrade::SS,
            m if m >= 10.0 => StyleGrade::S,
            m if m >= 5.0 => StyleGrade::A,
            m if m >= 3.0 => StyleGrade::B,
            m if m >= 1.5 => StyleGrade::C,
            _ => StyleGrade::D,
        }
    }

    /// Reset for new stage
    pub fn reset_stage(&mut self) {
        self.chain = 0;
        self.chain_timer = 0.0;
        self.multiplier = 1.0;
        self.no_damage_bonus = true;
    }

    /// Reset for new game
    pub fn reset_game(&mut self) {
        *self = Self::default();
    }
}

/// Style grades (like Devil May Cry)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum StyleGrade {
    D,
    C,
    B,
    A,
    S,
    SS,
    SSS,
}

impl StyleGrade {
    pub fn as_str(&self) -> &'static str {
        match self {
            StyleGrade::D => "D",
            StyleGrade::C => "C",
            StyleGrade::B => "B",
            StyleGrade::A => "A",
            StyleGrade::S => "S",
            StyleGrade::SS => "SS",
            StyleGrade::SSS => "SSS",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            StyleGrade::D => Color::srgb(0.5, 0.5, 0.5),
            StyleGrade::C => Color::srgb(0.6, 0.6, 0.4),
            StyleGrade::B => Color::srgb(0.3, 0.7, 0.3),
            StyleGrade::A => Color::srgb(0.3, 0.5, 0.9),
            StyleGrade::S => Color::srgb(0.9, 0.7, 0.2),
            StyleGrade::SS => Color::srgb(1.0, 0.5, 0.2),
            StyleGrade::SSS => Color::srgb(1.0, 0.2, 0.2),
        }
    }
}

/// Salt Miner mode - meter fills from proximity kills, manual activation with B/Y
/// Based on finishing guide: meter 0-100, manual activation, 5x score, 8 second duration
#[derive(Debug, Clone, Resource)]
pub struct SaltMinerSystem {
    /// Salt Miner meter (0.0 to 100.0)
    pub meter: f32,
    /// Meter gained per proximity kill
    pub meter_per_kill: f32,
    /// Proximity range for kills to count (closer = more meter)
    pub proximity_range: f32,
    /// Meter decay rate when not killing (per second)
    pub decay_rate: f32,
    /// Grace period before decay starts (seconds since last kill)
    pub decay_grace: f32,
    /// Time since last kill (for grace period tracking)
    pub time_since_kill: f32,
    /// Whether salt miner mode is active
    pub is_active: bool,
    /// Remaining salt miner duration
    pub timer: f32,
    /// Total salt miner duration
    pub duration: f32,
    /// Score multiplier when active
    pub score_multiplier: f32,
    /// Flash timer for activation effect
    pub activation_flash: f32,
}

impl Default for SaltMinerSystem {
    fn default() -> Self {
        Self {
            meter: 0.0,
            meter_per_kill: 15.0,   // ~7 close kills to fill
            proximity_range: 120.0, // Slightly more forgiving range
            decay_rate: 5.0,        // Slow decay when not killing
            decay_grace: 1.5,       // 1.5s grace before decay starts
            time_since_kill: 0.0,
            is_active: false,
            timer: 0.0,
            duration: 8.0,         // 8 seconds when activated
            score_multiplier: 5.0, // 5x score as per guide
            activation_flash: 0.0,
        }
    }
}

impl SaltMinerSystem {
    /// Register a kill at distance. Fills meter based on proximity.
    /// Closer kills fill more meter. Returns meter gained.
    pub fn on_kill_at_distance(&mut self, distance: f32) -> f32 {
        if self.is_active {
            return 0.0; // Already active, no meter gain
        }

        // Reset decay grace timer on kill
        self.time_since_kill = 0.0;

        // Calculate meter gain based on proximity (closer = more)
        let proximity_bonus = if distance <= self.proximity_range {
            // Linear falloff: point-blank = 100%, max range = 50%
            let normalized = distance / self.proximity_range;
            1.0 - (normalized * 0.5)
        } else {
            // Outside range: minimal gain
            0.25
        };

        let gain = self.meter_per_kill * proximity_bonus;
        self.meter = (self.meter + gain).min(100.0);
        gain
    }

    /// Legacy on_kill for compatibility (assumes point-blank)
    pub fn on_kill(&mut self) {
        self.on_kill_at_distance(0.0);
    }

    /// Check if salt miner can be activated (meter full)
    pub fn can_activate(&self) -> bool {
        !self.is_active && self.meter >= 100.0
    }

    /// Try to activate salt miner. Returns true if activated.
    pub fn try_activate(&mut self) -> bool {
        if self.can_activate() {
            self.is_active = true;
            self.timer = self.duration;
            self.meter = 0.0;
            self.activation_flash = 0.5; // Half second flash
            return true;
        }
        false
    }

    /// Update salt miner state (call each frame)
    pub fn update(&mut self, dt: f32) {
        // Update activation flash
        if self.activation_flash > 0.0 {
            self.activation_flash = (self.activation_flash - dt).max(0.0);
        }

        if self.is_active {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.is_active = false;
            }
        } else {
            // Track time since last kill
            self.time_since_kill += dt;

            // Decay meter slowly when not killing (after grace period)
            if self.meter > 0.0 && self.time_since_kill > self.decay_grace {
                self.meter = (self.meter - self.decay_rate * dt).max(0.0);
            }
        }
    }

    /// Get score multiplier (5x when active)
    pub fn score_mult(&self) -> f32 {
        if self.is_active {
            self.score_multiplier
        } else {
            1.0
        }
    }

    /// Get damage multiplier (2x when active)
    pub fn damage_mult(&self) -> f32 {
        if self.is_active {
            2.0
        } else {
            1.0
        }
    }

    /// Get speed multiplier (1.5x when active)
    pub fn speed_mult(&self) -> f32 {
        if self.is_active {
            1.5
        } else {
            1.0
        }
    }

    /// Get progress toward salt miner (0.0 - 1.0)
    /// When active, shows remaining duration. When inactive, shows meter fill.
    pub fn progress(&self) -> f32 {
        if self.is_active {
            self.timer / self.duration
        } else {
            self.meter / 100.0
        }
    }

    /// Get meter percentage (0.0 - 1.0)
    pub fn meter_percent(&self) -> f32 {
        self.meter / 100.0
    }

    /// Check if activation flash is active (for visual effects)
    pub fn is_flashing(&self) -> bool {
        self.activation_flash > 0.0
    }

    /// Reset salt miner state (for new stage)
    pub fn reset(&mut self) {
        self.meter = 0.0;
        self.is_active = false;
        self.timer = 0.0;
        self.activation_flash = 0.0;
        self.time_since_kill = 0.0;
    }
}

/// Game currency and progression
#[derive(Debug, Clone, Resource, Default)]
pub struct GameProgress {
    /// In-run currency for upgrades
    pub credits: u64,
    /// Lifetime currency for unlocks
    pub isk: u64,
    /// Highest stage reached
    pub highest_stage: u32,
    /// Campaigns completed
    pub campaigns_completed: Vec<String>,
    /// Ships unlocked
    pub ships_unlocked: Vec<u32>,
    /// Achievements unlocked
    pub achievements: Vec<String>,
}

/// Player input configuration
#[derive(Debug, Clone, Resource)]
pub struct InputConfig {
    pub controller_enabled: bool,
    pub controller_deadzone: f32,
    pub keyboard_enabled: bool,
    pub mouse_enabled: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            controller_enabled: true,
            controller_deadzone: 0.15,
            keyboard_enabled: true,
            mouse_enabled: true,
        }
    }
}

/// Audio settings
#[derive(Debug, Clone, Resource)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub music_enabled: bool,
    pub sfx_enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            music_enabled: true,
            sfx_enabled: true,
        }
    }
}

/// Difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum DifficultyLevel {
    /// Easy - Relaxed gameplay, generous shields, forgiving combat
    Carebear,
    /// Normal - Balanced experience for new pilots
    #[default]
    Newbro,
    /// Hard - Punishing difficulty for experienced pilots
    BitterVet,
    /// Nightmare - One-shot kills, relentless enemies, no mercy
    Triglavian,
}

impl DifficultyLevel {
    pub fn name(&self) -> &'static str {
        match self {
            DifficultyLevel::Carebear => "CAREBEAR",
            DifficultyLevel::Newbro => "NEWBRO",
            DifficultyLevel::BitterVet => "BITTER VET",
            DifficultyLevel::Triglavian => "TRIGLAVIAN",
        }
    }

    pub fn tagline(&self) -> &'static str {
        match self {
            DifficultyLevel::Carebear => "High-sec living",
            DifficultyLevel::Newbro => "Welcome to New Eden",
            DifficultyLevel::BitterVet => "I remember when...",
            DifficultyLevel::Triglavian => "Clade proving grounds",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DifficultyLevel::Carebear => {
                "Relaxed gameplay with generous shields and forgiving combat."
            }
            DifficultyLevel::Newbro => {
                "Balanced experience for new pilots. Fair challenge with room to learn."
            }
            DifficultyLevel::BitterVet => {
                "Punishing difficulty for experienced pilots. Enemies hit hard."
            }
            DifficultyLevel::Triglavian => {
                "Nightmare mode. One-shot kills, relentless enemies, no mercy."
            }
        }
    }

    pub fn color(&self) -> Color {
        match self {
            DifficultyLevel::Carebear => Color::srgb(0.4, 0.8, 0.4), // Green
            DifficultyLevel::Newbro => Color::srgb(0.4, 0.6, 1.0),   // Blue
            DifficultyLevel::BitterVet => Color::srgb(1.0, 0.6, 0.2), // Orange
            DifficultyLevel::Triglavian => Color::srgb(0.8, 0.2, 0.2), // Red
        }
    }

    /// Get all difficulty levels in order
    pub fn all() -> [DifficultyLevel; 4] {
        [
            DifficultyLevel::Carebear,
            DifficultyLevel::Newbro,
            DifficultyLevel::BitterVet,
            DifficultyLevel::Triglavian,
        ]
    }

    /// Get the next difficulty (wraps around)
    pub fn next(&self) -> DifficultyLevel {
        match self {
            DifficultyLevel::Carebear => DifficultyLevel::Newbro,
            DifficultyLevel::Newbro => DifficultyLevel::BitterVet,
            DifficultyLevel::BitterVet => DifficultyLevel::Triglavian,
            DifficultyLevel::Triglavian => DifficultyLevel::Carebear,
        }
    }

    /// Get the previous difficulty (wraps around)
    pub fn prev(&self) -> DifficultyLevel {
        match self {
            DifficultyLevel::Carebear => DifficultyLevel::Triglavian,
            DifficultyLevel::Newbro => DifficultyLevel::Carebear,
            DifficultyLevel::BitterVet => DifficultyLevel::Newbro,
            DifficultyLevel::Triglavian => DifficultyLevel::BitterVet,
        }
    }
}

/// Player stat modifiers based on difficulty
#[derive(Debug, Clone, Copy)]
pub struct PlayerModifiers {
    pub hull_multiplier: f32,
    pub shield_multiplier: f32,
    pub armor_multiplier: f32,
    pub damage_multiplier: f32,
    pub capacitor_recharge_multiplier: f32,
    pub capacitor_drain_multiplier: f32,
    pub maneuver_cooldown_multiplier: f32,
    pub invincibility_duration_multiplier: f32,
}

impl Default for PlayerModifiers {
    fn default() -> Self {
        Self {
            hull_multiplier: 1.0,
            shield_multiplier: 1.0,
            armor_multiplier: 1.0,
            damage_multiplier: 1.0,
            capacitor_recharge_multiplier: 1.0,
            capacitor_drain_multiplier: 1.0,
            maneuver_cooldown_multiplier: 1.0,
            invincibility_duration_multiplier: 1.0,
        }
    }
}

/// Enemy stat modifiers based on difficulty
#[derive(Debug, Clone, Copy)]
pub struct EnemyModifiers {
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
    pub fire_rate_multiplier: f32,
    pub speed_multiplier: f32,
    pub accuracy_multiplier: f32,
    pub spawn_rate_multiplier: f32,
}

impl Default for EnemyModifiers {
    fn default() -> Self {
        Self {
            health_multiplier: 1.0,
            damage_multiplier: 1.0,
            fire_rate_multiplier: 1.0,
            speed_multiplier: 1.0,
            accuracy_multiplier: 1.0,
            spawn_rate_multiplier: 1.0,
        }
    }
}

/// Boss modifiers based on difficulty
#[derive(Debug, Clone, Copy)]
pub struct BossModifiers {
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
    pub attack_cooldown_multiplier: f32,
}

impl Default for BossModifiers {
    fn default() -> Self {
        Self {
            health_multiplier: 1.0,
            damage_multiplier: 1.0,
            attack_cooldown_multiplier: 1.0,
        }
    }
}

/// Scoring modifiers based on difficulty
#[derive(Debug, Clone, Copy)]
pub struct ScoringModifiers {
    pub base_score_multiplier: f32,
    pub combo_decay_multiplier: f32,
}

impl Default for ScoringModifiers {
    fn default() -> Self {
        Self {
            base_score_multiplier: 1.0,
            combo_decay_multiplier: 1.0,
        }
    }
}

/// Complete difficulty settings resource
#[derive(Debug, Clone, Resource)]
pub struct DifficultySettings {
    pub level: DifficultyLevel,
    pub player: PlayerModifiers,
    pub enemy: EnemyModifiers,
    pub boss: BossModifiers,
    pub scoring: ScoringModifiers,
}

impl Default for DifficultySettings {
    fn default() -> Self {
        Self::from_level(DifficultyLevel::default())
    }
}

impl DifficultySettings {
    /// Create settings for a specific difficulty level
    pub fn from_level(level: DifficultyLevel) -> Self {
        match level {
            DifficultyLevel::Carebear => Self {
                level,
                player: PlayerModifiers {
                    hull_multiplier: 1.5,
                    shield_multiplier: 2.0,
                    armor_multiplier: 1.5,
                    damage_multiplier: 1.2,
                    capacitor_recharge_multiplier: 1.5,
                    capacitor_drain_multiplier: 0.7,
                    maneuver_cooldown_multiplier: 0.7,
                    invincibility_duration_multiplier: 1.5,
                },
                enemy: EnemyModifiers {
                    health_multiplier: 0.7,
                    damage_multiplier: 0.5,
                    fire_rate_multiplier: 0.7,
                    speed_multiplier: 0.85,
                    accuracy_multiplier: 0.6,
                    spawn_rate_multiplier: 0.8,
                },
                boss: BossModifiers {
                    health_multiplier: 0.6,
                    damage_multiplier: 0.5,
                    attack_cooldown_multiplier: 1.3,
                },
                scoring: ScoringModifiers {
                    base_score_multiplier: 0.5,
                    combo_decay_multiplier: 0.7,
                },
            },
            DifficultyLevel::Newbro => Self {
                level,
                player: PlayerModifiers::default(),
                enemy: EnemyModifiers::default(),
                boss: BossModifiers::default(),
                scoring: ScoringModifiers::default(),
            },
            DifficultyLevel::BitterVet => Self {
                level,
                player: PlayerModifiers {
                    hull_multiplier: 0.8,
                    shield_multiplier: 0.8,
                    armor_multiplier: 0.8,
                    damage_multiplier: 0.9,
                    capacitor_recharge_multiplier: 0.8,
                    capacitor_drain_multiplier: 1.2,
                    maneuver_cooldown_multiplier: 1.2,
                    invincibility_duration_multiplier: 0.8,
                },
                enemy: EnemyModifiers {
                    health_multiplier: 1.3,
                    damage_multiplier: 1.5,
                    fire_rate_multiplier: 1.3,
                    speed_multiplier: 1.15,
                    accuracy_multiplier: 1.3,
                    spawn_rate_multiplier: 1.2,
                },
                boss: BossModifiers {
                    health_multiplier: 1.4,
                    damage_multiplier: 1.5,
                    attack_cooldown_multiplier: 0.8,
                },
                scoring: ScoringModifiers {
                    base_score_multiplier: 1.5,
                    combo_decay_multiplier: 1.3,
                },
            },
            DifficultyLevel::Triglavian => Self {
                level,
                player: PlayerModifiers {
                    hull_multiplier: 0.5,
                    shield_multiplier: 0.5,
                    armor_multiplier: 0.5,
                    damage_multiplier: 0.8,
                    capacitor_recharge_multiplier: 0.6,
                    capacitor_drain_multiplier: 1.5,
                    maneuver_cooldown_multiplier: 1.4,
                    invincibility_duration_multiplier: 0.5,
                },
                enemy: EnemyModifiers {
                    health_multiplier: 1.5,
                    damage_multiplier: 3.0,
                    fire_rate_multiplier: 1.5,
                    speed_multiplier: 1.3,
                    accuracy_multiplier: 1.5,
                    spawn_rate_multiplier: 1.5,
                },
                boss: BossModifiers {
                    health_multiplier: 2.0,
                    damage_multiplier: 2.5,
                    attack_cooldown_multiplier: 0.6,
                },
                scoring: ScoringModifiers {
                    base_score_multiplier: 3.0,
                    combo_decay_multiplier: 2.0,
                },
            },
        }
    }

    /// Set difficulty level and update all modifiers
    pub fn set_level(&mut self, level: DifficultyLevel) {
        *self = Self::from_level(level);
    }
}

// =============================================================================
// ENDLESS MODE
// =============================================================================

/// Endless mode state - survive infinite waves with escalating difficulty
#[derive(Debug, Clone, Resource)]
pub struct EndlessMode {
    /// Is endless mode active
    pub active: bool,
    /// Current wave in endless mode
    pub wave: u32,
    /// Highest wave reached (for high score)
    pub best_wave: u32,
    /// Survival time in seconds
    pub time_survived: f32,
    /// Best survival time
    pub best_time: f32,
    /// Enemies killed in this run
    pub kills: u32,
    /// Mini-bosses defeated
    pub mini_bosses_defeated: u32,
    /// Difficulty escalation factor (increases over time)
    pub escalation: f32,
}

impl Default for EndlessMode {
    fn default() -> Self {
        Self {
            active: false,
            wave: 0,
            best_wave: 0,
            time_survived: 0.0,
            best_time: 0.0,
            kills: 0,
            mini_bosses_defeated: 0,
            escalation: 1.0,
        }
    }
}

impl EndlessMode {
    /// Start a new endless mode run
    pub fn start(&mut self) {
        self.active = true;
        self.wave = 0;
        self.time_survived = 0.0;
        self.kills = 0;
        self.mini_bosses_defeated = 0;
        self.escalation = 1.0;
    }

    /// End the current run and update best scores
    pub fn end_run(&mut self) {
        self.active = false;
        if self.wave > self.best_wave {
            self.best_wave = self.wave;
        }
        if self.time_survived > self.best_time {
            self.best_time = self.time_survived;
        }
    }

    /// Advance to next wave
    pub fn next_wave(&mut self) {
        self.wave += 1;
        // Escalation increases 5% per wave, capping at 3x
        self.escalation = (1.0 + self.wave as f32 * 0.05).min(3.0);
    }

    /// Check if it's time for a mini-boss (every 10 waves)
    pub fn is_mini_boss_wave(&self) -> bool {
        self.wave > 0 && self.wave % 10 == 0
    }

    /// Get enemy count for current wave
    pub fn wave_enemy_count(&self) -> u32 {
        let base = 4 + self.wave / 2;
        (base as f32 * self.escalation).min(25.0) as u32
    }

    /// Get enemy health multiplier for current wave
    pub fn enemy_health_mult(&self) -> f32 {
        self.escalation
    }

    /// Get enemy damage multiplier for current wave
    pub fn enemy_damage_mult(&self) -> f32 {
        (1.0 + self.wave as f32 * 0.02).min(2.5)
    }

    /// Get enemy speed multiplier for current wave
    pub fn enemy_speed_mult(&self) -> f32 {
        (1.0 + self.wave as f32 * 0.01).min(1.5)
    }

    /// Format survival time as MM:SS
    pub fn time_display(&self) -> String {
        let minutes = (self.time_survived / 60.0) as u32;
        let seconds = (self.time_survived % 60.0) as u32;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    // ==================== ScoreSystem Tests ====================

    #[test]
    fn score_system_default_values() {
        let s = ScoreSystem::default();
        assert_eq!(s.score, 0);
        assert_eq!(s.multiplier, 1.0);
        assert_eq!(s.chain, 0);
        assert!(s.no_damage_bonus);
    }

    #[test]
    fn score_system_add_score_applies_multiplier() {
        let mut s = ScoreSystem {
            multiplier: 2.0,
            ..Default::default()
        };
        s.add_score(100);
        assert_eq!(s.score, 200);
    }

    #[test]
    fn score_system_on_kill_extends_chain() {
        let mut s = ScoreSystem::default();
        s.on_kill(100);
        assert_eq!(s.chain, 1);
        assert_eq!(s.chain_timer, 2.5); // CHAIN_TIMEOUT
        assert_eq!(s.multiplier, 1.1); // 1.0 + 1 * 0.1
    }

    #[test]
    fn score_system_multiplier_caps_at_99_9() {
        let mut s = ScoreSystem::default();
        // Kill 1000 times to push multiplier
        for _ in 0..1000 {
            s.on_kill(1);
        }
        assert!(s.multiplier <= 99.9);
        assert!(s.multiplier >= 99.0);
    }

    #[test]
    fn score_system_chain_timer_decay_resets_chain() {
        let mut s = ScoreSystem::default();
        s.on_kill(100);
        assert_eq!(s.chain, 1);

        // Simulate time passing (past CHAIN_TIMEOUT of 2.5s)
        s.update(2.6);
        assert_eq!(s.chain, 0);
        assert_eq!(s.multiplier, 1.0);
    }

    #[test]
    fn score_system_grades() {
        let mut s = ScoreSystem::default();

        s.multiplier = 0.5;
        assert_eq!(s.get_grade(), StyleGrade::D);

        s.multiplier = 1.5;
        assert_eq!(s.get_grade(), StyleGrade::C);

        s.multiplier = 3.0;
        assert_eq!(s.get_grade(), StyleGrade::B);

        s.multiplier = 5.0;
        assert_eq!(s.get_grade(), StyleGrade::A);

        s.multiplier = 10.0;
        assert_eq!(s.get_grade(), StyleGrade::S);

        s.multiplier = 20.0;
        assert_eq!(s.get_grade(), StyleGrade::SS);

        s.multiplier = 50.0;
        assert_eq!(s.get_grade(), StyleGrade::SSS);
    }

    #[test]
    fn score_system_reset_stage() {
        let mut s = ScoreSystem::default();
        s.on_kill(100);
        s.score = 5000;
        s.no_damage_bonus = false;

        s.reset_stage();

        assert_eq!(s.chain, 0);
        assert_eq!(s.multiplier, 1.0);
        assert!(s.no_damage_bonus);
        // Score persists through stage reset
        assert_eq!(s.score, 5000);
    }

    #[test]
    fn score_system_reset_game() {
        let mut s = ScoreSystem::default();
        s.on_kill(100);
        s.score = 5000;
        s.souls_liberated = 42;

        s.reset_game();

        assert_eq!(s.score, 0);
        assert_eq!(s.souls_liberated, 0);
        assert_eq!(s.multiplier, 1.0);
    }

    // ==================== SaltMinerSystem Tests ====================

    #[test]
    fn salt_miner_default_values() {
        let b = SaltMinerSystem::default();
        assert_eq!(b.meter, 0.0);
        assert_eq!(b.meter_per_kill, 15.0);
        assert_eq!(b.proximity_range, 120.0);
        assert_eq!(b.duration, 8.0);
        assert_eq!(b.score_multiplier, 5.0);
        assert!(!b.is_active);
    }

    #[test]
    fn salt_miner_proximity_kill_fills_meter() {
        let mut b = SaltMinerSystem::default();
        let gain = b.on_kill_at_distance(0.0); // Point blank
        assert!(gain > 0.0);
        assert!(b.meter > 0.0);
    }

    #[test]
    fn salt_miner_closer_kills_give_more_meter() {
        let mut b1 = SaltMinerSystem::default();
        let mut b2 = SaltMinerSystem::default();

        let gain_close = b1.on_kill_at_distance(0.0);
        let gain_far = b2.on_kill_at_distance(100.0);

        assert!(gain_close > gain_far, "closer kills should give more meter");
    }

    #[test]
    fn salt_miner_cannot_activate_when_meter_not_full() {
        let mut b = SaltMinerSystem::default();
        b.meter = 50.0;
        assert!(!b.can_activate());
        assert!(!b.try_activate());
        assert!(!b.is_active);
    }

    #[test]
    fn salt_miner_activates_when_meter_full_and_triggered() {
        let mut b = SaltMinerSystem::default();
        b.meter = 100.0;
        assert!(b.can_activate());
        assert!(b.try_activate());
        assert!(b.is_active);
        assert_eq!(b.timer, 8.0);
        assert_eq!(b.meter, 0.0); // Reset after activation
    }

    #[test]
    fn salt_miner_multipliers_when_active() {
        let mut b = SaltMinerSystem::default();
        assert_eq!(b.score_mult(), 1.0);
        assert_eq!(b.damage_mult(), 1.0);
        assert_eq!(b.speed_mult(), 1.0);

        // Activate
        b.meter = 100.0;
        b.try_activate();

        assert_eq!(b.score_mult(), 5.0);
        assert_eq!(b.damage_mult(), 2.0);
        assert_eq!(b.speed_mult(), 1.5);
    }

    #[test]
    fn salt_miner_duration_decay() {
        let mut b = SaltMinerSystem::default();
        b.meter = 100.0;
        b.try_activate();
        assert!(b.is_active);

        b.update(4.0);
        assert!(b.is_active);
        assert_eq!(b.timer, 4.0);

        b.update(4.1);
        assert!(!b.is_active);
    }

    #[test]
    fn salt_miner_meter_decays_when_not_killing() {
        let mut b = SaltMinerSystem::default();
        b.meter = 50.0;
        b.time_since_kill = 2.0; // Past grace period (1.5s)

        b.update(2.0); // 2 seconds at 5.0/s decay = -10
        assert!((b.meter - 40.0).abs() < 0.1);
    }

    #[test]
    fn salt_miner_progress_calculation() {
        let mut b = SaltMinerSystem::default();
        assert_eq!(b.progress(), 0.0);

        b.meter = 50.0;
        assert!((b.progress() - 0.5).abs() < 0.01); // 50%

        // Activate
        b.meter = 100.0;
        b.try_activate();
        assert_eq!(b.progress(), 1.0); // Full timer

        b.update(4.0);
        assert!((b.progress() - 0.5).abs() < 0.01); // Half timer
    }

    #[test]
    fn salt_miner_reset() {
        let mut b = SaltMinerSystem::default();
        b.meter = 75.0;
        b.is_active = true;
        b.timer = 3.0;

        b.reset();

        assert_eq!(b.meter, 0.0);
        assert!(!b.is_active);
        assert_eq!(b.timer, 0.0);
    }

    #[test]
    fn salt_miner_activation_flash() {
        let mut b = SaltMinerSystem::default();
        b.meter = 100.0;
        b.try_activate();

        assert!(b.is_flashing());
        assert!(b.activation_flash > 0.0);

        b.update(0.6); // Wait past flash duration
        assert!(!b.is_flashing());
    }

    // ==================== DifficultyLevel Tests ====================

    #[test]
    fn difficulty_level_cycling() {
        let d = DifficultyLevel::Carebear;
        assert_eq!(d.next(), DifficultyLevel::Newbro);
        assert_eq!(d.next().next(), DifficultyLevel::BitterVet);
        assert_eq!(d.next().next().next(), DifficultyLevel::Triglavian);
        assert_eq!(d.next().next().next().next(), DifficultyLevel::Carebear);
    }

    #[test]
    fn difficulty_level_prev_cycling() {
        let d = DifficultyLevel::Carebear;
        assert_eq!(d.prev(), DifficultyLevel::Triglavian);
        assert_eq!(DifficultyLevel::Newbro.prev(), DifficultyLevel::Carebear);
    }

    #[test]
    fn difficulty_level_names() {
        assert_eq!(DifficultyLevel::Carebear.name(), "CAREBEAR");
        assert_eq!(DifficultyLevel::Newbro.name(), "NEWBRO");
        assert_eq!(DifficultyLevel::BitterVet.name(), "BITTER VET");
        assert_eq!(DifficultyLevel::Triglavian.name(), "TRIGLAVIAN");
    }

    // ==================== DifficultySettings Tests ====================

    #[test]
    fn difficulty_settings_carebear_is_easier() {
        let settings = DifficultySettings::from_level(DifficultyLevel::Carebear);

        // Player should be stronger
        assert!(settings.player.hull_multiplier > 1.0);
        assert!(settings.player.shield_multiplier > 1.0);
        assert!(settings.player.damage_multiplier > 1.0);

        // Enemies should be weaker
        assert!(settings.enemy.health_multiplier < 1.0);
        assert!(settings.enemy.damage_multiplier < 1.0);

        // Score multiplier lower (easy mode = less reward)
        assert!(settings.scoring.base_score_multiplier < 1.0);
    }

    #[test]
    fn difficulty_settings_newbro_is_baseline() {
        let settings = DifficultySettings::from_level(DifficultyLevel::Newbro);

        assert_eq!(settings.player.hull_multiplier, 1.0);
        assert_eq!(settings.enemy.health_multiplier, 1.0);
        assert_eq!(settings.scoring.base_score_multiplier, 1.0);
    }

    #[test]
    fn difficulty_settings_bittrevet_is_harder() {
        let settings = DifficultySettings::from_level(DifficultyLevel::BitterVet);

        // Player should be weaker
        assert!(settings.player.hull_multiplier < 1.0);

        // Enemies should be stronger
        assert!(settings.enemy.health_multiplier > 1.0);
        assert!(settings.enemy.damage_multiplier > 1.0);

        // Score multiplier higher (hard mode = more reward)
        assert!(settings.scoring.base_score_multiplier > 1.0);
    }

    #[test]
    fn difficulty_settings_triglavian_is_nightmare() {
        let settings = DifficultySettings::from_level(DifficultyLevel::Triglavian);

        // Player very weak
        assert!(settings.player.hull_multiplier <= 0.5);

        // Enemies very strong - the 3.0x damage
        assert!(settings.enemy.damage_multiplier >= 3.0);

        // Boss doubled health
        assert!(settings.boss.health_multiplier >= 2.0);

        // Score multiplier highest
        assert!(settings.scoring.base_score_multiplier >= 3.0);
    }

    #[test]
    fn difficulty_settings_set_level() {
        let mut settings = DifficultySettings::default();
        assert_eq!(settings.level, DifficultyLevel::Newbro);

        settings.set_level(DifficultyLevel::Triglavian);
        assert_eq!(settings.level, DifficultyLevel::Triglavian);
        assert!(settings.enemy.damage_multiplier >= 3.0);
    }
}
