//! HUD shared components and types
//!
//! Marker components and enums used across HUD submodules.

use bevy::prelude::*;

/// Marker for HUD root
#[derive(Component)]
pub struct HudRoot;

/// Score text
#[derive(Component)]
pub struct ScoreText;

/// Combo/multiplier text
#[derive(Component)]
pub struct ComboText;

/// Style grade text
#[derive(Component)]
pub struct GradeText;

/// Salt Miner meter bar
#[derive(Component)]
pub struct SaltMinerBar;

/// Heat bar
#[derive(Component)]
pub struct HeatBar;

/// Combo kill count text
#[derive(Component)]
pub struct ComboKillsText;

/// Combo timer bar container (for show/hide)
#[derive(Component)]
pub struct ComboTimerContainer;

/// Combo timer bar fill (shows time remaining to keep combo)
#[derive(Component)]
pub struct ComboTimerBar;

/// Wave display text
#[derive(Component)]
pub struct WaveText;

/// Mission name text
#[derive(Component)]
pub struct MissionNameText;

/// Mission objective text
#[derive(Component)]
pub struct ObjectiveText;

/// Souls liberated text
#[derive(Component)]
pub struct SoulsText;

/// Enemies killed counter text
#[derive(Component)]
pub struct KillCountText;

/// Powerup indicator container
#[derive(Component)]
pub struct PowerupIndicator;

/// Overdrive indicator
#[derive(Component)]
pub struct OverdriveIndicator;

/// Damage boost indicator
#[derive(Component)]
pub struct DamageBoostIndicator;

/// Invuln indicator
#[derive(Component)]
pub struct InvulnIndicator;

/// Timer bar for powerup effects (depletes over time)
#[derive(Component)]
pub struct PowerupTimerBar {
    /// Which powerup this bar is for
    pub powerup_type: PowerupType,
}

/// Countdown text for expiring buffs
#[derive(Component)]
pub struct PowerupCountdown {
    pub powerup_type: PowerupType,
}

/// Screen edge warning overlay for expiring buffs (one per edge)
#[derive(Component)]
pub struct BuffExpirationWarning {
    pub edge: ScreenEdge,
}

/// Which edge of the screen this warning covers
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScreenEdge {
    Top,
    Bottom,
    Left,
    Right,
}

/// Powerup type for status bar tracking
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PowerupType {
    Overdrive,
    DamageBoost,
    Invulnerability,
}

/// Container for a single powerup status box
#[derive(Component)]
pub struct PowerupStatusBox {
    pub powerup_type: PowerupType,
}

/// Boss health bar container
#[derive(Component)]
pub struct BossHealthContainer;

/// Boss health bar fill
#[derive(Component)]
pub struct BossHealthFill;

/// Boss name text
#[derive(Component)]
pub struct BossNameText;

/// Stage display text
#[derive(Component)]
pub struct StageText;

/// Dialogue box container
#[derive(Component)]
pub struct DialogueContainer;

/// Dialogue speaker name text
#[derive(Component)]
pub struct DialogueSpeakerText;

/// Dialogue content text
#[derive(Component)]
pub struct DialogueContentText;

/// Wingman gauge container
#[derive(Component)]
pub struct WingmanGauge;

/// Wingman gauge fill bar
#[derive(Component)]
pub struct WingmanGaugeFill;

/// Wingman count text
#[derive(Component)]
pub struct WingmanCountText;

/// Drone status container
#[derive(Component)]
pub struct DroneStatusContainer;

/// Drone status text (count + lifetime)
#[derive(Component)]
pub struct DroneStatusText;

/// Ability indicator container
#[derive(Component)]
pub struct AbilityIndicatorContainer;

/// Ability indicator fill bar
#[derive(Component)]
pub struct AbilityIndicatorFill;

/// Ability indicator name text
#[derive(Component)]
pub struct AbilityIndicatorText;

/// Ability cooldown key hint
#[derive(Component)]
pub struct AbilityKeyHint;

/// Ammo type display text
#[derive(Component)]
pub struct AmmoTypeText;

/// Achievement popup container
#[derive(Component)]
pub struct AchievementPopup;

/// Achievement popup text (name)
#[derive(Component)]
pub struct AchievementPopupName;

/// Achievement popup description
#[derive(Component)]
pub struct AchievementPopupDesc;

/// Warning threshold for buff expiration (seconds)
pub const BUFF_WARNING_THRESHOLD: f32 = 2.0;
