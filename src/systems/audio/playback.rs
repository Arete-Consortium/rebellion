//! Audio playback systems
//!
//! All `play_*` systems that respond to game events and spawn audio.

use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;

use crate::core::{CampaignBossSpawned, WaveCompleteEvent, *};
use crate::systems::ability::{AbilityActivatedEvent, AbilityType};

use super::{SoundAssets, SoundSettings, WarningState};

#[derive(Component)]
pub struct WeaponVoice;

#[derive(Component)]
pub struct ExplosionVoice {
    pub boss: bool,
}

#[derive(Component)]
pub struct HealthWarningVoice;

/// Play weapon firing sounds with subtle variation
pub fn play_weapon_sounds(
    mut commands: Commands,
    mut fire_events: EventReader<PlayerFireEvent>,
    voices: Query<Entity, With<WeaponVoice>>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        fire_events.clear();
        return;
    }

    let mut remaining = 8usize.saturating_sub(voices.iter().count()).min(2);
    for event in fire_events.read() {
        if remaining == 0 {
            continue;
        }
        let sound = match event.weapon_type {
            WeaponType::Autocannon | WeaponType::Artillery => sounds.autocannon.clone(),
            WeaponType::Laser => sounds.laser.clone(),
            WeaponType::Railgun => sounds.railgun.clone(),
            WeaponType::MissileLauncher => sounds.missile.clone(),
            WeaponType::Drone => sounds.drone.clone(),
            WeaponType::Disintegrator => sounds.laser.clone(), // Triglavian beam sound
            WeaponType::Vorton => sounds.laser.clone(),        // EDENCOM arc sound
        };

        if let Some(source) = sound {
            remaining -= 1;
            // Add subtle volume and speed variation to avoid repetition
            let volume_var = 0.9 + fastrand::f32() * 0.2; // 0.9 - 1.1
            let speed_var = 0.95 + fastrand::f32() * 0.1; // 0.95 - 1.05 pitch variation

            commands.spawn((
                AudioPlayer(source),
                WeaponVoice,
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
    voices: Query<&ExplosionVoice>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        destroy_events.clear();
        return;
    }

    let mut ordinary = 8usize
        .saturating_sub(voices.iter().filter(|v| !v.boss).count())
        .min(3);
    let mut boss_available = !voices.iter().any(|voice| voice.boss);
    for event in destroy_events.read() {
        if (event.was_boss && !boss_available) || (!event.was_boss && ordinary == 0) {
            continue;
        }
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
            if event.was_boss {
                boss_available = false;
            } else {
                ordinary -= 1;
            }
            // Add variation
            let volume_var = 0.9 + fastrand::f32() * 0.2;
            let pitch_var = 0.95 + fastrand::f32() * 0.1;

            commands.spawn((
                AudioPlayer(source),
                ExplosionVoice {
                    boss: event.was_boss,
                },
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
    mut booster_events: EventReader<crate::entities::boosters::BoosterActivatedEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        pickup_events.clear();
        booster_events.clear();
        return;
    }

    for (kind, activated) in pickup_events
        .read()
        .map(|event| (event.collectible_type, false))
        .chain(booster_events.read().map(|event| (event.0.pickup(), true)))
    {
        // Choose sound based on collectible type
        let sound = match kind {
            kind if !activated
                && crate::entities::boosters::BoosterKind::from_pickup(kind).is_some() =>
            {
                sounds.pickup.clone()
            }
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
    active_warnings: Query<Entity, With<HealthWarningVoice>>,
    time: Res<Time>,
) {
    warning_state.warning_cooldown = (warning_state.warning_cooldown - time.delta_secs()).max(0.0);
    let Ok(stats) = player_query.get_single() else {
        *warning_state = WarningState::default();
        return;
    };
    let fractions = [
        stats.shield / stats.max_shield.max(1.0),
        stats.armor / stats.max_armor.max(1.0),
        stats.hull / stats.max_hull.max(1.0),
    ];
    // Hysteresis prevents shield regeneration around 20% from chattering.
    if fractions[0] > 0.25 {
        warning_state.shield_warned = false;
    }
    if fractions[1] > 0.25 {
        warning_state.armor_warned = false;
    }
    if fractions[2] > 0.25 {
        warning_state.hull_warned = false;
    }
    if !settings.enabled || stats.hull <= 0.0 {
        return;
    }
    let Some(layer) = (0..3)
        .rev()
        .find(|&i| fractions[i] > 0.0 && fractions[i] <= 0.20)
    else {
        warning_state.last_priority = 0;
        return;
    };
    let priority = layer as u8 + 1;
    let (warned, source) = match layer {
        0 => (warning_state.shield_warned, sounds.shield_warning.clone()),
        1 => (warning_state.armor_warned, sounds.armor_warning.clone()),
        _ => (warning_state.hull_warned, sounds.hull_warning.clone()),
    };
    if warned || (warning_state.warning_cooldown > 0.0 && priority <= warning_state.last_priority) {
        return;
    }
    if let Some(source) = source {
        // Escalating hull danger interrupts an earlier, less urgent alarm.
        for entity in &active_warnings {
            commands.entity(entity).despawn();
        }
        commands.spawn((
            AudioPlayer(source),
            HealthWarningVoice,
            PlaybackSettings {
                mode: PlaybackMode::Despawn,
                volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.9),
                ..default()
            },
        ));
        match layer {
            0 => warning_state.shield_warned = true,
            1 => warning_state.armor_warned = true,
            _ => warning_state.hull_warned = true,
        }
        warning_state.last_priority = priority;
        warning_state.warning_cooldown = 3.0;
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

/// One audio cue per carrier arrival, including subsequent waves on the same entity.
pub fn play_carrier_warps(
    mut commands: Commands,
    carriers: Query<(Entity, &crate::systems::spawning::EnemyCarrier)>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
    mut heard: Local<std::collections::HashMap<Entity, u32>>,
) {
    heard.retain(|entity, _| carriers.get(*entity).is_ok());
    for (entity, carrier) in &carriers {
        if heard.get(&entity) == Some(&carrier.wave_number) {
            continue;
        }
        heard.insert(entity, carrier.wave_number);
        if settings.enabled {
            if let Some(source) = sounds.carrier_warp.clone() {
                commands.spawn((
                    AudioPlayer(source),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new(settings.master_volume * settings.sfx_volume * 0.65),
                        ..default()
                    },
                ));
            }
        }
    }
}

pub fn play_chapter_boss_alert(
    mut commands: Commands,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if settings.enabled {
        if let Some(source) = sounds.boss_spawn.clone() {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.master_volume * settings.sfx_volume * 0.55),
                    ..default()
                },
            ));
        }
    }
}

#[cfg(test)]
mod playback_tests {
    use super::*;
    use crate::entities::{Player, ShipStats};

    fn audio_app() -> App {
        let mut app = App::new();
        let mut assets = Assets::<AudioSource>::default();
        let shield = assets.add(super::super::generators::generate_shield_warning().unwrap());
        let armor = assets.add(super::super::generators::generate_armor_warning().unwrap());
        let hull = assets.add(super::super::generators::generate_hull_warning().unwrap());
        app.insert_resource(SoundAssets {
            shield_warning: Some(shield.clone()),
            armor_warning: Some(armor.clone()),
            hull_warning: Some(hull.clone()),
            explosion_small: Some(shield),
            explosion_medium: Some(armor),
            explosion_large: Some(hull),
            ..default()
        })
        .insert_resource(assets)
        .init_resource::<SoundSettings>()
        .init_resource::<WarningState>()
        .init_resource::<Time>()
        .add_event::<EnemyDestroyedEvent>();
        app
    }

    #[test]
    fn critical_hull_warning_interrupts_shield_cooldown_and_does_not_repeat() {
        let mut app = audio_app();
        app.add_systems(Update, play_health_warnings);
        let stats = ShipStats::default();
        let player = app
            .world_mut()
            .spawn((
                Player,
                ShipStats {
                    shield: stats.max_shield * 0.1,
                    ..stats
                },
            ))
            .id();
        app.update();
        assert!(app.world().resource::<WarningState>().shield_warned);
        let first = app
            .world_mut()
            .query_filtered::<Entity, With<HealthWarningVoice>>()
            .single(app.world());
        {
            let mut stats = app.world_mut().get_mut::<ShipStats>(player).unwrap();
            stats.armor = stats.max_armor * 0.1;
            stats.hull = stats.max_hull * 0.1;
        }
        app.update();
        assert!(app.world().resource::<WarningState>().hull_warned);
        assert!(app.world().resource::<WarningState>().warning_cooldown > 0.0);
        assert!(app.world().get_entity(first).is_err());
        let (voice, sound) = app
            .world_mut()
            .query_filtered::<(Entity, &AudioPlayer), With<HealthWarningVoice>>()
            .single(app.world());
        assert_eq!(
            Some(&sound.0),
            app.world().resource::<SoundAssets>().hull_warning.as_ref()
        );
        for _ in 0..10 {
            app.update();
        }
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<HealthWarningVoice>>()
                .single(app.world()),
            voice
        );
    }

    #[test]
    fn crowded_explosions_reserve_boss_audio_and_drain_dropped_events() {
        let mut app = audio_app();
        app.add_systems(Update, play_explosion_sounds);
        for _ in 0..8 {
            app.world_mut().spawn(ExplosionVoice { boss: false });
        }
        for i in 0..101 {
            app.world_mut().send_event(EnemyDestroyedEvent {
                enemy: Entity::PLACEHOLDER,
                position: Vec2::ZERO,
                enemy_type: "test".into(),
                score_value: 50,
                was_boss: i == 100,
                liberation_value: 0,
                type_id: 587,
            });
        }
        app.update();
        let mut voices = app.world_mut().query::<&ExplosionVoice>();
        assert_eq!(voices.iter(app.world()).count(), 9);
        assert_eq!(voices.iter(app.world()).filter(|v| v.boss).count(), 1);
        let entities: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<ExplosionVoice>>()
            .iter(app.world())
            .collect();
        for entity in entities {
            app.world_mut().despawn(entity);
        }
        app.update();
        assert_eq!(
            app.world_mut()
                .query::<&ExplosionVoice>()
                .iter(app.world())
                .count(),
            0
        );
    }
}
