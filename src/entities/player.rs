//! Player Ship Entity
//!
//! The player-controlled ship with shield/armor/hull.

#![allow(dead_code)]

use crate::core::*;
use crate::systems::{Ability, AbilityEffects, AbilityType, EngineTrail, ManeuverState};
use bevy::prelude::*;

/// Result of taking damage - which layers absorbed how much
#[derive(Debug, Clone, Copy, Default)]
pub struct DamageResult {
    pub shield_damage: f32,
    pub armor_damage: f32,
    pub hull_damage: f32,
    pub destroyed: bool,
}

/// Marker component for the player entity
#[derive(Component, Debug)]
pub struct Player;

/// Player ship stats
#[derive(Component, Debug, Clone)]
pub struct ShipStats {
    /// Ship type ID
    pub type_id: u32,
    /// Ship display name
    pub name: String,
    /// Maximum shield HP
    pub max_shield: f32,
    /// Current shield HP
    pub shield: f32,
    /// Shield recharge rate per second
    pub shield_recharge: f32,
    /// Time until shields start recharging
    pub shield_recharge_delay: f32,
    /// Timer for shield recharge delay
    pub shield_timer: f32,
    /// Maximum armor HP
    pub max_armor: f32,
    /// Current armor HP
    pub armor: f32,
    /// Maximum hull HP
    pub max_hull: f32,
    /// Current hull HP
    pub hull: f32,
    /// Maximum capacitor (GJ)
    pub max_capacitor: f32,
    /// Current capacitor
    pub capacitor: f32,
    /// Capacitor recharge rate per second
    pub capacitor_recharge: f32,
}

impl Default for ShipStats {
    fn default() -> Self {
        Self {
            type_id: 587, // Rifter
            name: "Rifter".into(),
            max_shield: PLAYER_DEFAULT_SHIELD,
            shield: PLAYER_DEFAULT_SHIELD,
            shield_recharge: PLAYER_SHIELD_RECHARGE_RATE,
            shield_recharge_delay: PLAYER_SHIELD_RECHARGE_DELAY,
            shield_timer: 0.0,
            max_armor: PLAYER_DEFAULT_ARMOR,
            armor: PLAYER_DEFAULT_ARMOR,
            max_hull: PLAYER_DEFAULT_HULL,
            hull: PLAYER_DEFAULT_HULL,
            max_capacitor: CAP_FRIGATE,
            capacitor: CAP_FRIGATE,
            capacitor_recharge: 10.0,
        }
    }
}

impl ShipStats {
    /// Create stats for a ship by type ID
    pub fn from_type_id(type_id: u32) -> Self {
        let (name, cap, shield_mult, armor_mult) = match type_id {
            // Minmatar Frigates
            587 => ("Rifter", CAP_FRIGATE, 0.8, 1.2),
            585 => ("Slasher", CAP_FRIGATE * 0.9, 0.7, 1.0),
            586 => ("Probe", CAP_FRIGATE * 0.8, 0.6, 0.8),
            598 => ("Breacher", CAP_FRIGATE, 1.0, 0.9),
            // Amarr Frigates
            597 => ("Punisher", CAP_FRIGATE * 1.2, 0.6, 1.5),
            589 => ("Executioner", CAP_FRIGATE, 0.7, 1.1),
            // Caldari Frigates
            603 => ("Merlin", CAP_FRIGATE, 1.3, 0.7),
            602 => ("Kestrel", CAP_FRIGATE * 1.1, 1.2, 0.6),
            // Gallente Frigates
            593 => ("Tristan", CAP_FRIGATE * 1.1, 0.9, 1.1),
            594 => ("Incursus", CAP_FRIGATE, 0.8, 1.3),
            _ => ("Unknown", CAP_FRIGATE, 1.0, 1.0),
        };

        Self {
            type_id,
            name: name.into(),
            max_shield: PLAYER_DEFAULT_SHIELD * shield_mult,
            shield: PLAYER_DEFAULT_SHIELD * shield_mult,
            max_armor: PLAYER_DEFAULT_ARMOR * armor_mult,
            armor: PLAYER_DEFAULT_ARMOR * armor_mult,
            max_capacitor: cap,
            capacitor: cap,
            ..default()
        }
    }

    /// Take damage with shield/armor/hull application order
    /// Returns true if ship is destroyed (legacy compatibility)
    pub fn take_damage(&mut self, damage: f32, damage_type: DamageType) -> bool {
        self.take_damage_detailed(damage, damage_type).destroyed
    }

    /// Take damage with detailed layer information
    pub fn take_damage_detailed(&mut self, damage: f32, damage_type: DamageType) -> DamageResult {
        // Apply damage type resistances (simplified)
        let resistance = match damage_type {
            DamageType::EM => 0.0, // Shield weak to EM
            DamageType::Thermal => 0.2,
            DamageType::Kinetic => 0.4,
            DamageType::Explosive => 0.5, // Shield strong vs Explosive
        };

        let mut remaining = damage * (1.0 - resistance);
        let mut shield_damage = 0.0;
        let mut armor_damage = 0.0;
        let mut hull_damage = 0.0;

        // Damage order: Shield -> Armor -> Hull
        if self.shield > 0.0 {
            shield_damage = remaining.min(self.shield);
            self.shield -= shield_damage;
            remaining -= shield_damage;
            self.shield_timer = self.shield_recharge_delay;
        }

        if remaining > 0.0 && self.armor > 0.0 {
            armor_damage = remaining.min(self.armor);
            self.armor -= armor_damage;
            remaining -= armor_damage;
        }

        if remaining > 0.0 {
            hull_damage = remaining;
            self.hull -= hull_damage;
        }

        DamageResult {
            shield_damage,
            armor_damage,
            hull_damage,
            destroyed: self.hull <= 0.0,
        }
    }

    /// Update shield recharge
    pub fn update(&mut self, dt: f32) {
        // Shield recharge after delay
        if self.shield_timer > 0.0 {
            self.shield_timer -= dt;
        } else if self.shield < self.max_shield {
            self.shield = (self.shield + self.shield_recharge * dt).min(self.max_shield);
        }

        // Capacitor recharge
        if self.capacitor < self.max_capacitor {
            self.capacitor =
                (self.capacitor + self.capacitor_recharge * dt).min(self.max_capacitor);
        }
    }

    /// Get health percentage (combined)
    pub fn health_percent(&self) -> f32 {
        let total_max = self.max_shield + self.max_armor + self.max_hull;
        let total_current = self.shield + self.armor + self.hull;
        total_current / total_max
    }
}

/// Player movement component
#[derive(Component, Debug, Clone)]
pub struct Movement {
    /// Current velocity
    pub velocity: Vec2,
    /// Maximum speed
    pub max_speed: f32,
    /// Acceleration
    pub acceleration: f32,
    /// Deceleration (friction)
    pub friction: f32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            max_speed: PLAYER_SPEED,
            acceleration: 1500.0,
            friction: 8.0,
        }
    }
}

/// Player weapon component
#[derive(Component, Debug, Clone)]
pub struct Weapon {
    /// Weapon type
    pub weapon_type: WeaponType,
    /// Shots per second (base rate)
    pub fire_rate: f32,
    /// Time until next shot
    pub cooldown: f32,
    /// Bullet speed
    pub bullet_speed: f32,
    /// Damage per hit (base damage)
    pub damage: f32,
    /// Capacitor cost per shot
    pub cap_usage: f32,
    /// Current aim direction
    pub aim_direction: Vec2,
    /// Bullet/projectile color (overridden by ammo type for autocannons)
    pub bullet_color: Color,
    /// Current ammo type (for autocannons)
    pub ammo_type: AmmoType,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            weapon_type: WeaponType::Autocannon,
            fire_rate: PLAYER_FIRE_RATE,
            cooldown: 0.0,
            bullet_speed: PLAYER_BULLET_SPEED,
            damage: PLAYER_BULLET_DAMAGE,
            cap_usage: 5.0,
            aim_direction: Vec2::Y,                   // Up by default
            bullet_color: Color::srgb(1.0, 0.8, 0.4), // Default orange tracer
            ammo_type: AmmoType::default(),           // Sabot by default
        }
    }
}

/// Tracks the current tilt-into-turns angle (radians), layered on BaseRotation.
/// Stored in its own component so the lerp doesn't have to re-derive it from
/// the Transform quaternion — that's ambiguous when base rotation is near PI.
#[derive(Component, Default)]
pub struct CurrentTilt(pub f32);

/// Stores the ship's base rotation (sprite correction) so tilt can layer on top
#[derive(Component, Debug, Clone, Copy)]
pub struct BaseRotation(pub f32);

impl Default for BaseRotation {
    fn default() -> Self {
        Self(0.0)
    }
}

/// Respawn invincibility duration in seconds
const RESPAWN_INVINCIBILITY_SECS: f32 = 6.0;

/// Player hitbox for collision detection
#[derive(Component, Debug)]
pub struct Hitbox {
    pub radius: f32,
}

impl Default for Hitbox {
    fn default() -> Self {
        Self {
            radius: PLAYER_HITBOX_SIZE,
        }
    }
}

/// Bundle for spawning a player ship
#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub stats: ShipStats,
    pub movement: Movement,
    pub weapon: Weapon,
    pub hitbox: Hitbox,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            stats: ShipStats::default(),
            movement: Movement::default(),
            weapon: Weapon::default(),
            hitbox: Hitbox::default(),
            sprite: Sprite {
                color: COLOR_MINMATAR,
                custom_size: Some(Vec2::splat(PLAYER_SPRITE_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -250.0, LAYER_PLAYER),
        }
    }
}

/// Player plugin
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                FixedUpdate,
                (
                    player_movement,
                    player_shooting,
                    update_player_stats,
                    iframes_flash,
                )
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight)))
                    .run_if(not_last_stand),
            )
            .add_systems(
                OnExit(GameState::Playing),
                despawn_player.run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
            );
    }
}

/// Run condition: NOT in Last Stand mode
fn not_last_stand(last_stand: Option<Res<crate::games::caldari_gallente::LastStandState>>) -> bool {
    match last_stand {
        Some(ls) => !ls.active,
        None => true,
    }
}

/// Spawn player at start of gameplay
fn spawn_player(
    mut commands: Commands,
    session: Res<GameSession>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    save_data: Res<crate::core::SaveData>,
    last_stand: Option<Res<crate::games::caldari_gallente::LastStandState>>,
) {
    // Skip player spawn in Last Stand mode (titan is spawned instead)
    match &last_stand {
        Some(ls) => {
            info!("spawn_player: LastStandState exists, active={}", ls.active);
            if ls.active {
                info!("Skipping player spawn - Last Stand mode active");
                return;
            }
        }
        None => {
            info!("spawn_player: LastStandState resource not found");
        }
    }

    let ship_def = session.selected_ship();
    let faction = session.player_faction;
    let type_id = ship_def.type_id;

    // Get upgrade bonuses from persistent save data
    let bonuses = save_data.get_upgrade_bonuses();

    // Create stats from ship definition + upgrade bonuses
    let base_shield = ship_def.health * 0.4;
    let base_armor = ship_def.health * 0.35;
    let base_hull = ship_def.health * 0.25;

    let stats = ShipStats {
        type_id,
        name: ship_def.name.to_string(),
        max_shield: base_shield + bonuses.shield_bonus,
        shield: base_shield + bonuses.shield_bonus,
        shield_recharge: PLAYER_SHIELD_RECHARGE_RATE * bonuses.shield_regen_mult,
        shield_recharge_delay: PLAYER_SHIELD_RECHARGE_DELAY,
        shield_timer: 0.0,
        max_armor: base_armor + bonuses.armor_bonus,
        armor: base_armor + bonuses.armor_bonus,
        max_hull: base_hull,
        hull: base_hull,
        max_capacitor: CAP_FRIGATE + bonuses.capacitor_bonus,
        capacitor: CAP_FRIGATE + bonuses.capacitor_bonus,
        capacitor_recharge: 10.0,
    };

    // Create movement from ship speed + upgrade bonus
    let movement = Movement {
        velocity: Vec2::ZERO,
        max_speed: ship_def.speed * bonuses.speed_mult,
        acceleration: ship_def.speed * 3.0 * bonuses.speed_mult,
        friction: 8.0,
    };

    // Create weapon from ship stats + upgrade bonuses.
    // Per-hull weapon family (Hawk → rockets, Confessor → lasers, etc.)
    // overrides the faction default so invasion-era crossovers fire correctly.
    let hull_weapon = super::enemy::get_player_weapon_type(type_id, faction);
    // Gila fires missiles at 2× since it has no drone swarm in this build.
    let rate_mult = if type_id == 17713 { 2.0 } else { 1.0 };
    let weapon = Weapon {
        fire_rate: ship_def.fire_rate * bonuses.fire_rate_mult * rate_mult,
        damage: ship_def.damage * bonuses.damage_mult,
        weapon_type: hull_weapon,
        bullet_color: faction.weapon_type().bullet_color(),
        ..default()
    };

    let base_color = faction.primary_color();

    // Get ship size from class with player visibility bonus
    let base_size = ship_def.class.sprite_size();
    let player_size = base_size * crate::core::PLAYER_SIZE_BONUS;

    // Get rotation correction for this ship type (player faces UP, so no base rotation)
    let rotation = super::enemy::get_ship_rotation_correction(type_id);

    // Adjust engine trail offset based on rotation correction
    // For 180° rotated ships, the engine offset needs to be flipped
    // Engine trail matches hull faction (EDENCOM/Triglavian/Pirate override
    // the player faction for invasion-era cross-empire hulls).
    let mut engine_trail = EngineTrail::for_hull(type_id, faction);
    if (rotation - std::f32::consts::PI).abs() < 0.1 {
        // Ship is flipped 180°, flip the engine offset
        engine_trail.offset.y = -engine_trail.offset.y;
    }

    // Create ability from ship definition
    let ability_type = AbilityType::from_special(ship_def.special);

    info!("Ship ability: {:?} ({})", ability_type, ability_type.name());

    // Use sprites (2D camera compatible)
    if let Some(texture) = sprite_cache.get(type_id) {
        info!(
            "Spawning {} {} with {} engine (size: {:.0}px)",
            faction.short_name(),
            ship_def.name,
            faction.weapon_type().name(),
            player_size
        );
        // Grant respawn invincibility
        let maneuver = ManeuverState {
            invincible: true,
            invincibility_timer: RESPAWN_INVINCIBILITY_SECS,
            ..Default::default()
        };

        commands.spawn((
            Player,
            stats,
            weapon,
            Ability::new(ability_type),
            AbilityEffects::default(),
            Hitbox::default(),
            super::collectible::PowerupEffects::default(),
            maneuver,
            engine_trail,
            BaseRotation(rotation),
            Sprite {
                image: texture,
                custom_size: Some(Vec2::splat(player_size)),
                // Tint sprite toward faction color so the hull reads as the
                // player's alliance without repainting the PNG.
                color: super::enemy::ship_sprite_tint(type_id, faction),
                ..default()
            },
            Transform::from_xyz(0.0, -250.0, LAYER_PLAYER)
                .with_rotation(Quat::from_rotation_z(rotation)),
            (
                movement,
                super::items::Inventory::default(),
                super::items::EffectiveStats::neutral(),
                CurrentTilt::default(),
            ),
        ));
    } else {
        // Fallback: simple colored sprite
        warn!("No sprite for type {}, using color fallback", type_id);
        // Grant respawn invincibility
        let maneuver_fb = ManeuverState {
            invincible: true,
            invincibility_timer: RESPAWN_INVINCIBILITY_SECS,
            ..Default::default()
        };

        commands.spawn((
            Player,
            stats,
            weapon,
            Ability::new(ability_type),
            AbilityEffects::default(),
            Hitbox::default(),
            super::collectible::PowerupEffects::default(),
            maneuver_fb,
            engine_trail,
            BaseRotation(0.0),
            Sprite {
                color: base_color,
                custom_size: Some(Vec2::new(player_size * 0.85, player_size)),
                ..default()
            },
            Transform::from_xyz(0.0, -250.0, LAYER_PLAYER),
            (
                movement,
                super::items::Inventory::default(),
                super::items::EffectiveStats::neutral(),
                CurrentTilt::default(),
            ),
        ));
    }

    info!(
        "Player spawned: {} [{}] - HP:{} SPD:{} DMG:{}",
        ship_def.name,
        ship_def.class.name(),
        ship_def.health,
        ship_def.speed,
        ship_def.damage
    );
}

/// Lighten a color
fn lighten_color(color: Color, amount: f32) -> Color {
    let srgba = color.to_srgba();
    Color::srgba(
        (srgba.red + amount).min(1.0),
        (srgba.green + amount).min(1.0),
        (srgba.blue + amount).min(1.0),
        srgba.alpha,
    )
}

/// Darken a color
fn darken_color(color: Color, amount: f32) -> Color {
    let srgba = color.to_srgba();
    Color::srgba(
        (srgba.red - amount).max(0.0),
        (srgba.green - amount).max(0.0),
        (srgba.blue - amount).max(0.0),
        srgba.alpha,
    )
}

/// Max tilt angle in radians (~15 degrees)
const MAX_TILT_RADIANS: f32 = 0.26;
/// How much tilt per unit of horizontal velocity
const TILT_FACTOR: f32 = 0.001;
/// Smoothing speed for tilt interpolation (higher = snappier)
const TILT_LERP_SPEED: f32 = 10.0;

/// Player movement system
fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<crate::systems::JoystickState>,
    mut query: Query<
        (
            &mut Transform,
            &mut Movement,
            &BaseRotation,
            &ManeuverState,
            &mut CurrentTilt,
        ),
        With<Player>,
    >,
    salt_miner: Res<SaltMinerSystem>,
) {
    let Ok((mut transform, mut movement, base_rot, maneuver, mut tilt)) = query.get_single_mut()
    else {
        return;
    };

    // Get keyboard input direction
    let mut input = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        input.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        input.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        input.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        input.x += 1.0;
    }

    // Combine with joystick input
    let joy_input = joystick.movement();
    if joy_input.length_squared() > input.length_squared() {
        input = joy_input;
    }

    let dt = time.delta_secs();
    let speed_mult = salt_miner.speed_mult();

    // Apply acceleration
    if input != Vec2::ZERO {
        let input_normalized = input.normalize();
        let accel = movement.acceleration;
        movement.velocity += input_normalized * accel * dt;
    }

    // Apply friction
    let friction = movement.friction;
    movement.velocity *= 1.0 - friction * dt;

    // Clamp speed
    let max_speed = movement.max_speed * speed_mult;
    if movement.velocity.length() > max_speed {
        movement.velocity = movement.velocity.normalize() * max_speed;
    }

    // Update position
    transform.translation.x += movement.velocity.x * dt;
    transform.translation.y += movement.velocity.y * dt;

    // Clamp to screen bounds
    let half_width = SCREEN_WIDTH / 2.0 - PLAYER_SPRITE_SIZE / 2.0;
    let half_height = SCREEN_HEIGHT / 2.0 - PLAYER_SPRITE_SIZE / 2.0;
    transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
    transform.translation.y = transform.translation.y.clamp(-half_height, half_height);

    // Ship tilt into turns (skip during barrel roll — it has its own rotation)
    if !maneuver.barrel_roll_active {
        let target_tilt =
            (movement.velocity.x * TILT_FACTOR).clamp(-MAX_TILT_RADIANS, MAX_TILT_RADIANS);
        let step = (TILT_LERP_SPEED * dt).min(1.0);
        tilt.0 += (target_tilt - tilt.0) * step;
        transform.rotation = Quat::from_rotation_z(base_rot.0 + tilt.0);
    }
}

/// Player shooting system
/// Note: Python game removed capacitor - unlimited ammo, only heat matters
fn player_shooting(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<crate::systems::JoystickState>,
    mut query: Query<
        (
            &Transform,
            &mut Weapon,
            &AbilityEffects,
            Option<&super::items::EffectiveStats>,
        ),
        With<Player>,
    >,
    mut fire_events: EventWriter<PlayerFireEvent>,
    salt_miner: Res<SaltMinerSystem>,
    mut heat_system: ResMut<crate::systems::ComboHeatSystem>,
    mut screen_shake: ResMut<crate::systems::effects::ScreenShake>,
) {
    let Ok((transform, mut weapon, ability_effects, eff)) = query.get_single_mut() else {
        return;
    };
    let eff = eff
        .copied()
        .unwrap_or_else(super::items::EffectiveStats::neutral);

    let dt = time.delta_secs();

    // Update cooldown
    if weapon.cooldown > 0.0 {
        weapon.cooldown -= dt;
    }

    // Ammo switching (only for autocannons)
    if weapon.weapon_type == WeaponType::Autocannon {
        // Number keys 1-5 for direct selection
        if keyboard.just_pressed(KeyCode::Digit1) {
            weapon.ammo_type = AmmoType::Sabot;
        } else if keyboard.just_pressed(KeyCode::Digit2) {
            weapon.ammo_type = AmmoType::EMP;
        } else if keyboard.just_pressed(KeyCode::Digit3) {
            weapon.ammo_type = AmmoType::Plasma;
        } else if keyboard.just_pressed(KeyCode::Digit4) {
            weapon.ammo_type = AmmoType::Fusion;
        } else if keyboard.just_pressed(KeyCode::Digit5) {
            weapon.ammo_type = AmmoType::Barrage;
        }
        // Q/E or D-pad left/right for cycling
        else if keyboard.just_pressed(KeyCode::KeyQ) || joystick.dpad_just_left() {
            weapon.ammo_type = weapon.ammo_type.prev();
        } else if keyboard.just_pressed(KeyCode::KeyE) || joystick.dpad_just_right() {
            weapon.ammo_type = weapon.ammo_type.next();
        }
    }

    // Update aim direction from keyboard (IJKL or arrows for aiming)
    let mut aim = Vec2::ZERO;
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyI) {
        aim.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyK) {
        aim.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyJ) {
        aim.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyL) {
        aim.x += 1.0;
    }

    // Twin-stick controls: right stick aims AND fires
    // If right stick is pushed, use its direction for aiming
    let joystick_firing = if let Some(joy_aim) = joystick.aim_direction() {
        aim = joy_aim;
        true
    } else {
        false
    };

    if aim != Vec2::ZERO {
        weapon.aim_direction = aim.normalize();
    }

    // Fire if: Space pressed, right stick pushed (twin-stick), or A/X held.
    // Without right-stick aim, default to travel direction / up.
    let face_button_fire = joystick.buttons[0] || joystick.buttons[2];
    if face_button_fire && aim == Vec2::ZERO {
        aim = Vec2::new(0.0, 1.0);
        weapon.aim_direction = aim;
    }
    let fire_pressed = keyboard.pressed(KeyCode::Space) || joystick_firing || face_button_fire;

    if fire_pressed && weapon.cooldown <= 0.0 {
        // Track heat (doesn't block firing, just affects fire rate)
        heat_system.on_fire();

        // Calculate fire rate with modifiers:
        // - Base fire rate
        // - Ammo type modifier (Fusion slower, Barrage faster)
        // - Salt Miner bonus (1.5x when active)
        // - Heat penalty (0.7x when overheated)
        let ammo_mult = weapon.ammo_type.fire_rate_mult();
        let salt_miner_mult = if salt_miner.is_active { 1.5 } else { 1.0 };
        let heat_mult = heat_system.fire_rate_mult();
        let fire_rate =
            weapon.fire_rate * ammo_mult * salt_miner_mult * heat_mult * eff.fire_rate_mult;
        weapon.cooldown = 1.0 / fire_rate.max(0.1);

        // Calculate burst parameters from ability effects + inventory mods
        // extra_projectiles: 2 = triple shot (Rocket Barrage), 3 = quad shot (Salvo)
        // inventory mods (Scatter/Homing) add on top
        // Per-hull baseline bullet pattern so each weapon family plays
        // distinctly even before mods (autocannons twin-tracer, drones
        // spread wide).
        let (hull_burst_bonus, hull_spread_bonus) = match weapon.weapon_type {
            WeaponType::Autocannon => (1u32, 6_f32.to_radians()),
            WeaponType::Drone => (2u32, 24_f32.to_radians()),
            _ => (0u32, 0.0),
        };
        let burst_count =
            1 + ability_effects.extra_projectiles + eff.extra_projectiles + hull_burst_bonus;
        let spread_angle = if burst_count > 1 {
            let ability_spread = if ability_effects.extra_projectiles > 0 {
                ((1 + ability_effects.extra_projectiles) as f32 * 10.0).to_radians()
            } else {
                0.0
            };
            ability_spread
                + eff.spread_radians * (burst_count.saturating_sub(1) as f32)
                + hull_spread_bonus
        } else {
            0.0
        };

        // Use ammo color for autocannons, weapon color for others
        let bullet_color = if weapon.weapon_type == WeaponType::Autocannon {
            weapon.ammo_type.color()
        } else {
            weapon.bullet_color
        };

        // Send fire event (damage scaled by inventory mods + crit/pierce/homing/burn)
        let crit_chance_override = if eff.crit_chance > 0.0 {
            Some(eff.crit_chance)
        } else {
            None
        };
        let crit_mult_override = if eff.crit_damage_mult > 1.0 {
            Some(eff.crit_damage_mult)
        } else {
            None
        };
        fire_events.send(PlayerFireEvent {
            position: transform.translation.truncate(),
            direction: weapon.aim_direction,
            weapon_type: weapon.weapon_type,
            bullet_color,
            damage: weapon.damage * eff.damage_mult,
            burst_count,
            spread_angle,
            ammo_type: weapon.ammo_type,
            crit_chance_override,
            crit_mult_override,
            pierce: eff.pierce,
            homing: eff.homing,
            burn_dps: eff.burn_dps,
            chain_targets: eff.chain_targets,
        });

        // Subtle recoil shake on fire
        screen_shake.trigger(1.0, 0.03);
    }
}

/// Update player stats (shield recharge, etc)
fn update_player_stats(time: Res<Time>, mut query: Query<&mut ShipStats, With<Player>>) {
    let Ok(mut stats) = query.get_single_mut() else {
        return;
    };

    stats.update(time.delta_secs());
}

/// Visual feedback during any i-frames: flash sprite alpha
fn iframes_flash(time: Res<Time>, mut query: Query<(&ManeuverState, &mut Sprite), With<Player>>) {
    let Ok((maneuver, mut sprite)) = query.get_single_mut() else {
        return;
    };

    if maneuver.invincible {
        // Toggle alpha between 1.0 and 0.3 every 0.1 seconds
        let t = time.elapsed_secs();
        let alpha = if (t * 10.0) as i32 % 2 == 0 { 1.0 } else { 0.3 };
        sprite.color = sprite.color.with_alpha(alpha);
    } else {
        // Ensure full opacity when not invincible
        sprite.color = sprite.color.with_alpha(1.0);
    }
}

/// Despawn player when leaving gameplay
fn despawn_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
