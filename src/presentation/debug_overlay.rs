//! Debug Visualization Overlay
//!
//! Toggleable gizmo rendering for environment colliders and other hitboxes.
//! Only active when `DevConfig::show_environment_colliders` is true.

use bevy::prelude::*;

use crate::entities::environment::{EnvironmentCollider, EnvironmentObject};
use crate::entities::Hitbox;
use crate::entities::Player;

/// Dev/debug toggle resource.
#[derive(Resource, Debug, Clone, Copy)]
pub struct DevConfig {
    pub show_environment_colliders: bool,
    pub show_player_hitbox: bool,
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            show_environment_colliders: false,
            show_player_hitbox: false,
        }
    }
}

/// Draw wireframe circles for environment colliders and player hitbox.
pub fn draw_environment_colliders(
    mut gizmos: Gizmos,
    dev: Res<DevConfig>,
    env_query: Query<(&Transform, &EnvironmentCollider), With<EnvironmentObject>>,
    player_query: Query<(&Transform, &Hitbox), With<Player>>,
) {
    if dev.show_environment_colliders {
        for (transform, collider) in env_query.iter() {
            let pos = transform.translation.truncate();
            gizmos.circle_2d(pos, collider.radius, Color::srgb(0.0, 1.0, 0.5));
        }
    }

    if dev.show_player_hitbox {
        if let Ok((transform, hitbox)) = player_query.get_single() {
            let pos = transform.translation.truncate();
            gizmos.circle_2d(pos, hitbox.radius, Color::srgb(1.0, 0.2, 0.2));
        }
    }
}
