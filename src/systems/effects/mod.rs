//! Visual Effects System
//!
//! Starfield, explosions, particle effects, screen shake, engine trails.

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod abilities;
pub mod buff_visuals;
pub mod cleanup;
pub mod combat_feedback;
pub mod damage_layers;
pub mod explosions;
pub mod impact_sparks;
pub mod muzzle_flash;
pub mod overlays;
pub mod pickups;
pub mod screen_effects;
pub mod starfield;
pub mod trails;

pub use abilities::*;
pub use buff_visuals::*;
pub use combat_feedback::*;
pub use damage_layers::*;
pub use explosions::*;
pub use impact_sparks::*;
pub use muzzle_flash::*;
pub use overlays::*;
pub use pickups::*;
pub use screen_effects::*;
pub use starfield::*;
pub use trails::*;

use crate::core::GameState;
use bevy::prelude::*;

/// Maximum particles to prevent slowdown during intense combat
pub const MAX_EXPLOSION_PARTICLES: usize = 500;
pub const MAX_ENGINE_PARTICLES: usize = 200;

/// Effects plugin
pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>()
            .init_resource::<ScreenFlash>()
            .init_resource::<CameraZoom>()
            .init_resource::<HitStop>()
            .add_systems(OnEnter(GameState::Playing), starfield::spawn_starfield)
            // Split into multiple system groups due to Bevy tuple limits
            .add_systems(
                Update,
                (
                    starfield::update_starfield,
                    explosions::update_explosions,
                    explosions::update_shockwave_rings,
                    explosions::update_explosion_flashes,
                    explosions::update_explosion_embers,
                    screen_effects::update_screen_shake,
                    screen_effects::update_screen_flash,
                    overlays::update_salt_miner_tint,
                    overlays::update_low_health_vignette,
                    screen_effects::update_hit_stop,
                    screen_effects::update_camera_zoom,
                    explosions::handle_explosion_events,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    trails::spawn_engine_trails,
                    trails::update_engine_particles,
                    trails::spawn_bullet_trails,
                    trails::update_bullet_trails,
                    combat_feedback::update_hit_flash,
                    combat_feedback::update_damage_numbers,
                    muzzle_flash::tick_muzzle_flash,
                    impact_sparks::tick_impact_sparks,
                    abilities::spawn_ability_effects,
                    abilities::update_ability_effects,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    // Damage layer visual effects
                    damage_layers::handle_damage_layer_events,
                    damage_layers::update_shield_ripples,
                    damage_layers::update_armor_sparks,
                    damage_layers::update_hull_fire,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    // Pickup visual effects
                    pickups::handle_pickup_effect_events,
                    pickups::update_pickup_flashes,
                    pickups::update_pickup_shockwaves,
                    pickups::update_pickup_particles,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    // Active buff visual effects on player
                    buff_visuals::update_active_buff_visuals,
                    buff_visuals::update_overdrive_speed_lines,
                    buff_visuals::update_damage_boost_aura,
                    overlays::update_disintegrator_beams,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (
                    cleanup::cleanup_effects,
                    cleanup::cleanup_effects_2,
                    cleanup::cleanup_buff_visuals,
                ),
            );
    }
}
