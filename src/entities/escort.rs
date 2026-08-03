//! Escort / Friendly Entity System
//!
//! Supports the EscortObjective: friendly NPCs that move along waypoint paths
//! and must survive for mission completion.

use crate::core::{DamageType, GameState, LAYER_PLAYER};
use crate::entities::player::DamageResult;
use crate::entities::Hitbox;
use bevy::prelude::*;

/// Marker component for friendly entities the player must protect
#[derive(Component, Debug)]
pub struct Friendly;

/// Data for an escort target — health, path waypoints, movement speed
#[derive(Component, Debug, Clone)]
pub struct EscortData {
    /// Current health
    pub health: f32,
    /// Maximum health
    pub max_health: f32,
    /// Waypoints to follow (world coordinates)
    pub waypoints: Vec<Vec2>,
    /// Index of the current waypoint being moved toward
    pub current_waypoint: usize,
    /// Movement speed in units per second
    pub speed: f32,
    /// Waypoint arrival threshold (squared distance)
    pub arrival_threshold_sq: f32,
    /// Whether the escort has reached the final waypoint
    pub reached_end: bool,
}

impl Default for EscortData {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            waypoints: Vec::new(),
            current_waypoint: 0,
            speed: 80.0,
            arrival_threshold_sq: 16.0, // 4 units squared
            reached_end: false,
        }
    }
}

impl EscortData {
    /// Create escort data with a horizontal path from left to right
    pub fn horizontal_path(y: f32, start_x: f32, end_x: f32, segments: usize, speed: f32) -> Self {
        let mut waypoints = Vec::with_capacity(segments + 1);
        let step = (end_x - start_x) / segments as f32;
        for i in 0..=segments {
            waypoints.push(Vec2::new(start_x + step * i as f32, y));
        }
        Self {
            waypoints,
            speed,
            ..Default::default()
        }
    }

    /// Returns health fraction (0.0–1.0)
    pub fn health_fraction(&self) -> f32 {
        if self.max_health > 0.0 {
            (self.health / self.max_health).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Take damage with simple hull-only reduction (escorts don't have shield/armor layers)
    pub fn take_damage(&mut self, damage: f32, _damage_type: DamageType) -> DamageResult {
        self.health -= damage;
        let destroyed = self.health <= 0.0;
        DamageResult {
            destroyed,
            shield_damage: 0.0,
            armor_damage: 0.0,
            hull_damage: damage,
        }
    }
}

/// Spawn a friendly escort entity at the given position with the given path
#[allow(dead_code)]
pub fn spawn_friendly_escort(
    commands: &mut Commands,
    position: Vec2,
    escort_data: EscortData,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Friendly,
            escort_data.clone(),
            Hitbox {
                radius: 12.0, // Slightly larger than player hitbox
            },
            Sprite {
                color,
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_PLAYER),
        ))
        .id()
}

/// Update escort movement along its waypoint path
pub fn update_escort_movement(
    time: Res<Time>,
    mut escort_query: Query<(&mut Transform, &mut EscortData), (With<Friendly>,)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut data) in escort_query.iter_mut() {
        if data.waypoints.is_empty() || data.reached_end {
            continue;
        }

        let target = data.waypoints[data.current_waypoint];
        let pos = transform.translation.xy();
        let to_target = target - pos;
        let dist_sq = to_target.length_squared();

        if dist_sq <= data.arrival_threshold_sq {
            // Reached waypoint
            data.current_waypoint += 1;
            if data.current_waypoint >= data.waypoints.len() {
                data.reached_end = true;
                data.current_waypoint = data.waypoints.len().saturating_sub(1);
            }
            continue;
        }

        let direction = to_target.normalize_or_zero();
        let movement = direction * data.speed * dt;
        transform.translation += movement.extend(0.0);
    }
}

/// Despawn escort entities when leaving gameplay states
pub fn despawn_escorts(mut commands: Commands, escort_query: Query<Entity, With<Friendly>>) {
    for entity in escort_query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Plugin for escort / friendly entity systems
pub struct EscortPlugin;

impl Plugin for EscortPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_escort_movement.run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), despawn_escorts)
        .add_systems(OnExit(GameState::BossFight), despawn_escorts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escort_data_horizontal_path_points() {
        let data = EscortData::horizontal_path(100.0, -200.0, 200.0, 4, 50.0);
        assert_eq!(data.waypoints.len(), 5);
        assert_eq!(data.waypoints[0], Vec2::new(-200.0, 100.0));
        assert_eq!(data.waypoints[4], Vec2::new(200.0, 100.0));
    }

    #[test]
    fn escort_data_health_fraction() {
        let mut data = EscortData::default();
        assert!((data.health_fraction() - 1.0).abs() < f32::EPSILON);
        data.health = 50.0;
        assert!((data.health_fraction() - 0.5).abs() < f32::EPSILON);
        data.health = 0.0;
        assert!((data.health_fraction()).abs() < f32::EPSILON);
    }

    #[test]
    fn escort_data_take_damage() {
        let mut data = EscortData::default();
        let result = data.take_damage(30.0, DamageType::Thermal);
        assert_eq!(data.health, 70.0);
        assert!(!result.destroyed);
        let result = data.take_damage(80.0, DamageType::Kinetic);
        assert!(result.destroyed);
        assert!(data.health <= 0.0);
    }
}
