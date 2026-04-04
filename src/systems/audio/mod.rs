//! Audio System
//!
//! Procedural sound effects for Rebellion.
//! Uses procedural WAV generation (wav_encoder module).

#![allow(dead_code)]

mod generators;
mod playback;

use bevy::prelude::*;

use crate::core::GameState;

use generators::*;
use playback::*;

/// Audio plugin
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundSettings>()
            .init_resource::<SoundAssets>()
            .init_resource::<WarningState>()
            .add_systems(Startup, generate_sounds)
            .add_systems(
                Update,
                (
                    play_weapon_sounds,
                    play_explosion_sounds,
                    play_pickup_sounds,
                    play_damage_sounds,
                    play_health_warnings,
                    play_wave_complete_sound,
                    play_boss_spawn_sound,
                    play_ability_sounds,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Sound settings
#[derive(Resource)]
pub struct SoundSettings {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub enabled: bool,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            sfx_volume: 0.8,
            music_volume: 0.5,
            enabled: true,
        }
    }
}

/// Pre-generated sound assets
#[derive(Resource, Default)]
pub struct SoundAssets {
    pub autocannon: Option<Handle<AudioSource>>,
    pub laser: Option<Handle<AudioSource>>,
    pub missile: Option<Handle<AudioSource>>,
    pub explosion_small: Option<Handle<AudioSource>>,
    pub explosion_medium: Option<Handle<AudioSource>>,
    pub explosion_large: Option<Handle<AudioSource>>,
    pub pickup: Option<Handle<AudioSource>>,
    pub shield_hit: Option<Handle<AudioSource>>,
    pub armor_hit: Option<Handle<AudioSource>>,
    pub hull_hit: Option<Handle<AudioSource>>,
    // Warning alarms
    pub shield_warning: Option<Handle<AudioSource>>,
    pub armor_warning: Option<Handle<AudioSource>>,
    pub hull_warning: Option<Handle<AudioSource>>,
    // Game events
    pub wave_complete: Option<Handle<AudioSource>>,
    pub boss_spawn: Option<Handle<AudioSource>>,
    // Powerup-specific sounds
    pub powerup_overdrive: Option<Handle<AudioSource>>,
    pub powerup_damage: Option<Handle<AudioSource>>,
    pub powerup_invuln: Option<Handle<AudioSource>>,
    pub powerup_health: Option<Handle<AudioSource>>,
    // Menu sounds
    pub menu_select: Option<Handle<AudioSource>>,
    pub menu_confirm: Option<Handle<AudioSource>>,
    // Ability sounds
    pub ability_speed: Option<Handle<AudioSource>>, // Overdrive, Afterburner
    pub ability_shield: Option<Handle<AudioSource>>, // Shield Boost
    pub ability_armor: Option<Handle<AudioSource>>, // Armor Hardener, Armor Repair
    pub ability_weapon: Option<Handle<AudioSource>>, // Salvo, Rocket Barrage, Scorch
    pub ability_drone: Option<Handle<AudioSource>>, // Deploy Drone, Drone Bay
    pub ability_debuff: Option<Handle<AudioSource>>, // Warp Disruptor
    pub ability_damage: Option<Handle<AudioSource>>, // Close Range
}

/// Tracks when warnings should play (to avoid spamming)
#[derive(Resource)]
pub struct WarningState {
    pub shield_warned: bool,
    pub armor_warned: bool,
    pub hull_warned: bool,
    pub warning_cooldown: f32,
}

impl Default for WarningState {
    fn default() -> Self {
        Self {
            shield_warned: false,
            armor_warned: false,
            hull_warned: false,
            warning_cooldown: 0.0,
        }
    }
}

/// Generate procedural sound effects at startup
fn generate_sounds(
    mut sounds: ResMut<SoundAssets>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    info!("Generating procedural sound effects...");

    // Autocannon - chunky industrial sound
    if let Some(source) = generate_autocannon() {
        sounds.autocannon = Some(audio_sources.add(source));
    }

    // Laser - high-pitched beam
    if let Some(source) = generate_laser() {
        sounds.laser = Some(audio_sources.add(source));
    }

    // Missile launch - whoosh
    if let Some(source) = generate_missile() {
        sounds.missile = Some(audio_sources.add(source));
    }

    // Explosions - various sizes
    if let Some(source) = generate_explosion(0.15, 300.0) {
        sounds.explosion_small = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_explosion(0.25, 200.0) {
        sounds.explosion_medium = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_explosion(0.4, 120.0) {
        sounds.explosion_large = Some(audio_sources.add(source));
    }

    // Pickup - cheerful blip
    if let Some(source) = generate_pickup() {
        sounds.pickup = Some(audio_sources.add(source));
    }

    // Damage sounds
    if let Some(source) = generate_shield_hit() {
        sounds.shield_hit = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_armor_hit() {
        sounds.armor_hit = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_hull_hit() {
        sounds.hull_hit = Some(audio_sources.add(source));
    }

    // Warning alarms (when health drops below 20%)
    if let Some(source) = generate_shield_warning() {
        sounds.shield_warning = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_armor_warning() {
        sounds.armor_warning = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_hull_warning() {
        sounds.hull_warning = Some(audio_sources.add(source));
    }

    // Game event sounds
    if let Some(source) = generate_wave_complete() {
        sounds.wave_complete = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_boss_spawn() {
        sounds.boss_spawn = Some(audio_sources.add(source));
    }

    // Powerup-specific sounds
    if let Some(source) = generate_powerup_overdrive() {
        sounds.powerup_overdrive = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_powerup_damage() {
        sounds.powerup_damage = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_powerup_invuln() {
        sounds.powerup_invuln = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_powerup_health() {
        sounds.powerup_health = Some(audio_sources.add(source));
    }

    // Menu sounds
    if let Some(source) = generate_menu_select() {
        sounds.menu_select = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_menu_confirm() {
        sounds.menu_confirm = Some(audio_sources.add(source));
    }

    // Ability sounds
    if let Some(source) = generate_ability_speed() {
        sounds.ability_speed = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_shield() {
        sounds.ability_shield = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_armor() {
        sounds.ability_armor = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_weapon() {
        sounds.ability_weapon = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_drone() {
        sounds.ability_drone = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_debuff() {
        sounds.ability_debuff = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_damage() {
        sounds.ability_damage = Some(audio_sources.add(source));
    }

    info!("Sound effects generated!");
}
