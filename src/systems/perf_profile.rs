//! Runtime performance profile.
//!
//! Holds the effective per-frame caps for particle systems and effects.
//! Initialized at startup from `MobileMode`: desktop = full caps, mobile
//! = roughly 30% across the board so iOS Safari can hit 60fps. The
//! game-feel difference on small phone screens is minimal — most
//! particle effects are visually busy at 100% and still read fine at 30%.

#![allow(dead_code)]

use bevy::prelude::*;

use super::touch_joystick::MobileMode;

#[derive(Resource, Debug, Clone, Copy)]
pub struct PerfProfile {
    pub explosion_particles: usize,
    pub engine_particles: usize,
    pub damage_layer_particles: usize,
    pub shield_ripples: usize,
    pub speed_lines: usize,
    pub aura_particles: usize,
    pub trail_particles: usize,
    pub pickup_particles: usize,
}

impl PerfProfile {
    pub const fn desktop() -> Self {
        Self {
            explosion_particles: 500,
            engine_particles: 200,
            damage_layer_particles: 100,
            shield_ripples: 15,
            speed_lines: 30,
            aura_particles: 12,
            trail_particles: 300,
            pickup_particles: 100,
        }
    }

    pub const fn mobile() -> Self {
        Self {
            explosion_particles: 150,
            engine_particles: 60,
            damage_layer_particles: 30,
            shield_ripples: 6,
            speed_lines: 10,
            aura_particles: 5,
            trail_particles: 90,
            pickup_particles: 30,
        }
    }
}

impl Default for PerfProfile {
    fn default() -> Self {
        Self::desktop()
    }
}

pub struct PerfProfilePlugin;

impl Plugin for PerfProfilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerfProfile>()
            // Apply mobile profile after MobileMode is detected (also
            // a Startup system in TouchJoystickPlugin).
            .add_systems(Startup, apply_mobile_profile_if_active);
    }
}

fn apply_mobile_profile_if_active(
    mobile: Res<MobileMode>,
    mut profile: ResMut<PerfProfile>,
) {
    if mobile.active {
        *profile = PerfProfile::mobile();
        info!(
            "Mobile perf profile applied: explosions={}, engines={}, trails={}",
            profile.explosion_particles,
            profile.engine_particles,
            profile.trail_particles,
        );
    }
}
