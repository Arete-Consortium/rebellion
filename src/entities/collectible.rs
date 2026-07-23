//! Collectible Entities
//!
//! Power-ups, refugees, credits, etc.
//! Features a rarity system ported from the Python version.

#![allow(dead_code)]

use crate::core::*;
use crate::systems::{check_liberation_milestone, ComboHeatSystem, DialogueEvent};
use bevy::prelude::*;
use std::f32::consts::TAU;

// =============================================================================
// RARITY SYSTEM
// =============================================================================

/// Rarity tier for collectibles - affects visual intensity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
}

impl Rarity {
    /// Get the rarity for a collectible type
    pub fn for_collectible(collectible_type: CollectibleType) -> Self {
        match collectible_type {
            // Common - basic drops
            CollectibleType::Credits => Rarity::Common,
            CollectibleType::ShieldBoost => Rarity::Common,
            CollectibleType::ArmorRepair => Rarity::Common,
            CollectibleType::HullRepair => Rarity::Common,
            CollectibleType::CapacitorCharge => Rarity::Common,

            // Uncommon - moderate utility
            CollectibleType::Nanite => Rarity::Uncommon,
            CollectibleType::LiberationPod => Rarity::Uncommon,
            CollectibleType::SkillPointDrop => Rarity::Uncommon,

            // Rare - significant powerups
            CollectibleType::Overdrive => Rarity::Rare,
            CollectibleType::DamageBoost => Rarity::Rare,

            // Epic - game-changing effects
            CollectibleType::Invulnerability => Rarity::Epic,
            CollectibleType::ExtraLife => Rarity::Epic,

            // Persistent weapon mods
            CollectibleType::ScatterLauncher => Rarity::Uncommon,
            CollectibleType::HomingSwarm => Rarity::Uncommon,
            CollectibleType::RailSpike => Rarity::Rare,
            CollectibleType::PlasmaLance => Rarity::Rare,
            CollectibleType::VortonProjector => Rarity::Epic,
        }
    }

    /// Glow intensity multiplier
    pub fn glow_mult(&self) -> f32 {
        match self {
            Rarity::Common => 0.6,
            Rarity::Uncommon => 0.8,
            Rarity::Rare => 1.0,
            Rarity::Epic => 1.3,
        }
    }

    /// Number of orbital particles
    pub fn orbital_count(&self) -> u32 {
        match self {
            Rarity::Common => 4,
            Rarity::Uncommon => 6,
            Rarity::Rare => 8,
            Rarity::Epic => 12,
        }
    }

    /// Pulse animation speed
    pub fn pulse_speed(&self) -> f32 {
        match self {
            Rarity::Common => 2.0,
            Rarity::Uncommon => 2.5,
            Rarity::Rare => 3.0,
            Rarity::Epic => 4.0,
        }
    }

    /// Size multiplier for the collectible
    pub fn size_mult(&self) -> f32 {
        match self {
            Rarity::Common => 1.0,
            Rarity::Uncommon => 1.1,
            Rarity::Rare => 1.2,
            Rarity::Epic => 1.35,
        }
    }

    /// Border/outline color for rarity indicator
    pub fn border_color(&self) -> Color {
        match self {
            Rarity::Common => Color::srgba(0.6, 0.6, 0.6, 0.5), // Gray
            Rarity::Uncommon => Color::srgba(0.2, 0.8, 0.3, 0.7), // Green
            Rarity::Rare => Color::srgba(0.3, 0.5, 1.0, 0.8),   // Blue
            Rarity::Epic => Color::srgba(0.8, 0.4, 1.0, 0.9),   // Purple
        }
    }
}

/// Component for rarity-based visual effects
#[derive(Component, Debug)]
pub struct CollectibleRarity {
    pub rarity: Rarity,
    /// Animation phase for pulsing/orbital effects
    pub phase: f32,
    /// Base size before rarity scaling
    pub base_size: f32,
}

/// Orbital particle that circles around a collectible
#[derive(Component)]
pub struct OrbitalParticle {
    /// Entity this particle orbits around
    pub parent: Entity,
    /// Current angle around the parent
    pub angle: f32,
    /// Orbital radius
    pub radius: f32,
    /// Angular velocity (radians per second)
    pub speed: f32,
}

/// Maximum orbital particles per collectible (to prevent lag)
const MAX_ORBITALS_PER_COLLECTIBLE: usize = 12;

/// Marker component for collectibles
#[derive(Component, Debug)]
pub struct Collectible;

/// Collectible data
#[derive(Component, Debug, Clone)]
pub struct CollectibleData {
    /// Type of collectible
    pub collectible_type: CollectibleType,
    /// Value (amount of healing, credits, etc)
    pub value: u32,
}

/// Collectible movement pattern
#[derive(Component, Debug, Clone)]
pub struct CollectiblePhysics {
    /// Current velocity
    pub velocity: Vec2,
    /// Oscillation for floating effect
    pub oscillation: f32,
    /// Lifetime
    pub lifetime: f32,
}

impl Default for CollectiblePhysics {
    fn default() -> Self {
        Self {
            velocity: Vec2::new(0.0, -30.0),
            oscillation: 0.0,
            lifetime: 10.0,
        }
    }
}

/// Active powerup effects on the player
#[derive(Component, Debug, Default)]
pub struct PowerupEffects {
    /// Overdrive timer (speed boost)
    pub overdrive_timer: f32,
    /// Damage boost timer
    pub damage_boost_timer: f32,
    /// Invulnerability timer
    pub invuln_timer: f32,
}

impl PowerupEffects {
    pub fn is_overdrive(&self) -> bool {
        self.overdrive_timer > 0.0
    }

    pub fn is_damage_boosted(&self) -> bool {
        self.damage_boost_timer > 0.0
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invuln_timer > 0.0
    }

    pub fn speed_mult(&self) -> f32 {
        if self.is_overdrive() {
            1.5
        } else {
            1.0
        }
    }

    pub fn damage_mult(&self) -> f32 {
        if self.is_damage_boosted() {
            2.0
        } else {
            1.0
        }
    }
}

/// Bundle for spawning collectibles
#[derive(Bundle)]
pub struct CollectibleBundle {
    pub collectible: Collectible,
    pub data: CollectibleData,
    pub physics: CollectiblePhysics,
    pub rarity: CollectibleRarity,
    pub sprite: Sprite,
    pub transform: Transform,
}

/// Collectible plugin
pub struct CollectiblePlugin;

impl Plugin for CollectiblePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                collectible_movement,
                collectible_lifetime,
                collectible_rarity_effects,
                spawn_orbital_particles,
                update_orbital_particles,
                collectible_pickup,
                handle_pickup_effects,
                super::items::recompute_stats,
                update_powerup_timers,
            )
                .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
        );
    }
}

/// Move collectibles with floating effect
fn collectible_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CollectiblePhysics), With<Collectible>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut physics) in query.iter_mut() {
        physics.oscillation += dt * 3.0;

        // Float and drift
        let float_offset = physics.oscillation.sin() * 0.5;
        transform.translation.x += physics.velocity.x * dt + float_offset;
        transform.translation.y += physics.velocity.y * dt;
    }
}

/// Animate rarity-based visual effects
fn collectible_rarity_effects(
    time: Res<Time>,
    mut query: Query<(&mut CollectibleRarity, &mut Sprite, &mut Transform), With<Collectible>>,
) {
    let dt = time.delta_secs();

    for (mut rarity_data, mut sprite, mut transform) in query.iter_mut() {
        // Update animation phase
        rarity_data.phase += dt * rarity_data.rarity.pulse_speed();
        if rarity_data.phase > TAU {
            rarity_data.phase -= TAU;
        }

        // Pulsing glow effect - scale and alpha
        let pulse = 0.85 + 0.15 * rarity_data.phase.sin();
        let glow_mult = rarity_data.rarity.glow_mult();

        // Update size with pulse
        let target_size = rarity_data.base_size * rarity_data.rarity.size_mult() * pulse;
        sprite.custom_size = Some(Vec2::splat(target_size));

        // Update alpha with glow intensity
        let base_alpha = match rarity_data.rarity {
            Rarity::Common => 0.85,
            Rarity::Uncommon => 0.9,
            Rarity::Rare => 0.95,
            Rarity::Epic => 1.0,
        };
        let alpha = base_alpha * glow_mult * (0.9 + 0.1 * rarity_data.phase.cos());

        // Apply alpha to sprite color
        let current = sprite.color.to_srgba();
        sprite.color = Color::srgba(current.red, current.green, current.blue, alpha);

        // Slight rotation for rare/epic items
        if matches!(rarity_data.rarity, Rarity::Rare | Rarity::Epic) {
            let rotation_speed = if rarity_data.rarity == Rarity::Epic {
                0.5
            } else {
                0.25
            };
            transform.rotation = Quat::from_rotation_z(rarity_data.phase * rotation_speed);
        }
    }
}

/// Spawn orbital particles for rare/epic collectibles
fn spawn_orbital_particles(
    mut commands: Commands,
    collectibles: Query<(Entity, &CollectibleRarity, &CollectibleData), Added<CollectibleRarity>>,
) {
    for (entity, rarity_data, data) in collectibles.iter() {
        let orbital_count = rarity_data.rarity.orbital_count() as usize;

        // Only spawn orbitals for uncommon+ rarity
        if orbital_count <= 4 {
            continue;
        }

        // Get the color for orbitals based on collectible type
        let orbital_color = match data.collectible_type {
            CollectibleType::Overdrive => Color::srgba(0.3, 0.9, 1.0, 0.7),
            CollectibleType::DamageBoost => Color::srgba(1.0, 0.4, 0.3, 0.7),
            CollectibleType::Invulnerability => Color::srgba(1.0, 1.0, 0.8, 0.8),
            CollectibleType::ExtraLife => Color::srgba(0.3, 1.0, 0.5, 0.8),
            _ => rarity_data.rarity.border_color(),
        };

        // Spawn orbital particles
        let num_orbitals = orbital_count.min(MAX_ORBITALS_PER_COLLECTIBLE);
        let radius = rarity_data.base_size * 0.8;

        for i in 0..num_orbitals {
            let angle = (i as f32 / num_orbitals as f32) * TAU;
            let speed = 2.0 + fastrand::f32() * 1.0; // Vary speeds slightly

            commands.spawn((
                OrbitalParticle {
                    parent: entity,
                    angle,
                    radius,
                    speed,
                },
                Sprite {
                    color: orbital_color,
                    custom_size: Some(Vec2::splat(3.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, LAYER_EFFECTS + 0.1),
            ));
        }
    }
}

/// Update orbital particle positions
fn update_orbital_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut orbitals: Query<(Entity, &mut OrbitalParticle, &mut Transform)>,
    parents: Query<&Transform, (With<Collectible>, Without<OrbitalParticle>)>,
) {
    let dt = time.delta_secs();

    for (entity, mut orbital, mut transform) in orbitals.iter_mut() {
        // Check if parent still exists
        if let Ok(parent_transform) = parents.get(orbital.parent) {
            // Update angle
            orbital.angle += orbital.speed * dt;
            if orbital.angle > TAU {
                orbital.angle -= TAU;
            }

            // Calculate position around parent
            let offset_x = orbital.angle.cos() * orbital.radius;
            let offset_y = orbital.angle.sin() * orbital.radius;

            transform.translation.x = parent_transform.translation.x + offset_x;
            transform.translation.y = parent_transform.translation.y + offset_y;
            transform.translation.z = parent_transform.translation.z + 0.1;
        } else {
            // Parent despawned, despawn orbital too
            commands.entity(entity).despawn();
        }
    }
}

/// Update collectible lifetime
fn collectible_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut CollectiblePhysics, &mut Sprite), With<Collectible>>,
) {
    let dt = time.delta_secs();

    for (entity, mut physics, mut sprite) in query.iter_mut() {
        physics.lifetime -= dt;

        // Blink when about to expire
        if physics.lifetime < 3.0 {
            let alpha = (physics.lifetime * 5.0).sin().abs() * 0.5 + 0.5;
            sprite.color = sprite.color.with_alpha(alpha);
        }

        if physics.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Check for player pickup
fn collectible_pickup(
    mut commands: Commands,
    player_query: Query<&Transform, With<super::Player>>,
    collectible_query: Query<
        (
            Entity,
            &Transform,
            &CollectibleData,
            Option<&Sprite>,
            &CollectibleRarity,
        ),
        With<Collectible>,
    >,
    mut pickup_events: EventWriter<CollectiblePickedUpEvent>,
    mut effect_events: EventWriter<PickupEffectEvent>,
    mut screen_shake: ResMut<crate::systems::effects::ScreenShake>,
    mut screen_flash: ResMut<crate::systems::effects::ScreenFlash>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let pickup_radius = 30.0;

    for (entity, transform, data, sprite, rarity_data) in collectible_query.iter() {
        let collectible_pos = transform.translation.truncate();
        let distance = (player_pos - collectible_pos).length();

        if distance < pickup_radius {
            // Get color from sprite for visual effect
            let color = sprite.map(|s| s.color).unwrap_or(Color::WHITE);

            // Send pickup event
            pickup_events.send(CollectiblePickedUpEvent {
                collectible_type: data.collectible_type,
                position: collectible_pos,
                value: data.value,
            });

            // Send visual effect event
            effect_events.send(PickupEffectEvent {
                position: collectible_pos,
                collectible_type: data.collectible_type,
                color,
            });

            // Screen shake on any pickup
            screen_shake.trigger(2.0, 0.05);

            // Screen flash with rarity border color for rare/epic pickups
            if matches!(rarity_data.rarity, Rarity::Rare | Rarity::Epic) {
                screen_flash.colored(rarity_data.rarity.border_color(), 0.3);
            }

            // Despawn collectible
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Handle pickup effects - apply powerup to player
fn handle_pickup_effects(
    mut pickup_events: EventReader<CollectiblePickedUpEvent>,
    mut player_query: Query<
        (
            &mut super::player::ShipStats,
            &mut PowerupEffects,
            Option<&mut super::items::Inventory>,
        ),
        With<super::Player>,
    >,
    mut score: ResMut<ScoreSystem>,
    mut progress: ResMut<GameProgress>,
    mut save_data: ResMut<crate::core::SaveData>,
    mut heat_system: ResMut<ComboHeatSystem>,
    mut dialogue_events: EventWriter<DialogueEvent>,
    mut rumble_events: EventWriter<crate::systems::RumbleRequest>,
) {
    let Ok((mut stats, mut effects, mut inventory)) = player_query.get_single_mut() else {
        return;
    };

    for event in pickup_events.read() {
        match event.collectible_type {
            CollectibleType::LiberationPod => {
                let old_count = score.souls_liberated;
                score.souls_liberated += 1;
                score.add_score(500);

                // Award skill points (1 SP per soul liberated)
                save_data.add_skill_points(1);

                // Check for liberation milestone
                if let Some(milestone) =
                    check_liberation_milestone(old_count, score.souls_liberated)
                {
                    dialogue_events.send(DialogueEvent::liberation_milestone(milestone));
                    info!("Liberation milestone reached: {} souls!", milestone);
                }
            }
            CollectibleType::SkillPointDrop => {
                // Direct SP drop (from special enemies or bonuses)
                save_data.add_skill_points(event.value);
                info!("Gained {} SP!", event.value);
            }
            CollectibleType::Credits => {
                progress.credits += event.value as u64;
            }
            CollectibleType::ShieldBoost => {
                stats.shield = (stats.shield + event.value as f32).min(stats.max_shield);
                info!("Shield +{}", event.value);
            }
            CollectibleType::ArmorRepair => {
                stats.armor = (stats.armor + event.value as f32).min(stats.max_armor);
                info!("Armor +{}", event.value);
            }
            CollectibleType::HullRepair => {
                stats.hull = (stats.hull + event.value as f32).min(stats.max_hull);
                info!("Hull +{}", event.value);
            }
            CollectibleType::CapacitorCharge => {
                stats.capacitor = (stats.capacitor + event.value as f32).min(stats.max_capacitor);
            }
            CollectibleType::Overdrive => {
                effects.overdrive_timer = 5.0; // 5 second speed boost
                rumble_events.send(crate::systems::RumbleRequest::powerup());
                info!("OVERDRIVE ACTIVATED!");
            }
            CollectibleType::DamageBoost => {
                effects.damage_boost_timer = 10.0; // 10 second damage boost
                rumble_events.send(crate::systems::RumbleRequest::powerup());
                info!("DAMAGE BOOST!");
            }
            CollectibleType::Invulnerability => {
                effects.invuln_timer = 3.0; // 3 seconds of invuln
                rumble_events.send(crate::systems::RumbleRequest::powerup());
                info!("INVULNERABLE!");
            }
            CollectibleType::Nanite => {
                heat_system.reduce_heat(50.0);
                info!("Heat reduced by nanites");
            }
            CollectibleType::ExtraLife => {
                // Restore all HP
                stats.shield = stats.max_shield;
                stats.armor = stats.max_armor;
                stats.hull = stats.max_hull;
                info!("EXTRA LIFE! Full HP restored!");
            }
            // Persistent weapon mods — push onto inventory, mark dirty
            CollectibleType::ScatterLauncher
            | CollectibleType::RailSpike
            | CollectibleType::PlasmaLance
            | CollectibleType::HomingSwarm
            | CollectibleType::VortonProjector => {
                if let Some(inv) = inventory.as_mut() {
                    inv.add(event.collectible_type);
                    let count = inv
                        .stacks
                        .iter()
                        .find(|s| s.id == event.collectible_type)
                        .map(|s| s.count)
                        .unwrap_or(0);
                    info!("{:?} stack {}", event.collectible_type, count);
                    rumble_events.send(crate::systems::RumbleRequest::powerup());
                }
            }
        }
    }
}

/// Update powerup effect timers
fn update_powerup_timers(time: Res<Time>, mut query: Query<&mut PowerupEffects>) {
    let dt = time.delta_secs();
    for mut effects in query.iter_mut() {
        if effects.overdrive_timer > 0.0 {
            effects.overdrive_timer -= dt;
        }
        if effects.damage_boost_timer > 0.0 {
            effects.damage_boost_timer -= dt;
        }
        if effects.invuln_timer > 0.0 {
            effects.invuln_timer -= dt;
        }
    }
}

/// Spawn a collectible at position
pub fn spawn_collectible(
    commands: &mut Commands,
    position: Vec2,
    collectible_type: CollectibleType,
    icon_cache: Option<&crate::assets::PowerupIconCache>,
) {
    let (color, base_size, value) = match collectible_type {
        CollectibleType::LiberationPod => (Color::srgb(0.2, 0.9, 0.5), 20.0, 1), // Green glow
        CollectibleType::Credits => (Color::srgb(1.0, 0.84, 0.0), 12.0, 100),
        CollectibleType::ShieldBoost => (COLOR_SHIELD, 28.0, 25),
        CollectibleType::ArmorRepair => (COLOR_ARMOR, 28.0, 25),
        CollectibleType::HullRepair => (COLOR_HULL, 28.0, 25),
        CollectibleType::CapacitorCharge => (COLOR_CAPACITOR, 14.0, 50),
        CollectibleType::Overdrive => (Color::srgb(0.3, 0.9, 1.0), 28.0, 1),
        CollectibleType::DamageBoost => (Color::srgb(1.0, 0.3, 0.3), 28.0, 1),
        CollectibleType::Invulnerability => (Color::srgb(1.0, 1.0, 1.0), 28.0, 1),
        CollectibleType::Nanite => (Color::srgb(0.0, 0.8, 0.6), 28.0, 1),
        CollectibleType::ExtraLife => (Color::srgb(0.0, 1.0, 0.5), 28.0, 1),
        CollectibleType::SkillPointDrop => (Color::srgb(0.9, 0.7, 1.0), 18.0, 5), // Purple glow, 5 SP
        CollectibleType::ScatterLauncher => (Color::srgb(0.30, 0.70, 1.00), 24.0, 1),
        CollectibleType::RailSpike => (Color::srgb(0.80, 0.85, 0.90), 24.0, 1),
        CollectibleType::PlasmaLance => (Color::srgb(1.00, 0.35, 0.80), 24.0, 1),
        CollectibleType::HomingSwarm => (Color::srgb(1.00, 0.40, 0.40), 24.0, 1),
        CollectibleType::VortonProjector => (Color::srgb(0.55, 0.90, 1.00), 24.0, 1),
    };

    // Determine rarity and scale size accordingly
    let rarity = Rarity::for_collectible(collectible_type);
    let size = base_size * rarity.size_mult();

    // Try to use icon from cache, fallback to colored sprite
    let sprite = if let Some(cache) = icon_cache {
        if let Some(texture) = cache.get(&collectible_type) {
            Sprite {
                image: texture,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        } else {
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        }
    } else {
        Sprite {
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        }
    };

    commands.spawn(CollectibleBundle {
        collectible: Collectible,
        data: CollectibleData {
            collectible_type,
            value,
        },
        physics: CollectiblePhysics {
            velocity: Vec2::new(0.0, -20.0),
            oscillation: fastrand::f32() * TAU,
            lifetime: 10.0,
        },
        rarity: CollectibleRarity {
            rarity,
            phase: fastrand::f32() * TAU,
            base_size,
        },
        sprite,
        transform: Transform::from_xyz(position.x, position.y, LAYER_EFFECTS),
    });
}

/// Spawn liberation pods in a burst pattern
pub fn spawn_liberation_pods(commands: &mut Commands, position: Vec2, count: u32) {
    // Cap at reasonable maximum to avoid lag
    let pod_count = count.min(20);
    let rarity = Rarity::for_collectible(CollectibleType::LiberationPod);
    let base_size = 16.0;
    let size = base_size * rarity.size_mult();

    for i in 0..pod_count {
        // Spread pods in a circle burst
        let angle = (i as f32 / pod_count as f32) * TAU + fastrand::f32() * 0.3;
        let speed = 40.0 + fastrand::f32() * 30.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed - 20.0);

        // Offset spawn position slightly
        let offset = Vec2::new(
            (fastrand::f32() - 0.5) * 20.0,
            (fastrand::f32() - 0.5) * 20.0,
        );

        commands.spawn(CollectibleBundle {
            collectible: Collectible,
            data: CollectibleData {
                collectible_type: CollectibleType::LiberationPod,
                value: 1,
            },
            physics: CollectiblePhysics {
                velocity,
                oscillation: fastrand::f32() * TAU,
                lifetime: 12.0, // Pods last longer than powerups
            },
            rarity: CollectibleRarity {
                rarity,
                phase: fastrand::f32() * TAU,
                base_size,
            },
            sprite: Sprite {
                color: Color::srgb(0.2, 0.9, 0.5), // Green glow
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            transform: Transform::from_xyz(
                position.x + offset.x,
                position.y + offset.y,
                LAYER_EFFECTS,
            ),
        });
    }
}

/// Player health state for smart powerup drops
#[derive(Debug, Clone, Copy)]
pub struct PlayerHealthState {
    pub shield_percent: f32,
    pub armor_percent: f32,
    pub hull_percent: f32,
}

impl PlayerHealthState {
    pub fn from_stats(stats: &super::player::ShipStats) -> Self {
        Self {
            shield_percent: if stats.max_shield > 0.0 {
                stats.shield / stats.max_shield
            } else {
                1.0
            },
            armor_percent: if stats.max_armor > 0.0 {
                stats.armor / stats.max_armor
            } else {
                1.0
            },
            hull_percent: if stats.max_hull > 0.0 {
                stats.hull / stats.max_hull
            } else {
                1.0
            },
        }
    }

    /// Determine what health type is most needed
    pub fn most_needed_health(&self) -> CollectibleType {
        // Priority: Hull (critical) > Armor > Shield
        if self.hull_percent < 0.5 {
            // Hull is low - could give any health type, weighted toward hull/armor
            let roll = fastrand::f32();
            if roll < 0.4 {
                CollectibleType::HullRepair
            } else if roll < 0.75 {
                CollectibleType::ArmorRepair
            } else {
                CollectibleType::ShieldBoost
            }
        } else if self.armor_percent < 0.5 {
            // Armor is low - give armor or shield
            let roll = fastrand::f32();
            if roll < 0.6 {
                CollectibleType::ArmorRepair
            } else {
                CollectibleType::ShieldBoost
            }
        } else if self.shield_percent < 0.7 {
            // Shield is down - primarily shield
            CollectibleType::ShieldBoost
        } else {
            // Player is healthy - random health type
            let roll = fastrand::f32();
            if roll < 0.5 {
                CollectibleType::ShieldBoost
            } else if roll < 0.8 {
                CollectibleType::ArmorRepair
            } else {
                CollectibleType::HullRepair
            }
        }
    }
}

/// Spawn random powerup with weighted chances (legacy - no health awareness)
pub fn spawn_random_powerup(
    commands: &mut Commands,
    position: Vec2,
    icon_cache: Option<&crate::assets::PowerupIconCache>,
) {
    spawn_smart_powerup(commands, position, icon_cache, None);
}

/// Spawn powerup that's smart about what the player needs
pub fn spawn_smart_powerup(
    commands: &mut Commands,
    position: Vec2,
    icon_cache: Option<&crate::assets::PowerupIconCache>,
    player_health: Option<PlayerHealthState>,
) {
    let roll = fastrand::f32();

    // 20% credits, 30% health, 25% temp powerups, 5% epic temp,
    // 20% persistent weapon mods (rarity-weighted inside)
    let powerup = if roll < 0.20 {
        CollectibleType::Credits
    } else if roll < 0.50 {
        // Health drop - be smart about what type
        if let Some(health) = player_health {
            health.most_needed_health()
        } else {
            let h = fastrand::f32();
            if h < 0.4 {
                CollectibleType::ShieldBoost
            } else if h < 0.75 {
                CollectibleType::ArmorRepair
            } else {
                CollectibleType::HullRepair
            }
        }
    } else if roll < 0.60 {
        CollectibleType::Overdrive
    } else if roll < 0.68 {
        CollectibleType::DamageBoost
    } else if roll < 0.74 {
        CollectibleType::Nanite
    } else if roll < 0.78 {
        CollectibleType::Invulnerability
    } else if roll < 0.80 {
        CollectibleType::ExtraLife
    } else {
        // Weapon mod bucket: 20% of drops, weighted by rarity
        let m = fastrand::f32();
        if m < 0.35 {
            CollectibleType::ScatterLauncher // Uncommon
        } else if m < 0.70 {
            CollectibleType::HomingSwarm // Uncommon
        } else if m < 0.84 {
            CollectibleType::RailSpike // Rare
        } else if m < 0.96 {
            CollectibleType::PlasmaLance // Rare
        } else {
            CollectibleType::VortonProjector // Epic
        }
    };

    spawn_collectible(commands, position, powerup, icon_cache);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Rarity system tests
    #[test]
    fn rarity_default_is_common() {
        assert_eq!(Rarity::default(), Rarity::Common);
    }

    #[test]
    fn rarity_glow_increases_with_tier() {
        assert!(Rarity::Common.glow_mult() < Rarity::Uncommon.glow_mult());
        assert!(Rarity::Uncommon.glow_mult() < Rarity::Rare.glow_mult());
        assert!(Rarity::Rare.glow_mult() < Rarity::Epic.glow_mult());
    }

    #[test]
    fn rarity_orbitals_increase_with_tier() {
        assert!(Rarity::Common.orbital_count() < Rarity::Uncommon.orbital_count());
        assert!(Rarity::Uncommon.orbital_count() < Rarity::Rare.orbital_count());
        assert!(Rarity::Rare.orbital_count() < Rarity::Epic.orbital_count());
    }

    #[test]
    fn rarity_epic_max_orbitals_within_cap() {
        assert!(Rarity::Epic.orbital_count() as usize <= MAX_ORBITALS_PER_COLLECTIBLE);
    }

    #[test]
    fn rarity_pulse_speed_increases_with_tier() {
        assert!(Rarity::Common.pulse_speed() < Rarity::Epic.pulse_speed());
    }

    #[test]
    fn rarity_size_increases_with_tier() {
        assert!(Rarity::Common.size_mult() <= Rarity::Uncommon.size_mult());
        assert!(Rarity::Uncommon.size_mult() <= Rarity::Rare.size_mult());
        assert!(Rarity::Rare.size_mult() <= Rarity::Epic.size_mult());
    }

    #[test]
    fn rarity_all_collectible_types_have_rarity() {
        let types = [
            CollectibleType::Credits,
            CollectibleType::ShieldBoost,
            CollectibleType::ArmorRepair,
            CollectibleType::HullRepair,
            CollectibleType::CapacitorCharge,
            CollectibleType::Nanite,
            CollectibleType::LiberationPod,
            CollectibleType::SkillPointDrop,
            CollectibleType::Overdrive,
            CollectibleType::DamageBoost,
            CollectibleType::Invulnerability,
            CollectibleType::ExtraLife,
        ];
        for ct in &types {
            let _rarity = Rarity::for_collectible(*ct); // Should not panic
        }
    }

    #[test]
    fn rarity_extra_life_is_epic() {
        assert_eq!(
            Rarity::for_collectible(CollectibleType::ExtraLife),
            Rarity::Epic
        );
    }

    #[test]
    fn rarity_credits_is_common() {
        assert_eq!(
            Rarity::for_collectible(CollectibleType::Credits),
            Rarity::Common
        );
    }

    #[test]
    fn rarity_invulnerability_is_epic() {
        assert_eq!(
            Rarity::for_collectible(CollectibleType::Invulnerability),
            Rarity::Epic
        );
    }

    // PowerupEffects tests
    #[test]
    fn powerup_effects_default_inactive() {
        let effects = PowerupEffects::default();
        assert!(!effects.is_overdrive());
        assert!(!effects.is_damage_boosted());
        assert!(!effects.is_invulnerable());
    }

    #[test]
    fn powerup_effects_default_no_multipliers() {
        let effects = PowerupEffects::default();
        assert_eq!(effects.speed_mult(), 1.0);
        assert_eq!(effects.damage_mult(), 1.0);
    }

    #[test]
    fn powerup_effects_overdrive_active() {
        let effects = PowerupEffects {
            overdrive_timer: 5.0,
            ..default()
        };
        assert!(effects.is_overdrive());
        assert_eq!(effects.speed_mult(), 1.5);
    }

    #[test]
    fn powerup_effects_damage_boost_active() {
        let effects = PowerupEffects {
            damage_boost_timer: 5.0,
            ..default()
        };
        assert!(effects.is_damage_boosted());
        assert_eq!(effects.damage_mult(), 2.0);
    }

    #[test]
    fn powerup_effects_invuln_active() {
        let effects = PowerupEffects {
            invuln_timer: 5.0,
            ..default()
        };
        assert!(effects.is_invulnerable());
    }

    // CollectiblePhysics tests
    #[test]
    fn collectible_physics_defaults() {
        let physics = CollectiblePhysics::default();
        assert!(physics.velocity.y < 0.0, "Collectibles should drift down");
        assert!(physics.lifetime > 0.0);
    }
}
