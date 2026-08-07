//! Save/Load System
//!
//! Persists player progression, unlocks, and settings.

#![allow(dead_code)]

use crate::core::KeyBindings;
use crate::systems::{RumbleSettings, ScreenShake, SoundSettings};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// Save system plugin
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveData>()
            .add_systems(Startup, load_save_data)
            .add_systems(PostStartup, apply_saved_settings)
            .add_systems(Update, auto_save.run_if(resource_changed::<SaveData>))
            .add_systems(Update, sync_settings_to_save);
    }
}

/// Persistent save data
#[derive(Resource, Serialize, Deserialize, Clone, Debug, Default)]
pub struct SaveData {
    /// Highest stage completed per faction pair
    pub stage_progress: Vec<FactionProgress>,
    /// Ships unlocked (by type_id)
    pub unlocked_ships: HashSet<u32>,
    /// Total credits earned (lifetime)
    pub lifetime_credits: u64,
    /// High scores per faction pair
    pub high_scores: Vec<HighScore>,
    /// Settings
    pub settings: GameSettings,
    /// Player's remapped control layout.
    ///
    /// Migrations:
    /// - Pre-keybindings save (no `keybindings` field) → defaults via `KeyBindings::default()`.
    /// - Default save (no remap ever) → defaults.
    /// - Custom remap → exactly what the player saved.
    #[serde(default)]
    pub keybindings: super::KeyBindings,
    /// Unlocked achievements
    #[serde(default)]
    pub achievements: HashSet<super::Achievement>,
    /// Lifetime statistics
    #[serde(default)]
    pub lifetime_stats: LifetimeStats,
    /// Available skill points (persistent currency)
    #[serde(default)]
    pub skill_points: u32,
    /// Purchased upgrades
    #[serde(default)]
    pub purchased_upgrades: HashSet<super::Upgrade>,
    /// Gameplay analytics (deaths, ship picks, difficulty distribution)
    #[serde(default)]
    pub analytics: super::AnalyticsData,
    /// Top-10 leaderboard across all runs
    #[serde(default)]
    pub leaderboard: Vec<LeaderboardEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FactionProgress {
    pub player_faction: String,
    pub enemy_faction: String,
    pub highest_stage: u32,
    pub highest_mission: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HighScore {
    pub player_faction: String,
    pub enemy_faction: String,
    pub score: u64,
    pub stage: u32,
}

/// A single leaderboard entry recording a completed run
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LeaderboardEntry {
    /// Player faction for this run
    pub faction: String,
    /// Enemy faction for this run
    pub enemy: String,
    /// Final score
    pub score: u64,
    /// Stage reached
    pub stage: u32,
    /// Difficulty setting (e.g. "Normal", "Hard")
    pub difficulty: String,
    /// Ship name used
    pub ship_name: String,
    /// Session counter (monotonically increasing per save)
    pub session: u32,
}

/// Lifetime statistics for tracking total progress
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LifetimeStats {
    /// Total enemies destroyed across all runs
    pub total_kills: u64,
    /// Total souls liberated across all runs
    pub total_souls: u64,
    /// Total games played
    pub games_played: u32,
    /// Total missions completed
    pub missions_completed: u32,
    /// Total bosses defeated
    pub bosses_defeated: u32,
    /// Highest combo ever achieved
    pub highest_combo: u32,
    /// Highest score ever achieved
    pub highest_score: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameSettings {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    /// Screen shake intensity (0.0 = off, 1.0 = full)
    #[serde(default = "default_shake_intensity")]
    pub screen_shake_intensity: f32,
    /// Controller rumble intensity (0.0 = off, 1.0 = full)
    #[serde(default = "default_rumble_intensity")]
    pub rumble_intensity: f32,
}

fn default_shake_intensity() -> f32 {
    1.0
}

fn default_rumble_intensity() -> f32 {
    1.0
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            sfx_volume: 0.8,
            music_volume: 0.5,
            screen_shake_intensity: 1.0,
            rumble_intensity: 1.0,
        }
    }
}

impl SaveData {
    /// Get save file path (native only).
    ///
    /// Honors the `REBELLION_HOME` env var for test isolation —
    /// set it to a temp dir before calling `load()` or `save()`
    /// to keep your real save file untouched. Defaults to the
    /// platform `data_dir()/rebellion/save.json`. The env var is
    /// cheap to read (no allocation on the hot path), so we check
    /// it every call rather than caching at startup — caching
    /// would break tests that mutate it mid-process.
    #[cfg(not(target_arch = "wasm32"))]
    fn save_path() -> PathBuf {
        if let Ok(root) = std::env::var("REBELLION_HOME") {
            return PathBuf::from(root).join("save.json");
        }
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rebellion")
            .join("save.json")
    }

    /// Load from disk (native)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load() -> Self {
        let path = Self::save_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(data) => match serde_json::from_str(&data) {
                    Ok(save) => {
                        info!("Loaded save data from {:?}", path);
                        return save;
                    }
                    Err(e) => warn!("Failed to parse save data: {}", e),
                },
                Err(e) => warn!("Failed to read save file: {}", e),
            }
        }
        info!("No save data found, using defaults");
        Self::default()
    }

    /// Load from localStorage (WASM)
    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Self {
        use web_sys::window;

        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(data)) = storage.get_item("rebellion_save") {
                    if let Ok(save) = serde_json::from_str(&data) {
                        info!("Loaded save data from localStorage");
                        return save;
                    }
                }
            }
        }
        info!("No save data found, using defaults");
        Self::default()
    }

    /// Save to disk (native)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save(&self) {
        let path = Self::save_path();

        // Create directory if needed
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                warn!("Failed to create save directory: {}", e);
                return;
            }
        }

        match serde_json::to_string_pretty(self) {
            Ok(data) => {
                if let Err(e) = fs::write(&path, data) {
                    warn!("Failed to write save file: {}", e);
                } else {
                    info!("Saved progress to {:?}", path);
                }
            }
            Err(e) => warn!("Failed to serialize save data: {}", e),
        }
    }

    /// Save to localStorage (WASM)
    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        use web_sys::window;

        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(data) = serde_json::to_string(self) {
                    if storage.set_item("rebellion_save", &data).is_ok() {
                        info!("Saved progress to localStorage");
                    }
                }
            }
        }
    }

    /// Check if a ship is unlocked
    pub fn is_ship_unlocked(
        &self,
        type_id: u32,
        unlock_stage: u32,
        faction: &str,
        enemy: &str,
    ) -> bool {
        // Stage 0 = always unlocked
        if unlock_stage == 0 {
            return true;
        }

        // Check if explicitly unlocked
        if self.unlocked_ships.contains(&type_id) {
            return true;
        }

        // Check faction progress
        for progress in &self.stage_progress {
            if progress.player_faction == faction && progress.enemy_faction == enemy {
                return progress.highest_stage >= unlock_stage;
            }
        }

        false
    }

    /// Unlock a ship
    pub fn unlock_ship(&mut self, type_id: u32) {
        self.unlocked_ships.insert(type_id);
    }

    /// Record stage completion
    pub fn complete_stage(&mut self, faction: &str, enemy: &str, stage: u32, mission: u32) {
        // Find or create progress entry
        let mut found = false;
        for progress in &mut self.stage_progress {
            if progress.player_faction == faction && progress.enemy_faction == enemy {
                if stage > progress.highest_stage {
                    progress.highest_stage = stage;
                }
                if mission > progress.highest_mission {
                    progress.highest_mission = mission;
                }
                found = true;
                break;
            }
        }

        if !found {
            self.stage_progress.push(FactionProgress {
                player_faction: faction.to_string(),
                enemy_faction: enemy.to_string(),
                highest_stage: stage,
                highest_mission: mission,
            });
        }
    }

    /// Get highest stage for faction pair
    pub fn get_highest_stage(&self, faction: &str, enemy: &str) -> u32 {
        for progress in &self.stage_progress {
            if progress.player_faction == faction && progress.enemy_faction == enemy {
                return progress.highest_stage;
            }
        }
        0
    }

    /// Get high score for faction pair
    pub fn get_high_score(&self, faction: &str, enemy: &str) -> u64 {
        for hs in &self.high_scores {
            if hs.player_faction == faction && hs.enemy_faction == enemy {
                return hs.score;
            }
        }
        0
    }

    /// Record high score
    pub fn record_score(&mut self, faction: &str, enemy: &str, score: u64, stage: u32) {
        // Find or create entry
        let mut found = false;
        for hs in &mut self.high_scores {
            if hs.player_faction == faction && hs.enemy_faction == enemy {
                if score > hs.score {
                    hs.score = score;
                    hs.stage = stage;
                }
                found = true;
                break;
            }
        }

        if !found {
            self.high_scores.push(HighScore {
                player_faction: faction.to_string(),
                enemy_faction: enemy.to_string(),
                score,
                stage,
            });
        }
    }

    /// Record a leaderboard entry. Keeps only the top 10 scores (descending).
    /// Returns the position (0-based) if the entry made the leaderboard, or None.
    pub fn record_leaderboard_entry(
        &mut self,
        faction: &str,
        enemy: &str,
        score: u64,
        stage: u32,
        difficulty: &str,
        ship_name: &str,
    ) -> Option<usize> {
        let session = self.next_session();
        let entry = LeaderboardEntry {
            faction: faction.to_string(),
            enemy: enemy.to_string(),
            score,
            stage,
            difficulty: difficulty.to_string(),
            ship_name: ship_name.to_string(),
            session,
        };
        self.leaderboard.push(entry);
        self.leaderboard.sort_by_key(|e| std::cmp::Reverse(e.score));
        self.leaderboard.truncate(10);

        // Find position of this session entry (it has a unique session id)
        self.leaderboard.iter().position(|e| e.session == session)
    }

    /// Get the next session counter value
    fn next_session(&self) -> u32 {
        self.leaderboard
            .iter()
            .map(|e| e.session)
            .max()
            .unwrap_or(0)
            + 1
    }

    /// Add credits
    pub fn add_credits(&mut self, amount: u64) {
        self.lifetime_credits += amount;
    }

    /// Check if an achievement is unlocked
    pub fn has_achievement(&self, achievement: super::Achievement) -> bool {
        self.achievements.contains(&achievement)
    }

    /// Get achievement progress as (unlocked, total)
    pub fn achievement_progress(&self) -> (usize, usize) {
        let total = super::Achievement::all()
            .iter()
            .filter(|a| !a.is_hidden())
            .count();
        let unlocked = self.achievements.iter().filter(|a| !a.is_hidden()).count();
        (unlocked, total)
    }

    /// Update lifetime stats after a run
    pub fn update_lifetime_stats(
        &mut self,
        kills: u32,
        souls: u32,
        bosses: u32,
        max_combo: u32,
        score: u64,
        mission_completed: bool,
    ) {
        self.lifetime_stats.total_kills += kills as u64;
        self.lifetime_stats.total_souls += souls as u64;
        self.lifetime_stats.bosses_defeated += bosses;
        self.lifetime_stats.games_played += 1;
        if mission_completed {
            self.lifetime_stats.missions_completed += 1;
        }
        if max_combo > self.lifetime_stats.highest_combo {
            self.lifetime_stats.highest_combo = max_combo;
        }
        if score > self.lifetime_stats.highest_score {
            self.lifetime_stats.highest_score = score;
        }
    }

    /// Add skill points
    pub fn add_skill_points(&mut self, amount: u32) {
        self.skill_points += amount;
    }

    /// Check if an upgrade is purchased
    pub fn has_upgrade(&self, upgrade: super::Upgrade) -> bool {
        self.purchased_upgrades.contains(&upgrade)
    }

    /// Check if an upgrade can be purchased
    pub fn can_purchase_upgrade(&self, upgrade: super::Upgrade) -> bool {
        // Already purchased?
        if self.has_upgrade(upgrade) {
            return false;
        }
        // Have enough SP?
        if self.skill_points < upgrade.cost() {
            return false;
        }
        // Prerequisites met?
        if let Some(req) = upgrade.requires() {
            if !self.has_upgrade(req) {
                return false;
            }
        }
        true
    }

    /// Purchase an upgrade (returns true if successful)
    pub fn purchase_upgrade(&mut self, upgrade: super::Upgrade) -> bool {
        if !self.can_purchase_upgrade(upgrade) {
            return false;
        }
        self.skill_points -= upgrade.cost();
        self.purchased_upgrades.insert(upgrade);
        true
    }

    /// Get total stat bonuses from purchased upgrades
    pub fn get_upgrade_bonuses(&self) -> UpgradeBonuses {
        let mut bonuses = UpgradeBonuses::default();

        for upgrade in &self.purchased_upgrades {
            match upgrade {
                super::Upgrade::ShieldBoost1 => bonuses.shield_bonus += 20.0,
                super::Upgrade::ShieldBoost2 => bonuses.shield_bonus += 40.0,
                super::Upgrade::ArmorPlate1 => bonuses.armor_bonus += 30.0,
                super::Upgrade::ArmorPlate2 => bonuses.armor_bonus += 60.0,
                super::Upgrade::Gyrostabilizer1 => bonuses.fire_rate_mult *= 1.1,
                super::Upgrade::Gyrostabilizer2 => bonuses.fire_rate_mult *= 1.2,
                super::Upgrade::ExpandedRocketBay => bonuses.rocket_capacity += 5,
                super::Upgrade::DamageAmplifier1 => bonuses.damage_mult *= 1.1,
                super::Upgrade::DamageAmplifier2 => bonuses.damage_mult *= 1.2,
                super::Upgrade::Afterburner => bonuses.speed_mult *= 1.1,
                super::Upgrade::CapacitorBattery => bonuses.capacitor_bonus += 25.0,
                super::Upgrade::ShieldBooster => bonuses.shield_regen_mult *= 1.5,
            }
        }

        bonuses
    }
}

/// Stat bonuses from purchased upgrades
#[derive(Debug, Clone)]
pub struct UpgradeBonuses {
    pub shield_bonus: f32,
    pub armor_bonus: f32,
    pub capacitor_bonus: f32,
    pub fire_rate_mult: f32,
    pub damage_mult: f32,
    pub speed_mult: f32,
    pub shield_regen_mult: f32,
    pub rocket_capacity: i32,
}

impl Default for UpgradeBonuses {
    /// Baseline bonuses with multipliers at 1.0 (no upgrades purchased yet).
    /// A zeroed `speed_mult` multiplies `max_speed` and `acceleration` to 0 and
    /// breaks player_movement, which is why multipliers must not default to 0.
    fn default() -> Self {
        Self {
            shield_bonus: 0.0,
            armor_bonus: 0.0,
            capacitor_bonus: 0.0,
            fire_rate_mult: 1.0,
            damage_mult: 1.0,
            speed_mult: 1.0,
            shield_regen_mult: 1.0,
            rocket_capacity: 0,
        }
    }
}

impl UpgradeBonuses {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Load save data on startup
fn load_save_data(mut commands: Commands) {
    let save = SaveData::load();
    commands.insert_resource(save);
}

/// Auto-save when data changes
fn auto_save(save: Res<SaveData>) {
    save.save();
}

/// Apply saved settings to runtime resources (runs after all plugins init)
fn apply_saved_settings(
    save: Res<SaveData>,
    mut sound: ResMut<SoundSettings>,
    mut shake: ResMut<ScreenShake>,
    mut rumble: ResMut<RumbleSettings>,
    mut keybindings: ResMut<KeyBindings>,
) {
    let settings = &save.settings;

    // Apply audio settings
    sound.master_volume = settings.master_volume;
    sound.sfx_volume = settings.sfx_volume;
    sound.music_volume = settings.music_volume;

    // Apply screen shake intensity
    shake.multiplier = settings.screen_shake_intensity;

    // Apply rumble intensity
    rumble.intensity = settings.rumble_intensity;

    // Apply the saved binding layout. `KeyBindingsPlugin` already
    // registered `KeyBindings` with defaults; this overwrites them
    // with the saved values. If the field is missing in the save blob
    // (legacy pre-keybindings save), serde filled it with the
    // hand-written `Default::default()` which returns
    // `KeyBindings::defaults()` — the canonical layout.
    *keybindings = save.keybindings.clone();

    info!(
        "Applied saved settings: master={:.0}%, sfx={:.0}%, music={:.0}%, shake={:.0}%, rumble={:.0}%, keybindings=loaded",
        settings.master_volume * 100.0,
        settings.sfx_volume * 100.0,
        settings.music_volume * 100.0,
        settings.screen_shake_intensity * 100.0,
        settings.rumble_intensity * 100.0
    );
}

/// Sync runtime settings changes back to SaveData
/// Only runs when SoundSettings, ScreenShake, RumbleSettings, or
/// KeyBindings resources change.
fn sync_settings_to_save(
    sound: Res<SoundSettings>,
    shake: Res<ScreenShake>,
    rumble: Res<RumbleSettings>,
    keybindings: Res<KeyBindings>,
    mut save: ResMut<SaveData>,
) {
    // Only process if any resource changed this frame
    if !sound.is_changed()
        && !shake.is_changed()
        && !rumble.is_changed()
        && !keybindings.is_changed()
    {
        return;
    }

    // Check if settings actually differ from saved values
    let settings = &save.settings;
    let sound_changed = (settings.master_volume - sound.master_volume).abs() > 0.001
        || (settings.sfx_volume - sound.sfx_volume).abs() > 0.001
        || (settings.music_volume - sound.music_volume).abs() > 0.001;
    let shake_changed = (settings.screen_shake_intensity - shake.multiplier).abs() > 0.001;
    let rumble_changed = (settings.rumble_intensity - rumble.intensity).abs() > 0.001;
    let keybindings_changed = *keybindings != save.keybindings;

    if !sound_changed && !shake_changed && !rumble_changed && !keybindings_changed {
        return;
    }

    // Update settings
    let settings = &mut save.settings;
    if sound_changed {
        settings.master_volume = sound.master_volume;
        settings.sfx_volume = sound.sfx_volume;
        settings.music_volume = sound.music_volume;
    }
    if shake_changed {
        settings.screen_shake_intensity = shake.multiplier;
    }
    if rumble_changed {
        settings.rumble_intensity = rumble.intensity;
    }

    // The `&mut save.settings` reborrow above prevents touching other
    // fields of `save`, so we read the final values for logging first;
    // the borrow ends after the last use below, then we write
    // `save.keybindings`.
    let post_settings = (
        settings.master_volume,
        settings.sfx_volume,
        settings.music_volume,
        settings.screen_shake_intensity,
        settings.rumble_intensity,
    );

    if keybindings_changed {
        save.keybindings = keybindings.clone();
    }

    info!(
        "Settings synced to save: master={:.0}%, sfx={:.0}%, music={:.0}%, shake={:.0}%, rumble={:.0}%, keybindings={}",
        post_settings.0 * 100.0,
        post_settings.1 * 100.0,
        post_settings.2 * 100.0,
        post_settings.3 * 100.0,
        post_settings.4 * 100.0,
        if keybindings_changed { "synced" } else { "unchanged" }
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Ship Unlock Tests ====================

    #[test]
    fn ship_unlock_stage_zero_always_unlocked() {
        let save = SaveData::default();
        // unlock_stage 0 = always available
        assert!(save.is_ship_unlocked(12345, 0, "Minmatar", "Amarr"));
    }

    #[test]
    fn ship_unlock_explicit_unlock() {
        let mut save = SaveData::default();
        save.unlock_ship(587); // Wolf type_id

        // Should be unlocked regardless of stage requirement
        assert!(save.is_ship_unlocked(587, 5, "Minmatar", "Amarr"));
    }

    #[test]
    fn ship_unlock_via_stage_progress() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 5, 1);

        // Ship requiring stage 5 should be unlocked
        assert!(save.is_ship_unlocked(587, 5, "Minmatar", "Amarr"));

        // Ship requiring stage 6 should NOT be unlocked
        assert!(!save.is_ship_unlocked(588, 6, "Minmatar", "Amarr"));
    }

    #[test]
    fn ship_unlock_wrong_faction_not_counted() {
        let mut save = SaveData::default();
        save.complete_stage("Caldari", "Gallente", 10, 1);

        // Progress in Caldari vs Gallente doesn't unlock Minmatar ships
        assert!(!save.is_ship_unlocked(587, 5, "Minmatar", "Amarr"));
    }

    // ==================== Stage Progress Tests ====================

    #[test]
    fn complete_stage_creates_entry() {
        let mut save = SaveData::default();
        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 0);

        save.complete_stage("Minmatar", "Amarr", 3, 2);
        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 3);
    }

    #[test]
    fn complete_stage_updates_highest() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 3, 1);
        save.complete_stage("Minmatar", "Amarr", 5, 2);

        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 5);
    }

    #[test]
    fn complete_stage_does_not_decrease() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 5, 1);
        save.complete_stage("Minmatar", "Amarr", 3, 1); // Lower stage

        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 5);
    }

    #[test]
    fn complete_stage_multiple_factions_independent() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 5, 1);
        save.complete_stage("Caldari", "Gallente", 3, 1);

        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 5);
        assert_eq!(save.get_highest_stage("Caldari", "Gallente"), 3);
        assert_eq!(save.get_highest_stage("Gallente", "Caldari"), 0);
    }

    // ==================== High Score Tests ====================

    #[test]
    fn record_score_creates_entry() {
        let mut save = SaveData::default();
        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 0);

        save.record_score("Minmatar", "Amarr", 50000, 5);
        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 50000);
    }

    #[test]
    fn record_score_only_updates_on_beat() {
        let mut save = SaveData::default();
        save.record_score("Minmatar", "Amarr", 50000, 5);
        save.record_score("Minmatar", "Amarr", 30000, 3); // Lower score

        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 50000);
    }

    #[test]
    fn record_score_updates_on_new_high() {
        let mut save = SaveData::default();
        save.record_score("Minmatar", "Amarr", 50000, 5);
        save.record_score("Minmatar", "Amarr", 75000, 7); // Higher score

        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 75000);
    }

    #[test]
    fn record_score_multiple_factions_independent() {
        let mut save = SaveData::default();
        save.record_score("Minmatar", "Amarr", 50000, 5);
        save.record_score("Caldari", "Gallente", 100000, 10);

        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 50000);
        assert_eq!(save.get_high_score("Caldari", "Gallente"), 100000);
    }

    // ==================== Credits Tests ====================

    #[test]
    fn add_credits_accumulates() {
        let mut save = SaveData::default();
        assert_eq!(save.lifetime_credits, 0);

        save.add_credits(1000);
        assert_eq!(save.lifetime_credits, 1000);

        save.add_credits(500);
        assert_eq!(save.lifetime_credits, 1500);
    }

    // ==================== Settings Tests ====================

    #[test]
    fn default_settings() {
        let settings = GameSettings::default();
        assert_eq!(settings.master_volume, 0.7);
        assert_eq!(settings.sfx_volume, 0.8);
        assert_eq!(settings.music_volume, 0.5);
        assert_eq!(settings.screen_shake_intensity, 1.0);
        assert_eq!(settings.rumble_intensity, 1.0);
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn save_data_serialization_roundtrip() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 5, 3);
        save.record_score("Minmatar", "Amarr", 50000, 5);
        save.unlock_ship(587);
        save.add_credits(10000);

        // Serialize
        let json = serde_json::to_string(&save).expect("serialize");

        // Deserialize
        let loaded: SaveData = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(loaded.get_highest_stage("Minmatar", "Amarr"), 5);
        assert_eq!(loaded.get_high_score("Minmatar", "Amarr"), 50000);
        assert!(loaded.unlocked_ships.contains(&587));
        assert_eq!(loaded.lifetime_credits, 10000);
    }

    // ==================== Leaderboard Tests ====================

    #[test]
    fn leaderboard_insert_and_retrieve() {
        let mut save = SaveData::default();
        let pos = save.record_leaderboard_entry("Minmatar", "Amarr", 50000, 5, "Normal", "Wolf");
        assert_eq!(pos, Some(0));
        assert_eq!(save.leaderboard.len(), 1);
        assert_eq!(save.leaderboard[0].score, 50000);
        assert_eq!(save.leaderboard[0].ship_name, "Wolf");
    }

    #[test]
    fn leaderboard_sorted_descending() {
        let mut save = SaveData::default();
        save.record_leaderboard_entry("Minmatar", "Amarr", 10000, 2, "Normal", "Rifter");
        save.record_leaderboard_entry("Minmatar", "Amarr", 50000, 5, "Hard", "Wolf");
        save.record_leaderboard_entry("Caldari", "Gallente", 30000, 4, "Normal", "Merlin");

        assert_eq!(save.leaderboard.len(), 3);
        assert_eq!(save.leaderboard[0].score, 50000);
        assert_eq!(save.leaderboard[1].score, 30000);
        assert_eq!(save.leaderboard[2].score, 10000);
    }

    #[test]
    fn leaderboard_caps_at_ten() {
        let mut save = SaveData::default();
        // Insert 12 entries with ascending scores
        for i in 0..12 {
            save.record_leaderboard_entry(
                "Minmatar",
                "Amarr",
                (i + 1) * 1000,
                i as u32,
                "Normal",
                "Rifter",
            );
        }
        assert_eq!(save.leaderboard.len(), 10);
        // Highest score should be first
        assert_eq!(save.leaderboard[0].score, 12000);
        // Lowest kept score should be 3000 (entries 1000 and 2000 dropped)
        assert_eq!(save.leaderboard[9].score, 3000);
    }

    #[test]
    fn leaderboard_low_score_returns_none() {
        let mut save = SaveData::default();
        // Fill with 10 high scores
        for i in 0..10 {
            save.record_leaderboard_entry("Minmatar", "Amarr", (i + 10) * 10000, 5, "Hard", "Wolf");
        }
        assert_eq!(save.leaderboard.len(), 10);
        let lowest = save.leaderboard[9].score;

        // Insert a score lower than the lowest on the board
        let pos =
            save.record_leaderboard_entry("Minmatar", "Amarr", lowest - 1, 1, "Easy", "Rifter");
        assert_eq!(pos, None);
        assert_eq!(save.leaderboard.len(), 10);
    }

    #[test]
    fn leaderboard_serialization_roundtrip() {
        let mut save = SaveData::default();
        save.record_leaderboard_entry("Minmatar", "Amarr", 50000, 5, "Normal", "Wolf");
        save.record_leaderboard_entry("Caldari", "Gallente", 30000, 3, "Hard", "Merlin");

        let json = serde_json::to_string(&save).expect("serialize");
        let loaded: SaveData = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(loaded.leaderboard.len(), 2);
        assert_eq!(loaded.leaderboard[0].score, 50000);
        assert_eq!(loaded.leaderboard[1].score, 30000);
    }
}
