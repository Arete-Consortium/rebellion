//! Cleanup systems for despawning all effect entities on state exit

use bevy::prelude::*;
use super::starfield::Star;
use super::explosions::ExplosionParticle;
use super::trails::{EngineParticle, BulletTrailParticle};
use super::screen_effects::ScreenFlashOverlay;
use super::combat_feedback::DamageNumber;
use super::abilities::AbilityEffectParticle;
use super::damage_layers::{ShieldRipple, ArmorSpark, HullFireParticle};
use super::pickups::{PickupFlash, PickupShockwave, PickupParticle};
use super::overlays::{LowHealthVignette, DisintegratorBeamVisual};
use super::buff_visuals::{InvulnShieldBubble, OverdriveSpeedLine, DamageBoostAura};

pub fn cleanup_effects(
    mut commands: Commands,
    stars: Query<Entity, With<Star>>,
    explosion_particles: Query<Entity, With<ExplosionParticle>>,
    engine_particles: Query<Entity, With<EngineParticle>>,
    flash_overlays: Query<Entity, With<ScreenFlashOverlay>>,
    damage_numbers: Query<Entity, With<DamageNumber>>,
    bullet_trail_particles: Query<Entity, With<BulletTrailParticle>>,
    ability_effect_particles: Query<Entity, With<AbilityEffectParticle>>,
) {
    for entity in stars.iter() {
        commands.entity(entity).despawn();
    }
    for entity in explosion_particles.iter() {
        commands.entity(entity).despawn();
    }
    for entity in engine_particles.iter() {
        commands.entity(entity).despawn();
    }
    for entity in flash_overlays.iter() {
        commands.entity(entity).despawn();
    }
    for entity in damage_numbers.iter() {
        commands.entity(entity).despawn();
    }
    for entity in bullet_trail_particles.iter() {
        commands.entity(entity).despawn();
    }
    for entity in ability_effect_particles.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn cleanup_effects_2(
    mut commands: Commands,
    shield_ripples: Query<Entity, With<ShieldRipple>>,
    armor_sparks: Query<Entity, With<ArmorSpark>>,
    hull_fire_particles: Query<Entity, With<HullFireParticle>>,
    pickup_flashes: Query<Entity, With<PickupFlash>>,
    pickup_shockwaves: Query<Entity, With<PickupShockwave>>,
    pickup_particles: Query<Entity, With<PickupParticle>>,
    low_health_vignettes: Query<Entity, With<LowHealthVignette>>,
) {
    for entity in shield_ripples.iter() {
        commands.entity(entity).despawn();
    }
    for entity in armor_sparks.iter() {
        commands.entity(entity).despawn();
    }
    for entity in hull_fire_particles.iter() {
        commands.entity(entity).despawn();
    }
    for entity in pickup_flashes.iter() {
        commands.entity(entity).despawn();
    }
    for entity in pickup_shockwaves.iter() {
        commands.entity(entity).despawn();
    }
    for entity in pickup_particles.iter() {
        commands.entity(entity).despawn();
    }
    for entity in low_health_vignettes.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn cleanup_buff_visuals(
    mut commands: Commands,
    invuln_shields: Query<Entity, With<InvulnShieldBubble>>,
    speed_lines: Query<Entity, With<OverdriveSpeedLine>>,
    damage_auras: Query<Entity, With<DamageBoostAura>>,
    beam_visuals: Query<Entity, With<DisintegratorBeamVisual>>,
) {
    for entity in beam_visuals.iter() {
        commands.entity(entity).despawn();
    }
    for entity in invuln_shields.iter() {
        commands.entity(entity).despawn();
    }
    for entity in speed_lines.iter() {
        commands.entity(entity).despawn();
    }
    for entity in damage_auras.iter() {
        commands.entity(entity).despawn();
    }
}
