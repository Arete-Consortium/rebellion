//! Ability visual effects

use bevy::prelude::*;
use crate::core::*;
use crate::systems::ability::{AbilityActivatedEvent, AbilityType};
use super::screen_effects::ScreenFlash;

/// Visual effect type for abilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityEffectType {
    /// Speed boost - trailing lines
    SpeedBoost,
    /// Shield activation - expanding ring
    ShieldBubble,
    /// Armor hardening - metallic particles
    ArmorPlating,
    /// Weapon charge - energy particles
    WeaponCharge,
    /// Drone deployment - swarm particles
    DroneSwarm,
    /// Disruption - wave pulse
    Disruption,
    /// Damage boost - red aura
    DamageAura,
}

/// Component for ability visual effect particles
#[derive(Component)]
pub struct AbilityEffectParticle {
    pub effect_type: AbilityEffectType,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    /// For effects that orbit or rotate
    pub angle: f32,
    pub angular_velocity: f32,
}

/// Component for ability aura effect (attached to player)
#[derive(Component)]
pub struct AbilityAura {
    pub ability_type: AbilityType,
    pub timer: f32,
}

/// Spawn visual effects when ability is activated
pub fn spawn_ability_effects(
    mut commands: Commands,
    mut ability_events: EventReader<AbilityActivatedEvent>,
    player_query: Query<&Transform, With<crate::entities::Player>>,
    mut screen_flash: ResMut<ScreenFlash>,
) {
    for event in ability_events.read() {
        let Ok(player_transform) = player_query.get(event.player_entity) else {
            continue;
        };
        let pos = player_transform.translation.truncate();

        match event.ability_type {
            AbilityType::Overdrive | AbilityType::Afterburner => {
                // Speed boost - orange flash + trailing particles
                screen_flash.colored(Color::srgba(1.0, 0.6, 0.2, 0.8), 0.4);
                spawn_speed_boost_effect(&mut commands, pos);
            }
            AbilityType::ShieldBoost => {
                // Shield - blue expanding ring
                screen_flash.colored(Color::srgba(0.3, 0.6, 1.0, 0.8), 0.5);
                spawn_shield_effect(&mut commands, pos);
            }
            AbilityType::ArmorHardener | AbilityType::ArmorRepair => {
                // Armor - gold metallic flash
                screen_flash.colored(Color::srgba(1.0, 0.85, 0.3, 0.8), 0.4);
                spawn_armor_effect(&mut commands, pos);
            }
            AbilityType::RocketBarrage | AbilityType::Salvo | AbilityType::Scorch => {
                // Weapon - red charge particles
                screen_flash.colored(Color::srgba(1.0, 0.3, 0.2, 0.8), 0.3);
                spawn_weapon_charge_effect(&mut commands, pos);
            }
            AbilityType::DeployDrone | AbilityType::DroneBay => {
                // Drone - green swarm particles
                screen_flash.colored(Color::srgba(0.3, 1.0, 0.5, 0.8), 0.4);
                spawn_drone_effect(&mut commands, pos);
            }
            AbilityType::WarpDisruptor => {
                // Disruption - purple wave
                screen_flash.colored(Color::srgba(0.7, 0.3, 1.0, 0.8), 0.5);
                spawn_disruption_effect(&mut commands, pos);
            }
            AbilityType::CloseRange => {
                // Damage boost - red aura
                screen_flash.colored(Color::srgba(1.0, 0.2, 0.2, 0.8), 0.5);
                spawn_damage_aura_effect(&mut commands, pos);
            }
            AbilityType::None => {}
        }
    }
}

/// Spawn speed boost trailing particles
fn spawn_speed_boost_effect(commands: &mut Commands, position: Vec2) {
    let mut rng = fastrand::Rng::new();

    for i in 0..12 {
        let angle = (i as f32 / 12.0) * std::f32::consts::TAU + rng.f32() * 0.3;
        let speed = 80.0 + rng.f32() * 60.0;
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        commands.spawn((
            AbilityEffectParticle {
                effect_type: AbilityEffectType::SpeedBoost,
                velocity,
                lifetime: 0.4,
                max_lifetime: 0.4,
                angle: 0.0,
                angular_velocity: 0.0,
            },
            Sprite {
                color: Color::srgba(1.0, 0.6, 0.2, 0.9),
                custom_size: Some(Vec2::new(8.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.0)
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

/// Spawn shield bubble expanding ring
fn spawn_shield_effect(commands: &mut Commands, position: Vec2) {
    let mut rng = fastrand::Rng::new();

    // Expanding ring particles
    for i in 0..20 {
        let angle = (i as f32 / 20.0) * std::f32::consts::TAU;
        let speed = 120.0 + rng.f32() * 40.0;
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        commands.spawn((
            AbilityEffectParticle {
                effect_type: AbilityEffectType::ShieldBubble,
                velocity,
                lifetime: 0.5,
                max_lifetime: 0.5,
                angle,
                angular_velocity: 0.0,
            },
            Sprite {
                color: Color::srgba(0.4, 0.7, 1.0, 0.9),
                custom_size: Some(Vec2::splat(6.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.0),
        ));
    }
}

/// Spawn armor metallic particles
fn spawn_armor_effect(commands: &mut Commands, position: Vec2) {
    let mut rng = fastrand::Rng::new();

    for _ in 0..15 {
        let angle = rng.f32() * std::f32::consts::TAU;
        let dist = 10.0 + rng.f32() * 20.0;
        let offset = Vec2::new(angle.cos(), angle.sin()) * dist;

        commands.spawn((
            AbilityEffectParticle {
                effect_type: AbilityEffectType::ArmorPlating,
                velocity: Vec2::new(0.0, 30.0 + rng.f32() * 20.0),
                lifetime: 0.6,
                max_lifetime: 0.6,
                angle: rng.f32() * std::f32::consts::TAU,
                angular_velocity: (rng.f32() - 0.5) * 8.0,
            },
            Sprite {
                color: Color::srgba(1.0, 0.85, 0.3, 0.9),
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(
                position.x + offset.x,
                position.y + offset.y,
                LAYER_EFFECTS + 1.0,
            ),
        ));
    }
}

/// Spawn weapon charge energy particles
fn spawn_weapon_charge_effect(commands: &mut Commands, position: Vec2) {
    let mut rng = fastrand::Rng::new();

    // Inward-converging particles
    for _ in 0..16 {
        let angle = rng.f32() * std::f32::consts::TAU;
        let dist = 40.0 + rng.f32() * 20.0;
        let start_pos = position + Vec2::new(angle.cos(), angle.sin()) * dist;

        // Velocity pointing toward player
        let velocity = (position - start_pos).normalize() * (100.0 + rng.f32() * 50.0);

        commands.spawn((
            AbilityEffectParticle {
                effect_type: AbilityEffectType::WeaponCharge,
                velocity,
                lifetime: 0.3,
                max_lifetime: 0.3,
                angle: 0.0,
                angular_velocity: 0.0,
            },
            Sprite {
                color: Color::srgba(1.0, 0.4, 0.3, 0.9),
                custom_size: Some(Vec2::splat(4.0)),
                ..default()
            },
            Transform::from_xyz(start_pos.x, start_pos.y, LAYER_EFFECTS + 1.0),
        ));
    }
}

/// Spawn drone swarm particles
fn spawn_drone_effect(commands: &mut Commands, position: Vec2) {
    let mut rng = fastrand::Rng::new();

    for _ in 0..10 {
        let angle = rng.f32() * std::f32::consts::TAU;
        let speed = 60.0 + rng.f32() * 80.0;
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        commands.spawn((
            AbilityEffectParticle {
                effect_type: AbilityEffectType::DroneSwarm,
                velocity,
                lifetime: 0.5,
                max_lifetime: 0.5,
                angle,
                angular_velocity: 3.0 + rng.f32() * 4.0,
            },
            Sprite {
                color: Color::srgba(0.4, 1.0, 0.6, 0.9),
                custom_size: Some(Vec2::new(6.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.0),
        ));
    }
}

/// Spawn disruption wave effect
fn spawn_disruption_effect(commands: &mut Commands, position: Vec2) {
    let mut rng = fastrand::Rng::new();

    // Expanding wave ring
    for i in 0..24 {
        let angle = (i as f32 / 24.0) * std::f32::consts::TAU;
        let speed = 180.0 + rng.f32() * 30.0;
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        commands.spawn((
            AbilityEffectParticle {
                effect_type: AbilityEffectType::Disruption,
                velocity,
                lifetime: 0.6,
                max_lifetime: 0.6,
                angle,
                angular_velocity: 0.0,
            },
            Sprite {
                color: Color::srgba(0.7, 0.3, 1.0, 0.8),
                custom_size: Some(Vec2::new(10.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.0)
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

/// Spawn damage aura red particles
fn spawn_damage_aura_effect(commands: &mut Commands, position: Vec2) {
    let mut rng = fastrand::Rng::new();

    for _ in 0..18 {
        let angle = rng.f32() * std::f32::consts::TAU;
        let speed = 40.0 + rng.f32() * 60.0;
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        commands.spawn((
            AbilityEffectParticle {
                effect_type: AbilityEffectType::DamageAura,
                velocity,
                lifetime: 0.5,
                max_lifetime: 0.5,
                angle: 0.0,
                angular_velocity: 0.0,
            },
            Sprite {
                color: Color::srgba(1.0, 0.2, 0.2, 0.9),
                custom_size: Some(Vec2::splat(5.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS + 1.0),
        ));
    }
}

/// Update ability effect particles
pub fn update_ability_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut AbilityEffectParticle,
        &mut Sprite,
    )>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        // Move
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Rotate if applicable
        if particle.angular_velocity != 0.0 {
            particle.angle += particle.angular_velocity * dt;
            transform.rotation = Quat::from_rotation_z(particle.angle);
        }

        // Slow down
        particle.velocity *= 1.0 - 4.0 * dt;

        // Update lifetime
        particle.lifetime -= dt;
        let progress = particle.lifetime / particle.max_lifetime;

        // Fade out
        let alpha = progress * 0.9;
        sprite.color = sprite.color.with_alpha(alpha);

        // Shrink
        if let Some(size) = sprite.custom_size {
            sprite.custom_size = Some(size * (1.0 - 1.5 * dt));
        }

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
