//! Projectile Entities
//!
//! Player bullets, enemy bullets, missiles, drones.

#![allow(dead_code)]

use crate::core::*;
use crate::systems::effects::BulletTrail;
use bevy::prelude::*;

/// Marker for player projectiles
#[derive(Component, Debug)]
pub struct PlayerProjectile;

/// Marker for enemy projectiles
#[derive(Component, Debug)]
pub struct EnemyProjectile;

/// Seeking/homing projectile - tracks nearest enemy
#[derive(Component, Debug)]
pub struct SeekingProjectile {
    /// Turn rate in radians per second
    pub turn_rate: f32,
    /// Maximum range to acquire target
    pub acquire_range: f32,
}

/// Projectile survives hits — decrements on each hit, despawns when 0.
#[derive(Component, Debug, Clone, Copy)]
pub struct Pierce(pub u32);

/// On hit, apply burn DoT to the target with this dps for 3 seconds.
#[derive(Component, Debug, Clone, Copy)]
pub struct BurnOnHit(pub f32);

/// Active burn effect applied to an enemy.
#[derive(Component, Debug, Clone)]
pub struct BurnStatus {
    pub dps: f32,
    pub remaining: f32,
}

/// On hit, chain to this many additional enemies. EDENCOM / Vorton only.
#[derive(Component, Debug, Clone, Copy)]
pub struct ChainOnHit(pub u32);

/// Visual bolt drawn between chain targets; fades out.
#[derive(Component, Debug)]
pub struct ChainBolt {
    pub life: f32,
    pub max: f32,
}

/// Projectile physics
#[derive(Component, Debug, Clone)]
pub struct ProjectilePhysics {
    /// Current velocity
    pub velocity: Vec2,
    /// Lifetime remaining
    pub lifetime: f32,
}

/// Projectile damage info
#[derive(Component, Debug, Clone)]
pub struct ProjectileDamage {
    /// Damage amount (base, before ammo multipliers)
    pub damage: f32,
    /// Damage type
    pub damage_type: DamageType,
    /// Critical hit chance (0.0 - 1.0)
    pub crit_chance: f32,
    /// Critical hit damage multiplier
    pub crit_multiplier: f32,
    /// Ammo type (for damage multipliers vs shield/armor)
    pub ammo_type: AmmoType,
}

impl Default for ProjectileDamage {
    fn default() -> Self {
        Self {
            damage: 10.0,
            damage_type: DamageType::Kinetic,
            crit_chance: 0.1,     // 10% base crit chance
            crit_multiplier: 1.5, // 1.5x crit damage
            ammo_type: AmmoType::default(),
        }
    }
}

/// Bundle for player projectile
#[derive(Bundle)]
pub struct PlayerProjectileBundle {
    pub marker: PlayerProjectile,
    pub physics: ProjectilePhysics,
    pub damage: ProjectileDamage,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl Default for PlayerProjectileBundle {
    fn default() -> Self {
        Self {
            marker: PlayerProjectile,
            physics: ProjectilePhysics {
                velocity: Vec2::Y * PLAYER_BULLET_SPEED,
                lifetime: 2.0,
            },
            damage: ProjectileDamage {
                damage: PLAYER_BULLET_DAMAGE,
                damage_type: DamageType::Kinetic,
                crit_chance: 0.1, // 10% crit for autocannons
                crit_multiplier: 1.5,
                ammo_type: AmmoType::default(),
            },
            sprite: Sprite {
                color: Color::srgb(1.0, 0.9, 0.3),
                custom_size: Some(Vec2::new(4.0, 12.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, LAYER_PLAYER_BULLETS),
        }
    }
}

/// Bundle for enemy projectile
#[derive(Bundle)]
pub struct EnemyProjectileBundle {
    pub marker: EnemyProjectile,
    pub physics: ProjectilePhysics,
    pub damage: ProjectileDamage,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl Default for EnemyProjectileBundle {
    fn default() -> Self {
        Self {
            marker: EnemyProjectile,
            physics: ProjectilePhysics {
                velocity: Vec2::NEG_Y * ENEMY_BULLET_SPEED,
                lifetime: 3.0,
            },
            damage: ProjectileDamage {
                damage: 10.0,
                damage_type: DamageType::EM,
                crit_chance: 0.05,              // 5% crit for enemies (lower)
                crit_multiplier: 1.25,          // 1.25x crit for enemies
                ammo_type: AmmoType::default(), // Enemies don't use ammo types
            },
            sprite: Sprite {
                color: Color::srgb(1.0, 0.3, 0.3),
                custom_size: Some(Vec2::new(6.0, 6.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, LAYER_ENEMY_BULLETS),
        }
    }
}

/// Projectile plugin
pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DisintegratorHeat>().add_systems(
            FixedUpdate,
            (
                spawn_player_projectiles,
                seeking_projectile_update,
                projectile_update,
                tick_disintegrator_heat,
                player_disintegrator_beam_update,
                burn_tick,
            )
                .chain()
                .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
        );
    }
}

/// Tick BurnStatus on enemies, apply damage, remove when expired.
fn burn_tick(
    time: Res<Time>,
    mut commands: Commands,
    mut q: Query<
        (Entity, &mut BurnStatus, &mut super::enemy::EnemyStats),
        With<super::enemy::Enemy>,
    >,
) {
    let dt = time.delta_secs();
    for (e, mut b, mut stats) in q.iter_mut() {
        stats.health -= b.dps * dt;
        b.remaining -= dt;
        if b.remaining <= 0.0 {
            commands.entity(e).remove::<BurnStatus>();
        }
    }
}

/// Tracks sustained-fire heat + beam activity for the player's Disintegrator.
/// One persistent beam sweeps from the player's muzzle in the aim direction;
/// this resource is the bridge between the fire event (which refreshes the
/// beam's "on" timer + heat) and the render/damage system.
#[derive(Resource, Default)]
pub struct DisintegratorHeat {
    /// Ramp heat 0.0 → 1.0 (width, color, damage scale with this).
    pub level: f32,
    /// Seconds since last disintegrator fire; drives decay.
    pub idle: f32,
    /// Seconds the beam should stay "on"; refreshed by each fire event so
    /// continuous trigger-hold keeps the beam alive.
    pub active_timer: f32,
    /// Latest aim direction from the fire event.
    pub aim_dir: Vec2,
    /// Position where the beam should originate (player muzzle).
    pub origin: Vec2,
    /// Active damage this frame (base × heat ramp), for the beam system.
    pub damage_per_fire: f32,
}

impl DisintegratorHeat {
    pub fn on_fire(&mut self, origin: Vec2, aim: Vec2, damage: f32) {
        self.level = (self.level + 0.14).min(1.0);
        self.idle = 0.0;
        // Keep the beam alive for a hair longer than the fire interval so
        // gaps in cadence don't visibly break the sweep.
        self.active_timer = 0.16;
        self.aim_dir = aim;
        self.origin = origin;
        self.damage_per_fire = damage;
    }
}

/// Decay disintegrator heat + beam-active timer over time.
pub fn tick_disintegrator_heat(time: Res<Time>, mut heat: ResMut<DisintegratorHeat>) {
    let dt = time.delta_secs();
    heat.idle += dt;
    heat.active_timer = (heat.active_timer - dt).max(0.0);
    if heat.idle > 0.35 {
        heat.level = (heat.level - 1.8 * dt).max(0.0);
    }
}

/// Marker for the persistent player disintegrator beam sprite.
#[derive(Component)]
pub struct PlayerDisintegratorBeam;

/// Draw + damage the persistent beam. Runs every frame: if beam is active,
/// position a long rectangle sprite from the player's muzzle in the aim
/// direction, size/color driven by heat, and damage any enemy inside its
/// swept rect (per-frame damage × ramp × dt for continuous contact).
pub fn player_disintegrator_beam_update(
    time: Res<Time>,
    mut commands: Commands,
    heat: Res<DisintegratorHeat>,
    mut beam_query: Query<
        (Entity, &mut Transform, &mut Sprite, &mut Visibility),
        With<PlayerDisintegratorBeam>,
    >,
    mut enemy_query: Query<
        (Entity, &Transform, &mut super::enemy::EnemyStats),
        (With<super::Enemy>, Without<PlayerDisintegratorBeam>),
    >,
    mut destroy_events: EventWriter<crate::core::EnemyDestroyedEvent>,
    mut explosion_events: EventWriter<crate::core::ExplosionEvent>,
) {
    let active = heat.active_timer > 0.0 && heat.aim_dir.length_squared() > 0.01;

    // Ensure a single beam entity exists.
    let beam_exists = beam_query.iter().next().is_some();
    if active && !beam_exists {
        let beam_color = Color::srgb(1.0, 0.4, 0.15);
        commands.spawn((
            PlayerDisintegratorBeam,
            Sprite {
                color: beam_color,
                custom_size: Some(Vec2::new(6.0, 600.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, crate::core::LAYER_PLAYER_BULLETS + 0.05),
            Visibility::Hidden,
        ));
        return; // spawn takes a frame; update next pass.
    }
    if !active {
        for (e, _, _, mut vis) in beam_query.iter_mut() {
            *vis = Visibility::Hidden;
            // Despawn fully if beam has been idle long enough.
            if heat.idle > 0.5 {
                commands.entity(e).despawn_recursive();
            }
        }
        return;
    }

    let dt = time.delta_secs();
    let h = heat.level;
    let beam_width = 5.0 + 9.0 * h;
    let beam_length = 620.0;
    let color = Color::srgb(1.0, 0.35 + 0.55 * h, 0.1 + 0.75 * h);
    let aim = heat.aim_dir.normalize_or_zero();
    let origin = heat.origin;
    let center = origin + aim * (beam_length * 0.5);
    let angle = aim.y.atan2(aim.x) - std::f32::consts::FRAC_PI_2;

    for (_, mut tf, mut sprite, mut vis) in beam_query.iter_mut() {
        *vis = Visibility::Visible;
        tf.translation.x = center.x;
        tf.translation.y = center.y;
        tf.rotation = Quat::from_rotation_z(angle);
        sprite.color = color;
        sprite.custom_size = Some(Vec2::new(beam_width, beam_length));
    }

    // Damage enemies whose distance from the beam line < beam_width/2 AND
    // projected onto the beam axis is within 0..beam_length.
    // Ramp scales 1.0 → 2.5× DPS with heat — full white-hot is 2.5×
    // effective DPS compared to a cold first-frame shot.
    let dmg_per_sec = heat.damage_per_fire * (1.0 + 1.5 * h);
    let half_width = beam_width * 0.5 + 14.0; // hitbox margin
    for (enemy_entity, enemy_tf, mut enemy_stats) in enemy_query.iter_mut() {
        let enemy_pos = enemy_tf.translation.truncate();
        let to_enemy = enemy_pos - origin;
        let along = to_enemy.dot(aim);
        if !(0.0..=beam_length).contains(&along) {
            continue;
        }
        let perp = (to_enemy - aim * along).length();
        if perp > half_width {
            continue;
        }
        let damage = dmg_per_sec * dt;
        enemy_stats.health -= damage;
        if enemy_stats.health <= 0.0 {
            destroy_events.send(crate::core::EnemyDestroyedEvent {
                enemy: enemy_entity,
                position: enemy_pos,
                enemy_type: enemy_stats.name.clone(),
                score_value: enemy_stats.score_value,
                was_boss: enemy_stats.is_boss,
                liberation_value: enemy_stats.liberation_value,
                type_id: enemy_stats.type_id,
            });
            explosion_events.send(crate::core::ExplosionEvent {
                position: enemy_pos,
                size: if enemy_stats.is_boss {
                    crate::core::ExplosionSize::Massive
                } else {
                    crate::core::ExplosionSize::Small
                },
                color: Color::srgb(1.0, 0.5, 0.2),
            });
            commands.entity(enemy_entity).despawn_recursive();
        }
    }
}

/// Spawn player projectiles on fire event
fn spawn_player_projectiles(
    mut commands: Commands,
    mut fire_events: EventReader<PlayerFireEvent>,
    salt_miner: Res<SaltMinerSystem>,
    mut disint_heat: ResMut<DisintegratorHeat>,
) {
    for event in fire_events.read() {
        let damage_mult = salt_miner.damage_mult();

        // Weapon-family-colored muzzle flash at fire origin.
        crate::systems::effects::spawn_muzzle_flash(
            &mut commands,
            event.position,
            event.direction,
            event.weapon_type,
        );

        // Determine damage type from weapon
        let damage_type = match event.weapon_type {
            WeaponType::Autocannon | WeaponType::Artillery => DamageType::Kinetic,
            WeaponType::Laser => DamageType::EM,
            WeaponType::Railgun => DamageType::Kinetic,
            WeaponType::MissileLauncher => DamageType::Explosive,
            WeaponType::Drone => DamageType::Thermal,
            WeaponType::Disintegrator => DamageType::Thermal, // Triglavian entropic damage
            WeaponType::Vorton => DamageType::EM,             // EDENCOM chain lightning
        };

        // Use event's bullet color, or purple if salt miner active
        let color = if salt_miner.is_active {
            Color::srgb(1.0, 0.2, 0.8)
        } else {
            event.bullet_color
        };

        // Check if this is a seeking missile (Kestrel/Caldari missile launcher)
        let is_missile = event.weapon_type == WeaponType::MissileLauncher;
        // Disintegrator fires a long orange-red piercing beam (entropic).
        let is_beam = event.weapon_type == WeaponType::Disintegrator;
        // Vorton projector fires an instantaneous chain-seeking bolt — not a
        // traveling bullet; it locks onto the nearest enemy and jumps.
        let is_arc = event.weapon_type == WeaponType::Vorton;

        // Calculate projectile spread for burst fire
        let burst_count = event.burst_count.max(1);
        let spread_angle = event.spread_angle;

        for i in 0..burst_count {
            // Calculate direction offset for this projectile
            let angle_offset = if burst_count > 1 {
                // Distribute evenly across spread angle, centered
                let spread_step = spread_angle / (burst_count - 1) as f32;
                -spread_angle / 2.0 + spread_step * i as f32
            } else {
                0.0
            };

            // Rotate direction by angle offset
            let base_angle = event.direction.y.atan2(event.direction.x);
            let proj_angle = base_angle + angle_offset;
            let direction = Vec2::new(proj_angle.cos(), proj_angle.sin());

            // Small position offset for visual spread
            let pos_offset = Vec2::new((i as f32 - (burst_count - 1) as f32 / 2.0) * 5.0, 0.0);
            let spawn_pos = event.position + pos_offset;

            if is_missile {
                // Seeking missile - larger, slower, homes on enemies, more damage
                let missile_velocity = direction * (PLAYER_BULLET_SPEED * 0.7);
                let missile_damage = event.damage * damage_mult * 1.25;

                commands.spawn((
                    PlayerProjectile,
                    SeekingProjectile {
                        turn_rate: 4.0,
                        acquire_range: 400.0,
                    },
                    ProjectilePhysics {
                        velocity: missile_velocity,
                        lifetime: 3.0,
                    },
                    ProjectileDamage {
                        damage: missile_damage,
                        damage_type,
                        crit_chance: 0.15,
                        crit_multiplier: 1.75,
                        ammo_type: event.ammo_type,
                    },
                    BulletTrail::new(Color::srgb(1.0, 0.6, 0.2)),
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(6.0, 14.0)),
                        ..default()
                    },
                    Transform::from_xyz(spawn_pos.x, spawn_pos.y, LAYER_PLAYER_BULLETS),
                ));
            } else if is_arc {
                // Vorton — instantaneous chain-lightning arc. Fire a very
                // fast, homing, short-lived bolt with large pierce so it
                // reliably connects. The chain-on-hit then zaps 3 more.
                let arc_color = Color::srgb(0.6, 0.85, 1.0);
                let velocity = direction * (PLAYER_BULLET_SPEED * 3.0);
                let mut entity = commands.spawn((
                    PlayerProjectile,
                    ProjectilePhysics {
                        velocity,
                        lifetime: 0.35,
                    },
                    ProjectileDamage {
                        damage: event.damage * damage_mult * 1.1,
                        damage_type,
                        crit_chance: event.crit_chance_override.unwrap_or(0.1),
                        crit_multiplier: event.crit_mult_override.unwrap_or(1.5),
                        ammo_type: event.ammo_type,
                    },
                    BulletTrail::new(arc_color.with_alpha(0.85)),
                    Sprite {
                        color: arc_color,
                        custom_size: Some(Vec2::new(6.0, 18.0)),
                        ..default()
                    },
                    Transform::from_xyz(spawn_pos.x, spawn_pos.y, LAYER_PLAYER_BULLETS),
                    SeekingProjectile {
                        turn_rate: 18.0,
                        acquire_range: 700.0,
                    },
                ));
                // Guarantee the chain fires by attaching ChainOnHit up front;
                // baseline 3 + mod stacks (event.chain_targets already adds).
                entity.insert(ChainOnHit(3 + event.chain_targets));
                entity.insert(Pierce(1));
            } else if is_beam {
                // Disintegrator fires no discrete projectile — instead the
                // fire event refreshes the persistent beam's active timer +
                // heat. The beam update system draws the sweeping beam and
                // applies per-second damage to anything in its line.
                // Scale by ~7× so continuous-beam DPS matches what the
                // per-shot version delivered via fire_rate × damage.
                let beam_dps = event.damage * damage_mult * 7.5;
                disint_heat.on_fire(spawn_pos, direction, beam_dps);
            } else {
                // Standard projectile with bullet trail — mod overrides layered on.
                let velocity = direction * PLAYER_BULLET_SPEED;
                let mut entity = commands.spawn((
                    PlayerProjectile,
                    ProjectilePhysics {
                        velocity,
                        lifetime: 2.0,
                    },
                    ProjectileDamage {
                        damage: event.damage * damage_mult,
                        damage_type,
                        crit_chance: event.crit_chance_override.unwrap_or(0.1),
                        crit_multiplier: event.crit_mult_override.unwrap_or(1.5),
                        ammo_type: event.ammo_type,
                    },
                    BulletTrail::new(color.with_alpha(0.5)),
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(4.0, 12.0)),
                        ..default()
                    },
                    Transform::from_xyz(spawn_pos.x, spawn_pos.y, LAYER_PLAYER_BULLETS),
                ));
                if event.pierce > 0 {
                    entity.insert(Pierce(event.pierce));
                }
                if event.homing > 0.1 {
                    entity.insert(SeekingProjectile {
                        turn_rate: 2.0 + event.homing * 6.0,
                        acquire_range: 400.0,
                    });
                }
                if event.burn_dps > 0.0 {
                    entity.insert(BurnOnHit(event.burn_dps));
                }
                // Baseline chain for Vorton hulls even without any mod stacks —
                // chain lightning is the weapon's identity.
                let base_chain = if matches!(event.weapon_type, WeaponType::Vorton) {
                    3
                } else {
                    0
                };
                let total_chain = event.chain_targets + base_chain;
                if total_chain > 0 {
                    entity.insert(ChainOnHit(total_chain));
                }
            }
        }
    }
}

/// Seeking projectile homing behavior - finds nearest enemy and turns toward it
fn seeking_projectile_update(
    time: Res<Time>,
    enemy_query: Query<&Transform, With<super::Enemy>>,
    mut seeking_query: Query<
        (&Transform, &mut ProjectilePhysics, &SeekingProjectile),
        With<PlayerProjectile>,
    >,
) {
    let dt = time.delta_secs();

    for (transform, mut physics, seeking) in seeking_query.iter_mut() {
        let missile_pos = transform.translation.truncate();

        // Find nearest enemy within range
        let mut nearest_enemy: Option<Vec2> = None;
        let mut nearest_dist = seeking.acquire_range;

        for enemy_transform in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.truncate();
            let dist = (enemy_pos - missile_pos).length();

            if dist < nearest_dist {
                nearest_dist = dist;
                nearest_enemy = Some(enemy_pos);
            }
        }

        // If we found a target, turn toward it
        if let Some(target_pos) = nearest_enemy {
            let current_dir = physics.velocity.normalize_or_zero();
            let target_dir = (target_pos - missile_pos).normalize_or_zero();

            // Calculate angle difference
            let current_angle = current_dir.y.atan2(current_dir.x);
            let target_angle = target_dir.y.atan2(target_dir.x);
            let mut angle_diff = target_angle - current_angle;

            // Normalize to -PI..PI
            while angle_diff > std::f32::consts::PI {
                angle_diff -= std::f32::consts::TAU;
            }
            while angle_diff < -std::f32::consts::PI {
                angle_diff += std::f32::consts::TAU;
            }

            // Limit turn rate
            let max_turn = seeking.turn_rate * dt;
            let turn = angle_diff.clamp(-max_turn, max_turn);

            // Apply turn
            let new_angle = current_angle + turn;
            let speed = physics.velocity.length();
            physics.velocity = Vec2::new(new_angle.cos(), new_angle.sin()) * speed;
        }
    }
}

/// Combined projectile update: movement, lifetime, and bounds in one pass
/// This reduces from 3 iterations over all projectiles to just 1.
fn projectile_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ProjectilePhysics)>,
) {
    let dt = time.delta_secs();

    // Precompute bounds (with margin for off-screen cleanup)
    const MARGIN: f32 = 50.0;
    let half_w = SCREEN_WIDTH / 2.0 + MARGIN;
    let half_h = SCREEN_HEIGHT / 2.0 + MARGIN;

    for (entity, mut transform, mut physics) in query.iter_mut() {
        // Update lifetime
        physics.lifetime -= dt;

        // Move projectile
        transform.translation.x += physics.velocity.x * dt;
        transform.translation.y += physics.velocity.y * dt;

        // Check lifetime and bounds in one go
        let pos = transform.translation;
        if physics.lifetime <= 0.0 || pos.x.abs() > half_w || pos.y.abs() > half_h {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Spawn enemy projectile helper
pub fn spawn_enemy_projectile(
    commands: &mut Commands,
    position: Vec2,
    direction: Vec2,
    damage: f32,
    speed: f32,
) {
    let velocity = direction.normalize_or_zero() * speed;
    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

    commands.spawn(EnemyProjectileBundle {
        physics: ProjectilePhysics {
            velocity,
            lifetime: 5.0,
        },
        damage: ProjectileDamage {
            damage,
            damage_type: DamageType::EM,
            crit_chance: 0.05, // 5% crit for enemies
            crit_multiplier: 1.25,
            ammo_type: AmmoType::default(),
        },
        transform: Transform::from_xyz(position.x, position.y, LAYER_ENEMY_BULLETS)
            .with_rotation(Quat::from_rotation_z(angle)),
        ..default()
    });
}

/// Spawn enemy projectile with faction-appropriate weapon visuals
pub fn spawn_enemy_projectile_typed(
    commands: &mut Commands,
    position: Vec2,
    direction: Vec2,
    damage: f32,
    speed: f32,
    weapon_type: WeaponType,
) {
    let velocity = direction.normalize_or_zero() * speed;
    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

    // Get damage type and color based on weapon type
    let (damage_type, color, size) = match weapon_type {
        WeaponType::Laser => (
            DamageType::EM,
            Color::srgb(1.0, 0.2, 0.2), // Amarr red laser
            Vec2::new(4.0, 22.0),       // Beam shape — thicker, longer for readability
        ),
        WeaponType::Railgun => (
            DamageType::Kinetic,
            Color::srgb(0.4, 0.8, 1.0), // Caldari cyan
            Vec2::new(5.0, 14.0),       // Fast bolt — larger so it reads at speed
        ),
        WeaponType::MissileLauncher => (
            DamageType::Explosive,
            Color::srgb(1.0, 0.5, 0.15), // Orange missile
            Vec2::new(8.0, 12.0),        // Larger missile — must be seen to be dodged
        ),
        WeaponType::Drone => (
            DamageType::Thermal,
            Color::srgb(0.5, 1.0, 0.4), // Gallente green
            Vec2::new(7.0, 7.0),        // Round drone shot — bigger for visibility
        ),
        WeaponType::Autocannon | WeaponType::Artillery => (
            DamageType::Kinetic,
            Color::srgb(1.0, 0.8, 0.3), // Minmatar yellow/orange
            Vec2::new(5.0, 11.0),       // Bullet shape
        ),
        WeaponType::Disintegrator => (
            DamageType::Thermal,
            Color::srgb(0.9, 0.3, 0.0), // Triglavian orange-red beam
            Vec2::new(4.0, 26.0),       // Long beam shape
        ),
        WeaponType::Vorton => (
            DamageType::EM,
            Color::srgb(0.6, 0.7, 1.0), // EDENCOM blue-white arc
            Vec2::new(7.0, 16.0),       // Wide arc shape
        ),
    };

    commands.spawn((
        EnemyProjectile,
        ProjectilePhysics {
            velocity,
            lifetime: 5.0,
        },
        ProjectileDamage {
            damage,
            damage_type,
            crit_chance: 0.05, // 5% crit for enemies
            crit_multiplier: 1.25,
            ammo_type: AmmoType::default(),
        },
        BulletTrail::new(color.with_alpha(0.4)),
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, LAYER_ENEMY_BULLETS)
            .with_rotation(Quat::from_rotation_z(angle)),
    ));
}
