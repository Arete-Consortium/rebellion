//! Environmental Objects — asteroids, wreckage, and terrain.
//!
//! Three-tier hierarchy:
//!   - Decorative debris: no gameplay collision (presentation only).
//!   - Soft hazards: contact damage + forgiving deflection + destructible.
//!   - Hard terrain: blocks movement + blocks projectiles + indestructible.
//!
//! All authoritative behavior runs in `FixedUpdate`. Presentation reactions
//! are decoupled via events.

use bevy::prelude::*;
use crate::core::DamageType;

// =============================================================================
// Marker & Kind
// =============================================================================

/// Marker for any environmental object that participates in gameplay.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvironmentObject;

/// What kind of gameplay object this is.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentKind {
    /// Contact damage + forgiving deflection. May be destructible.
    SoftHazard,
    /// Blocks movement and projectiles. Usually indestructible.
    HardTerrain,
}

// =============================================================================
// Geometry
// =============================================================================

/// Forgiving circular collider for environment objects.
///
/// Radius is typically 70–80% of the visible solid mass.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnvironmentCollider {
    pub radius: f32,
}

/// Result of a circle-circle contact test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CircleContact {
    /// Normal pointing FROM the environment object TOWARD the player.
    pub normal: Vec2,
    /// How deeply the two circles overlap (positive = penetrating).
    pub penetration: f32,
}

/// Pure contact test between two circles.
///
/// Returns `None` when separated or tangent.
/// Returns `Some(CircleContact)` when overlapping.
///
/// Invariant: `normal` points from `b` (environment) toward `a` (player).
pub fn circle_contact(
    a_position: Vec2,
    a_radius: f32,
    b_position: Vec2,
    b_radius: f32,
) -> Option<CircleContact> {
    let delta = a_position - b_position;
    let radius_sum = a_radius + b_radius;
    let distance_sq = delta.length_squared();

    if distance_sq >= radius_sum * radius_sum {
        return None;
    }

    // Zero-distance fallback: use Up as stable normal.
    if distance_sq <= f32::EPSILON {
        return Some(CircleContact {
            normal: Vec2::Y,
            penetration: radius_sum,
        });
    }

    let distance = distance_sq.sqrt();

    Some(CircleContact {
        normal: delta / distance,
        penetration: radius_sum - distance,
    })
}

// =============================================================================
// Movement
// =============================================================================

/// Deterministic movement for environmental objects.
///
/// All values are in world units per second.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnvironmentMotion {
    pub velocity: Vec2,
    pub angular_velocity: f32,
}

// =============================================================================
// Contact Damage
// =============================================================================

/// Damage dealt on first contact (and again after cooldown expires).
#[derive(Component, Debug, Clone, Copy)]
pub struct EnvironmentContactDamage {
    pub amount: f32,
    pub damage_type: crate::core::DamageType,
    /// Cooldown in fixed ticks (at 60 Hz, 30 ticks = 0.5 s).
    pub cooldown_ticks: u16,
}

/// Tracks remaining cooldown ticks for this specific hazard.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ContactCooldown {
    pub remaining_ticks: u16,
}

/// Default contact cooldown: 30 fixed ticks ≈ 0.5 s at 60 Hz.
pub const DEFAULT_CONTACT_COOLDOWN_TICKS: u16 = 30;

// =============================================================================
// Projectile Interaction
// =============================================================================

/// How projectiles interact with this environmental object.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileInteraction {
    /// Projectile passes through.
    Ignore,
    /// Projectile is destroyed on contact.
    Absorb,
    /// Projectile damages the environmental object.
    Damageable,
}

// =============================================================================
// Health & Destruction
// =============================================================================

/// Health for destructible environmental objects.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnvironmentHealth {
    pub current: f32,
    pub maximum: f32,
}

/// Score awarded when this object is destroyed.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnvironmentScoreValue(pub u64);

// =============================================================================
// Mission Ownership
// =============================================================================

/// Marks an entity as belonging to the current mission for cleanup.
#[derive(Component, Debug, Clone, Copy)]
pub struct MissionEnvironment;

// =============================================================================
// Events (Simulation → Presentation / Resolution)
// =============================================================================

/// Authoritative event: environmental object took damage from a projectile.
#[derive(Event, Debug, Clone, Copy)]
pub struct EnvironmentDamageAppliedEvent {
    pub environment: Entity,
    pub position: Vec2,
    pub damage: f32,
    pub damage_type: crate::core::DamageType,
    pub destroyed: bool,
}

/// Authoritative event: environmental object was destroyed.
#[derive(Event, Debug, Clone)]
pub struct EnvironmentDestroyedEvent {
    pub environment: Entity,
    pub definition_id: String,
    pub position: Vec2,
    pub score_value: u64,
}

/// Authoritative event: player contacted an environmental hazard.
/// Emitted by detection; consumed by separation + damage resolution.
#[derive(Event, Debug, Clone, Copy)]
pub struct PlayerEnvironmentContact {
    pub player: Entity,
    pub environment: Entity,
    pub player_position: Vec2,
    pub environment_position: Vec2,
    pub normal: Vec2,
    pub penetration: f32,
}

/// Projectile collided with an environment object
#[derive(Event, Debug, Clone, Copy)]
pub struct ProjectileEnvironmentContact {
    pub projectile: Entity,
    pub environment: Entity,
    pub projectile_pos: Vec2,
    pub environment_pos: Vec2,
    pub damage: f32,
    pub damage_type: DamageType,
    pub pierce_remaining: Option<u32>,
    pub is_player_projectile: bool,
}

// =============================================================================
// Boundary Pin Protection
// =============================================================================

/// After separating the player along the contact normal, verify the result
/// is inside playable bounds and not still overlapping the obstacle.
///
/// If the naive separation would pin the player against a screen edge,
/// evaluate escape candidates (left/right/above/below) and pick the nearest
/// valid one.
pub fn resolve_boundary_pin(
    player_pos: Vec2,
    player_radius: f32,
    env_pos: Vec2,
    env_radius: f32,
    contact: &CircleContact,
    slop: f32,
    screen_width: f32,
    screen_height: f32,
) -> Vec2 {
    // Naive separation: push player out along normal.
    let correction = contact.normal * (contact.penetration + slop);
    let candidate = player_pos + correction;

    // Playable bounds (centered at origin).
    let half_w = screen_width / 2.0;
    let half_h = screen_height / 2.0;
    let min_x = -half_w + player_radius;
    let max_x = half_w - player_radius;
    let min_y = -half_h + player_radius;
    let max_y = half_h - player_radius;

    // Clamp to bounds.
    let clamped = Vec2::new(
        candidate.x.clamp(min_x, max_x),
        candidate.y.clamp(min_y, max_y),
    );

    // Check if clamped position still overlaps the obstacle.
    let dist_sq = (clamped - env_pos).length_squared();
    let radius_sum = player_radius + env_radius + slop;
    if dist_sq >= radius_sum * radius_sum {
        return clamped; // Safe.
    }

    // Still overlapping — evaluate escape candidates around the obstacle.
    let candidates = [
        Vec2::new(env_pos.x - radius_sum, env_pos.y), // left
        Vec2::new(env_pos.x + radius_sum, env_pos.y), // right
        Vec2::new(env_pos.x, env_pos.y + radius_sum), // above
        Vec2::new(env_pos.x, env_pos.y - radius_sum), // below
    ];

    let mut best: Option<Vec2> = None;
    let mut best_dist_sq = f32::MAX;

    for c in candidates {
        let bounded = Vec2::new(
            c.x.clamp(min_x, max_x),
            c.y.clamp(min_y, max_y),
        );
        // Must not overlap obstacle.
        let d_sq = (bounded - env_pos).length_squared();
        if d_sq < radius_sum * radius_sum {
            continue;
        }
        // Must be inside bounds.
        if bounded.x <= min_x || bounded.x >= max_x || bounded.y <= min_y || bounded.y >= max_y
        {
            // Edge-case: allow if it's the best we have, but prefer interior.
        }
        let to_player_sq = (bounded - player_pos).length_squared();
        if to_player_sq < best_dist_sq {
            best_dist_sq = to_player_sq;
            best = Some(bounded);
        }
    }

    best.unwrap_or_else(|| {
        // Absolute fallback: place player at origin (should never happen).
        Vec2::new(0.0, 0.0)
    })
}

// =============================================================================
// Spawn Helper
// =============================================================================

/// Spawn an environmental object with all required components.
///
/// Returns the spawned entity id.
pub fn spawn_environment(
    commands: &mut Commands,
    position: Vec2,
    kind: EnvironmentKind,
    radius: f32,
    motion: Option<EnvironmentMotion>,
    health: Option<f32>,
    contact_damage: Option<EnvironmentContactDamage>,
    interaction: ProjectileInteraction,
    score_value: u64,
) -> Entity {
    let mut entity = commands.spawn((
        EnvironmentObject,
        kind,
        EnvironmentCollider { radius },
        MissionEnvironment,
        Transform::from_xyz(position.x, position.y, crate::core::constants::LAYER_ENEMIES),
        interaction,
        EnvironmentScoreValue(score_value),
    ));

    if let Some(m) = motion {
        entity.insert(m);
    }

    if let Some(hp) = health {
        entity.insert(EnvironmentHealth {
            current: hp,
            maximum: hp,
        });
    }

    if let Some(cd) = contact_damage {
        entity.insert(cd);
    }

    entity.id()
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_contact_returns_none_when_separated() {
        let result = circle_contact(Vec2::ZERO, 5.0, Vec2::new(20.0, 0.0), 5.0);
        assert!(result.is_none(), "separated circles should not contact");
    }

    #[test]
    fn circle_contact_returns_none_when_tangent() {
        let result = circle_contact(Vec2::ZERO, 5.0, Vec2::new(10.0, 0.0), 5.0);
        assert!(result.is_none(), "tangent circles should not contact");
    }

    #[test]
    fn circle_contact_returns_penetration_when_overlapping() {
        let result = circle_contact(Vec2::ZERO, 5.0, Vec2::new(8.0, 0.0), 5.0);
        let contact = result.expect("overlapping circles should contact");
        assert!(
            (contact.penetration - 2.0).abs() < 0.001,
            "expected penetration ~2.0, got {}",
            contact.penetration
        );
        assert!(
            (contact.normal - Vec2::new(-1.0, 0.0)).length() < 0.001,
            "normal should point from env toward player"
        );
    }

    #[test]
    fn circle_contact_zero_distance_fallback() {
        let result = circle_contact(Vec2::ZERO, 5.0, Vec2::ZERO, 5.0);
        let contact = result.expect("coincident circles should contact");
        assert_eq!(contact.normal, Vec2::Y);
        assert_eq!(contact.penetration, 10.0);
    }

    #[test]
    fn boundary_pin_simple_separation_works() {
        let player_pos = Vec2::new(0.0, 0.0);
        let env_pos = Vec2::new(0.0, 0.0);
        let contact = circle_contact(player_pos, 5.0, env_pos, 6.0).unwrap();
        let resolved = resolve_boundary_pin(
            player_pos, 5.0, env_pos, 6.0, &contact, 0.5, 800.0, 700.0,
        );
        let dist = (resolved - env_pos).length();
        assert!(
            dist >= 11.5,
            "resolved position should be separated by at least radius sum + slop"
        );
    }

    #[test]
    fn boundary_pin_does_not_push_out_of_bounds() {
        // Player at left edge, obstacle pushing further left.
        let player_pos = Vec2::new(-390.0, 0.0);
        let env_pos = Vec2::new(-405.0, 0.0);
        let contact = circle_contact(player_pos, 10.0, env_pos, 20.0).unwrap();
        let resolved = resolve_boundary_pin(
            player_pos, 10.0, env_pos, 20.0, &contact, 0.5, 800.0, 700.0,
        );
        assert!(
            resolved.x >= -390.0,
            "player should not be pushed out of bounds (got x={})",
            resolved.x
        );
    }

    #[test]
    fn boundary_pin_finds_escape_candidate() {
        // Player pinned at left boundary directly against obstacle.
        let player_pos = Vec2::new(-390.0, 0.0);
        let env_pos = Vec2::new(-380.0, 0.0);
        let contact = circle_contact(player_pos, 10.0, env_pos, 15.0).unwrap();
        let resolved = resolve_boundary_pin(
            player_pos, 10.0, env_pos, 15.0, &contact, 0.5, 800.0, 700.0,
        );
        let dist = (resolved - env_pos).length();
        assert!(
            dist >= 25.5,
            "escape candidate should separate player from obstacle"
        );
    }

    #[test]
    fn contact_cooldown_saturates_at_zero() {
        let mut cd = ContactCooldown { remaining_ticks: 1 };
        cd.remaining_ticks = cd.remaining_ticks.saturating_sub(1);
        assert_eq!(cd.remaining_ticks, 0);
        cd.remaining_ticks = cd.remaining_ticks.saturating_sub(1);
        assert_eq!(cd.remaining_ticks, 0, "should not underflow");
    }
}
