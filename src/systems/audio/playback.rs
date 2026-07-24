//! Audio playback systems
//!
//! All `play_*` systems that respond to game events and spawn audio.

use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;

use crate::core::{CampaignBossSpawned, WaveCompleteEvent, *};
use crate::systems::ability::{AbilityActivatedEvent, AbilityType};

use super::{SoundAssets, SoundSettings, WarningState};

/// Play weapon firing sounds with subtle variation
pub fn play_weapon_sounds(
    mut commands: Commands,
    mut fire_events: EventReader<PlayerFireEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        fire_events.clear();
        return;
    }

    for event in fire_events.read() {
        let sound = match event.weapon_type {
            WeaponType::Autocannon | WeaponType::Artillery => sounds.autocannon.clone(),
            WeaponType::Laser | WeaponType::Railgun => sounds.laser.clone(),
            WeaponType::MissileLauncher => sounds.missile.clone(),
            WeaponType::Drone => sounds.laser.clone(), // Drones use laser-like sound
            WeaponType::Disintegrator => sounds.laser.clone(), // Triglavian beam sound
            WeaponType::Vorton => sounds.laser.clone(), // EDENCOM arc sound
        };

        if let Some(source) = sound {
            // Add subtle volume and speed variation to avoid repetition
            let volume_var = 0.9 + fastrand::f32() * 0.2; // 0.9 - 1.1
            let speed_var = 0.95 + fastrand::f32() * 0.1; // 0.95 - 1.05 pitch variation

            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(
                        settings.sfx_volume * settings.master_volume * 0.5 * volume_var,
                    ),
                    speed: speed_var,
                    ..default()
                },
            ));
        }
    }
}

/// Play explosion sounds on enemy destruction with size-based variation
pub fn play_explosion_sounds(
    mut commands: Commands,
    mut destroy_events: EventReader<EnemyDestroyedEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        destroy_events.clear();
        return;
    }

    for event in destroy_events.read() {
        // Select explosion sound based on enemy type/size
        let (sound, base_volume, base_pitch) = if event.was_boss {
            // Boss = large, deep explosion
            (sounds.explosion_large.clone(), 0.8, 0.8)
        } else {
            // Use score_value as proxy for ship size
            match event.score_value {
                0..=50 => (sounds.explosion_small.clone(), 0.5, 1.1), // Frigates
                51..=150 => (sounds.explosion_small.clone(), 0.6, 1.0), // Destroyers
                151..=300 => (sounds.explosion_medium.clone(), 0.65, 0.95), // Cruisers
                _ => (sounds.explosion_medium.clone(), 0.7, 0.9),     // Battlecruisers+
            }
        };

        if let Some(source) = sound {
            // Add variation
            let volume_var = 0.9 + fastrand::f32() * 0.2;
            let pitch_var = 0.95 + fastrand::f32() * 0.1;

            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(
                        settings.sfx_volume * settings.master_volume * base_volume * volume_var,
                    ),
                    speed: base_pitch * pitch_var,
                    ..default()
                },
            ));
        }
    }
}

/// Play pickup sounds with different sounds for different powerup types
pub fn play_pickup_sounds(
    mut commands: Commands,
    mut pickup_events: EventReader<CollectiblePickedUpEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        pickup_events.clear();
        return;
    }

    for event in pickup_events.read() {
        // Choose sound based on collectible type
        let sound = match event.collectible_type {
            CollectibleType::Overdrive => sounds.powerup_overdrive.clone(),
            CollectibleType::DamageBoost => sounds.powerup_damage.clone(),
            CollectibleType::Invulnerability => sounds.powerup_invuln.clone(),
            CollectibleType::ShieldBoost
            | CollectibleType::ArmorRepair
            | CollectibleType::HullRepair => sounds.powerup_health.clone(),
            _ => sounds.pickup.clone(), // Credits, souls, etc use generic pickup
        };

        if let Some(source) = sound.or(sounds.pickup.clone()) {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.7),
                    ..default()
                },
            ));
        }
    }
}

/// Play damage sounds when player is hit with intensity variation
pub fn play_damage_sounds(
    mut commands: Commands,
    mut damage_events: EventReader<PlayerDamagedEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        damage_events.clear();
        return;
    }

    for event in damage_events.read() {
        let sound = match event.damage_type {
            DamageType::EM => sounds.shield_hit.clone(),
            DamageType::Thermal | DamageType::Kinetic => sounds.armor_hit.clone(),
            DamageType::Explosive => sounds.hull_hit.clone(),
        };

        if let Some(source) = sound {
            // Scale volume and pitch based on damage amount (heavier hits = louder, deeper)
            let damage_scale = (event.damage / 50.0).clamp(0.5, 2.0);
            let volume = 0.6 + 0.2 * damage_scale.min(1.5);
            let pitch = 1.1 - 0.15 * damage_scale.min(1.5); // Bigger hits = deeper

            // Add subtle variation
            let volume_var = 0.95 + fastrand::f32() * 0.1;
            let pitch_var = 0.97 + fastrand::f32() * 0.06;

            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(
                        settings.sfx_volume * settings.master_volume * volume * volume_var,
                    ),
                    speed: pitch * pitch_var,
                    ..default()
                },
            ));
        }
    }
}

/// Play warning sounds when health drops below 20%
pub fn play_health_warnings(
    mut commands: Commands,
    player_query: Query<&crate::entities::ShipStats, With<crate::entities::Player>>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
    mut warning_state: ResMut<WarningState>,
    time: Res<Time>,
) {
    if !settings.enabled {
        return;
    }

    // Cooldown between warnings
    warning_state.warning_cooldown -= time.delta_secs();

    let Ok(stats) = player_query.get_single() else {
        return;
    };

    let shield_pct = stats.shield / stats.max_shield;
    let armor_pct = stats.armor / stats.max_armor;
    let hull_pct = stats.hull / stats.max_hull;

    const WARNING_THRESHOLD: f32 = 0.20;

    // Shield warning
    if shield_pct <= WARNING_THRESHOLD && shield_pct > 0.0 {
        if !warning_state.shield_warned && warning_state.warning_cooldown <= 0.0 {
            if let Some(source) = sounds.shield_warning.clone() {
                commands.spawn((
                    AudioPlayer(source),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.9),
                        ..default()
                    },
                ));
                warning_state.shield_warned = true;
                warning_state.warning_cooldown = 3.0; // 3 second cooldown between warnings
            }
        }
    } else if shield_pct > WARNING_THRESHOLD {
        warning_state.shield_warned = false;
    }

    // Armor warning (more urgent)
    if armor_pct <= WARNING_THRESHOLD && armor_pct > 0.0 {
        if !warning_state.armor_warned && warning_state.warning_cooldown <= 0.0 {
            if let Some(source) = sounds.armor_warning.clone() {
                commands.spawn((
                    AudioPlayer(source),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.95),
                        ..default()
                    },
                ));
                warning_state.armor_warned = true;
                warning_state.warning_cooldown = 2.5;
            }
        }
    } else if armor_pct > WARNING_THRESHOLD {
        warning_state.armor_warned = false;
    }

    // Hull warning (critical - most urgent)
    if hull_pct <= WARNING_THRESHOLD && hull_pct > 0.0 {
        if !warning_state.hull_warned && warning_state.warning_cooldown <= 0.0 {
            if let Some(source) = sounds.hull_warning.clone() {
                commands.spawn((
                    AudioPlayer(source),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new(settings.sfx_volume * settings.master_volume),
                        ..default()
                    },
                ));
                warning_state.hull_warned = true;
                warning_state.warning_cooldown = 2.0;
            }
        }
    } else if hull_pct > WARNING_THRESHOLD {
        warning_state.hull_warned = false;
    }
}

/// Play ability activation sounds
pub fn play_ability_sounds(
    mut commands: Commands,
    mut ability_events: EventReader<AbilityActivatedEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        ability_events.clear();
        return;
    }

    for event in ability_events.read() {
        let sound = match event.ability_type {
            AbilityType::Overdrive | AbilityType::Afterburner => sounds.ability_speed.clone(),
            AbilityType::ShieldBoost => sounds.ability_shield.clone(),
            AbilityType::ArmorHardener | AbilityType::ArmorRepair => sounds.ability_armor.clone(),
            AbilityType::RocketBarrage | AbilityType::Salvo | AbilityType::Scorch => {
                sounds.ability_weapon.clone()
            }
            AbilityType::DeployDrone | AbilityType::DroneBay => sounds.ability_drone.clone(),
            AbilityType::WarpDisruptor => sounds.ability_debuff.clone(),
            AbilityType::CloseRange => sounds.ability_damage.clone(),
            AbilityType::None => None,
        };

        if let Some(source) = sound {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.85),
                    ..default()
                },
            ));
        }
    }
}

/// Play wave complete sound
pub fn play_wave_complete_sound(
    mut commands: Commands,
    mut wave_events: EventReader<WaveCompleteEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        wave_events.clear();
        return;
    }

    for _event in wave_events.read() {
        if let Some(source) = sounds.wave_complete.clone() {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.8),
                    ..default()
                },
            ));
        }
    }
}

/// Play boss spawn sound
pub fn play_boss_spawn_sound(
    mut commands: Commands,
    mut boss_events: EventReader<CampaignBossSpawned>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        boss_events.clear();
        return;
    }

    for _event in boss_events.read() {
        if let Some(source) = sounds.boss_spawn.clone() {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.9),
                    ..default()
                },
            ));
        }
    }
}
