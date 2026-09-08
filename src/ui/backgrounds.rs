//! Background System
//!
//! Loads and displays background images for menus and gameplay.
//! Features a dynamic parallax starfield during gameplay.
//! Includes distant ship traffic for atmosphere (ported from Python version).

use crate::core::*;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;

/// Background plugin
pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        for destination in crate::core::NON_COMBAT_STATES {
            app.add_systems(OnEnter(destination), despawn_starfield);
        }
        app.init_resource::<BackgroundAssets>()
            .init_resource::<StarfieldConfig>()
            .init_resource::<BackgroundShipConfig>()
            .init_resource::<BackgroundShipSpawnTimer>()
            .add_systems(Startup, load_backgrounds)
            .add_systems(OnEnter(GameState::Loading), spawn_title_background)
            .add_systems(OnEnter(GameState::MainMenu), spawn_title_background)
            .add_systems(OnEnter(GameState::DifficultySelect), spawn_title_background)
            .add_systems(OnEnter(GameState::ShipSelect), spawn_title_background)
            .add_systems(OnExit(GameState::MainMenu), despawn_menu_background)
            .add_systems(OnExit(GameState::DifficultySelect), despawn_menu_background)
            .add_systems(OnExit(GameState::ShipSelect), despawn_menu_background)
            .add_systems(OnExit(GameState::Loading), despawn_menu_background)
            // Starfield for gameplay
            .add_systems(
                OnEnter(GameState::Playing),
                spawn_starfield.run_if(not_resuming_gameplay),
            )
            .add_systems(
                Update,
                (
                    update_starfield,
                    update_background_ship_spawning,
                    update_background_ships,
                    update_distant_battle,
                )
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
            )
            .add_systems(
                OnExit(GameState::Paused),
                despawn_starfield.run_if(not_resuming_gameplay),
            );
    }
}

/// Background image assets
#[derive(Resource, Default)]
pub struct BackgroundAssets {
    pub title: Option<Handle<Image>>,
}

/// Marker for menu background sprite
#[derive(Component)]
pub struct MenuBackground;

/// Configuration for the starfield
#[derive(Resource)]
pub struct StarfieldConfig {
    /// Number of stars per layer
    pub stars_per_layer: usize,
    /// Number of parallax layers (far to near)
    pub layers: usize,
    /// Base scroll speed (pixels per second)
    pub base_speed: f32,
    /// Speed multiplier per layer (near layers move faster)
    pub layer_speed_mult: f32,
}

impl Default for StarfieldConfig {
    fn default() -> Self {
        Self {
            stars_per_layer: 80,
            layers: 3,
            base_speed: 30.0,
            layer_speed_mult: 1.8,
        }
    }
}

/// A star in the background starfield
#[derive(Component)]
pub struct BackgroundStar {
    /// Parallax layer (0 = farthest, higher = nearer) - used at spawn time
    #[allow(dead_code)]
    pub layer: usize,
    /// Scroll speed for this star
    pub speed: f32,
    /// Base brightness (0.0-1.0)
    pub brightness: f32,
    /// Twinkle phase offset
    pub twinkle_phase: f32,
}

/// Marker for the entire starfield (for cleanup)
#[derive(Component)]
pub struct Starfield;

// =============================================================================
// BACKGROUND SHIPS - Distant ship traffic for atmosphere
// =============================================================================

/// Configuration for background ship spawning
#[derive(Resource)]
pub struct BackgroundShipConfig {
    /// Maximum ships on screen at once
    pub max_ships: usize,
    /// Frames between spawn attempts
    pub spawn_interval: u32,
    /// Chance to spawn when interval reached (0.0-1.0)
    pub spawn_chance: f32,
}

impl Default for BackgroundShipConfig {
    fn default() -> Self {
        Self {
            max_ships: 8,
            spawn_interval: 60, // ~2 seconds at 60fps
            spawn_chance: 0.85,
        }
    }
}

/// Spawn timer for background ships
#[derive(Resource, Default)]
pub struct BackgroundShipSpawnTimer {
    pub elapsed: f32,
}

/// Ship class for background ships (affects size)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BackgroundShipClass {
    Frigate,
    Cruiser,
    Battleship,
}

impl BackgroundShipClass {
    /// Base size in pixels
    pub fn base_size(&self) -> f32 {
        match self {
            BackgroundShipClass::Frigate => 25.0,
            BackgroundShipClass::Cruiser => 45.0,
            BackgroundShipClass::Battleship => 70.0,
        }
    }

    /// Random selection weighted toward frigates
    pub fn random() -> Self {
        let roll = fastrand::f32();
        if roll < 0.50 {
            BackgroundShipClass::Frigate
        } else if roll < 0.85 {
            BackgroundShipClass::Cruiser
        } else {
            BackgroundShipClass::Battleship
        }
    }
}

/// Faction for background ships (determines direction and color)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BackgroundShipFaction {
    /// Allies - fly left to right (retreating/flanking)
    Minmatar,
    /// Enemies - fly right to left (attacking)
    Amarr,
}

impl BackgroundShipFaction {
    pub fn random() -> Self {
        if fastrand::bool() {
            BackgroundShipFaction::Minmatar
        } else {
            BackgroundShipFaction::Amarr
        }
    }

    /// Hull color for this faction
    pub fn hull_color(&self) -> Color {
        match self {
            BackgroundShipFaction::Minmatar => {
                // Rust orange variants
                let variants = [
                    Color::srgb(0.71, 0.39, 0.24), // Rust orange
                    Color::srgb(0.59, 0.31, 0.20), // Dark rust
                    Color::srgb(0.78, 0.47, 0.27), // Light rust
                ];
                variants[fastrand::usize(..variants.len())]
            }
            BackgroundShipFaction::Amarr => {
                // Gold variants
                let variants = [
                    Color::srgb(0.78, 0.67, 0.31), // Gold
                    Color::srgb(0.71, 0.59, 0.24), // Dark gold
                    Color::srgb(0.86, 0.75, 0.39), // Bright gold
                ];
                variants[fastrand::usize(..variants.len())]
            }
        }
    }

    /// Engine glow color (for future engine trail effects)
    #[allow(dead_code)]
    pub fn engine_color(&self) -> Color {
        match self {
            BackgroundShipFaction::Minmatar => Color::srgba(1.0, 0.59, 0.2, 0.9),
            BackgroundShipFaction::Amarr => Color::srgba(1.0, 0.86, 0.4, 0.9),
        }
    }

    /// Does this faction fly left-to-right?
    pub fn flies_right(&self) -> bool {
        matches!(self, BackgroundShipFaction::Minmatar)
    }
}

/// A distant ship silhouette flying in the background
#[derive(Component)]
pub struct BackgroundShip {
    /// Faction determines flight direction and colors (stored for future queries)
    #[allow(dead_code)]
    pub faction: BackgroundShipFaction,
    /// Ship class (for future silhouette rendering)
    #[allow(dead_code)]
    pub ship_class: BackgroundShipClass,
    /// Simulated distance (0.3 = far, 0.8 = close) - used at spawn time
    #[allow(dead_code)]
    pub distance: f32,
    /// Actual rendered size
    pub size: f32,
    /// Alpha transparency (farther = fainter) - baked into sprite at spawn
    #[allow(dead_code)]
    pub alpha: f32,
    /// Velocity
    pub velocity: Vec2,
    /// Engine glow animation phase
    pub engine_phase: f32,
}

/// Engine glow child sprite
#[derive(Component)]
pub struct BackgroundShipEngine;

/// Ship hull child sprite
#[derive(Component)]
pub struct BackgroundShipHull;

/// Silhouette points for different ship classes (normalized 0-1 range)
/// These are side profiles for better visual distinction from gameplay ships
#[allow(dead_code)]
fn get_silhouette_points(class: BackgroundShipClass, faction: BackgroundShipFaction) -> Vec<Vec2> {
    let base = match (class, faction) {
        // Minmatar style - angular, aggressive
        (BackgroundShipClass::Frigate, BackgroundShipFaction::Minmatar) => vec![
            Vec2::new(0.0, 0.5),
            Vec2::new(0.2, 0.3),
            Vec2::new(0.4, 0.2),
            Vec2::new(0.7, 0.15),
            Vec2::new(1.0, 0.5), // nose
            Vec2::new(0.7, 0.85),
            Vec2::new(0.4, 0.8),
            Vec2::new(0.2, 0.7),
        ],
        (BackgroundShipClass::Cruiser, BackgroundShipFaction::Minmatar) => vec![
            Vec2::new(0.0, 0.4),
            Vec2::new(0.1, 0.25),
            Vec2::new(0.3, 0.2),
            Vec2::new(0.5, 0.15),
            Vec2::new(0.8, 0.2),
            Vec2::new(1.0, 0.5),
            Vec2::new(0.8, 0.8),
            Vec2::new(0.5, 0.85),
            Vec2::new(0.3, 0.8),
            Vec2::new(0.1, 0.75),
            Vec2::new(0.0, 0.6),
        ],
        (BackgroundShipClass::Battleship, BackgroundShipFaction::Minmatar) => vec![
            Vec2::new(0.0, 0.35),
            Vec2::new(0.05, 0.2),
            Vec2::new(0.2, 0.15),
            Vec2::new(0.4, 0.1),
            Vec2::new(0.6, 0.15),
            Vec2::new(0.85, 0.25),
            Vec2::new(1.0, 0.5),
            Vec2::new(0.85, 0.75),
            Vec2::new(0.6, 0.85),
            Vec2::new(0.4, 0.9),
            Vec2::new(0.2, 0.85),
            Vec2::new(0.05, 0.8),
            Vec2::new(0.0, 0.65),
        ],
        // Amarr style - sleek, golden curves
        (BackgroundShipClass::Frigate, BackgroundShipFaction::Amarr) => vec![
            Vec2::new(0.0, 0.5),
            Vec2::new(0.15, 0.35),
            Vec2::new(0.4, 0.25),
            Vec2::new(0.7, 0.3),
            Vec2::new(1.0, 0.5),
            Vec2::new(0.7, 0.7),
            Vec2::new(0.4, 0.75),
            Vec2::new(0.15, 0.65),
        ],
        (BackgroundShipClass::Cruiser, BackgroundShipFaction::Amarr) => vec![
            Vec2::new(0.0, 0.5),
            Vec2::new(0.1, 0.3),
            Vec2::new(0.25, 0.2),
            Vec2::new(0.5, 0.18),
            Vec2::new(0.75, 0.25),
            Vec2::new(1.0, 0.5),
            Vec2::new(0.75, 0.75),
            Vec2::new(0.5, 0.82),
            Vec2::new(0.25, 0.8),
            Vec2::new(0.1, 0.7),
        ],
        (BackgroundShipClass::Battleship, BackgroundShipFaction::Amarr) => vec![
            Vec2::new(0.0, 0.5),
            Vec2::new(0.08, 0.3),
            Vec2::new(0.2, 0.18),
            Vec2::new(0.4, 0.12),
            Vec2::new(0.6, 0.15),
            Vec2::new(0.8, 0.3),
            Vec2::new(1.0, 0.5),
            Vec2::new(0.8, 0.7),
            Vec2::new(0.6, 0.85),
            Vec2::new(0.4, 0.88),
            Vec2::new(0.2, 0.82),
            Vec2::new(0.08, 0.7),
        ],
    };

    // Flip horizontally if flying left (Amarr)
    if !faction.flies_right() {
        base.iter().map(|p| Vec2::new(1.0 - p.x, p.y)).collect()
    } else {
        base
    }
}

/// Load background images
fn load_backgrounds(mut backgrounds: ResMut<BackgroundAssets>, asset_server: Res<AssetServer>) {
    backgrounds.title = Some(asset_server.load("backgrounds/title_background.png"));
    info!("Loading background images...");
}

/// Spawn title background for menus
fn spawn_title_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
    existing: Query<Entity, With<MenuBackground>>,
    windows: Query<&Window>,
) {
    // Don't spawn if already exists
    if !existing.is_empty() {
        return;
    }

    let Some(handle) = backgrounds.title.clone() else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    // Spawn background sprite that covers the screen
    commands.spawn((
        MenuBackground,
        Sprite {
            image: handle,
            custom_size: Some(Vec2::new(window.width(), window.height())),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -100.0), // Behind everything
    ));
}

/// Despawn menu background when leaving menu states
fn despawn_menu_background(mut commands: Commands, query: Query<Entity, With<MenuBackground>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Spawn the parallax starfield for gameplay
fn spawn_starfield(mut commands: Commands, config: Res<StarfieldConfig>, windows: Query<&Window>) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let mut rng = rand::thread_rng();
    let half_width = window.width() / 2.0 + 100.0; // Extra buffer
    let half_height = window.height() / 2.0 + 100.0;

    // Spawn stars for each layer
    for layer in 0..config.layers {
        let layer_depth = layer as f32 / config.layers as f32;
        let speed = config.base_speed * config.layer_speed_mult.powi(layer as i32);

        // Stars in far layers are dimmer and smaller
        let base_brightness = 0.3 + 0.7 * layer_depth;
        let base_size = 1.0 + 2.0 * layer_depth;

        for _ in 0..config.stars_per_layer {
            let x = rng.gen_range(-half_width..half_width);
            let y = rng.gen_range(-half_height..half_height);
            let brightness = base_brightness * rng.gen_range(0.6..1.0);
            let size = base_size * rng.gen_range(0.5..1.5);
            let twinkle_phase = rng.gen_range(0.0..std::f32::consts::TAU);

            // Determine star color - mostly white/blue, occasional warm stars
            let color = if rng.gen_bool(0.85) {
                // Cool white/blue stars
                let blue_tint = rng.gen_range(0.9..1.0);
                Color::srgba(
                    brightness * 0.95,
                    brightness * 0.98,
                    brightness * blue_tint,
                    brightness,
                )
            } else if rng.gen_bool(0.5) {
                // Yellow/orange stars
                Color::srgba(brightness, brightness * 0.85, brightness * 0.5, brightness)
            } else {
                // Red dwarf stars
                Color::srgba(
                    brightness,
                    brightness * 0.6,
                    brightness * 0.5,
                    brightness * 0.9,
                )
            };

            // Z position based on layer (farther back for distant stars)
            let z = -99.0 + layer as f32 * 0.1;

            commands.spawn((
                BackgroundStar {
                    layer,
                    speed,
                    brightness,
                    twinkle_phase,
                },
                Starfield,
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz(x, y, z),
            ));
        }
    }

    // Add a few larger, brighter "prominent" stars
    for _ in 0..8 {
        let x = rng.gen_range(-half_width..half_width);
        let y = rng.gen_range(-half_height..half_height);
        let brightness = rng.gen_range(0.7..1.0);
        let size = rng.gen_range(3.0..5.0);

        commands.spawn((
            BackgroundStar {
                layer: config.layers - 1,
                speed: config.base_speed * config.layer_speed_mult.powi((config.layers - 1) as i32),
                brightness,
                twinkle_phase: rng.gen_range(0.0..std::f32::consts::TAU),
            },
            Starfield,
            Sprite {
                color: Color::srgba(brightness, brightness, brightness * 0.95, brightness),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(x, y, -98.5),
        ));
    }

    info!("Spawned starfield with {} layers", config.layers);
}

/// Update starfield - scroll and twinkle
fn update_starfield(
    mut stars: Query<(&mut Transform, &mut Sprite, &BackgroundStar)>,
    time: Res<Time>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let half_height = window.height() / 2.0 + 100.0;
    let half_width = window.width() / 2.0 + 100.0;
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    for (mut transform, mut sprite, star) in stars.iter_mut() {
        // Scroll downward (player flying "up" through space)
        transform.translation.y -= star.speed * dt;

        // Wrap around when off screen
        if transform.translation.y < -half_height {
            transform.translation.y = half_height;
            // Randomize x position when wrapping
            transform.translation.x = rand::thread_rng().gen_range(-half_width..half_width);
        }

        // Subtle twinkle effect
        let twinkle = 0.85 + 0.15 * (elapsed * 3.0 + star.twinkle_phase).sin();
        let alpha = star.brightness * twinkle;

        // Update alpha while preserving color
        let current = sprite.color.to_srgba();
        sprite.color = Color::srgba(current.red, current.green, current.blue, alpha);
    }
}

/// Despawn starfield when leaving gameplay
fn despawn_starfield(mut commands: Commands, stars: Query<Entity, With<Starfield>>) {
    for entity in stars.iter() {
        commands.entity(entity).despawn();
    }
    info!("Despawned starfield");
}

// =============================================================================
// BACKGROUND SHIP SYSTEMS
// =============================================================================

/// Spawn a new background ship
fn spawn_background_ship(
    commands: &mut Commands,
    window: &Window,
    session: &GameSession,
    sprites: &crate::assets::ShipSpriteCache,
) {
    let mut rng = rand::thread_rng();
    // Direction describes the two battle lines; hull and weapon colours come
    // from the selected conflict, including Caldari/Gallente.
    let direction = BackgroundShipFaction::random();
    let allied = direction.flies_right();
    let faction = if allied {
        session.player_faction
    } else {
        session.enemy_faction
    };
    let roster = faction.player_ships();
    let type_id = roster[fastrand::usize(..3.min(roster.len()))].type_id;
    let Some(image) = sprites.get(type_id) else {
        return;
    };
    let distance = rng.gen_range(0.3..0.65);
    let size = 55.0 * distance;
    let x = (window.width() / 2.0 + size) * if allied { -1.0 } else { 1.0 };
    let y = rng.gen_range(-window.height() / 2.0 + 50.0..window.height() / 2.0 - 50.0);
    let vx = if allied { 45.0 } else { -45.0 };
    let facing = if allied {
        -std::f32::consts::FRAC_PI_2
    } else {
        std::f32::consts::FRAC_PI_2
    };
    commands.spawn((
        BackgroundShip {
            faction: direction,
            ship_class: BackgroundShipClass::Frigate,
            distance,
            size,
            alpha: 0.35,
            velocity: Vec2::new(vx, -10.0),
            engine_phase: 0.0,
        },
        Starfield,
        Sprite {
            image,
            color: Color::srgba(0.7, 0.75, 0.85, 0.35),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_xyz(x, y, -90.0).with_rotation(Quat::from_rotation_z(
            facing + crate::entities::get_ship_rotation_correction(type_id),
        )),
    ));
    // Distant exchanges stay behind the carrier and have no collision or score.
    commands.spawn((
        DistantBattleFire {
            velocity: Vec2::new(vx * 6.0, -16.0),
            remaining: 4.0,
        },
        Starfield,
        Sprite {
            color: faction.primary_color().with_alpha(0.25),
            custom_size: Some(Vec2::new(18.0, 1.2)),
            ..default()
        },
        Transform::from_xyz(x, y, -85.0),
    ));
}

#[derive(Component)]
struct DistantBattleFire {
    velocity: Vec2,
    remaining: f32,
}

fn update_distant_battle(
    mut commands: Commands,
    time: Res<Time>,
    mut fire: Query<(Entity, &mut Transform, &mut DistantBattleFire)>,
) {
    for (entity, mut transform, mut bolt) in &mut fire {
        bolt.remaining -= time.delta_secs();
        if bolt.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += (bolt.velocity * time.delta_secs()).extend(0.0);
    }
}

/// Update background ship spawn timer and spawn new ships
fn update_background_ship_spawning(
    mut commands: Commands,
    mut timer: ResMut<BackgroundShipSpawnTimer>,
    time: Res<Time>,
    session: Res<GameSession>,
    sprites: Res<crate::assets::ShipSpriteCache>,
    config: Res<BackgroundShipConfig>,
    ships: Query<&BackgroundShip>,
    windows: Query<&Window>,
) {
    timer.elapsed += time.delta_secs();

    if timer.elapsed >= config.spawn_interval as f32 / 60.0 {
        timer.elapsed = 0.0;

        // Check if we can spawn more
        if ships.iter().count() < config.max_ships {
            // Random chance to actually spawn
            if fastrand::f32() < config.spawn_chance {
                if let Ok(window) = windows.get_single() {
                    spawn_background_ship(&mut commands, window, &session, &sprites);
                }
            }
        }
    }
}

/// Update background ship positions and despawn off-screen ships
fn update_background_ships(
    mut commands: Commands,
    mut ships: Query<(Entity, &mut Transform, &mut BackgroundShip), Without<BackgroundShipEngine>>,
    mut engines: Query<(&Parent, &mut Sprite), With<BackgroundShipEngine>>,
    time: Res<Time>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };

    let dt = time.delta_secs();
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    for (entity, mut transform, mut ship) in ships.iter_mut() {
        // Update position
        transform.translation.x += ship.velocity.x * dt;
        transform.translation.y += ship.velocity.y * dt;

        // Update engine animation
        ship.engine_phase += dt * 3.0;
        if ship.engine_phase > TAU {
            ship.engine_phase -= TAU;
        }

        // Engine glow pulsing factor - store for child engine sprites
        let _engine_intensity = 0.7 + 0.3 * ship.engine_phase.sin();

        // Check if off-screen (despawn with children)
        let margin = ship.size + 50.0;
        let x = transform.translation.x;
        let y = transform.translation.y;

        if x < -half_width - margin
            || x > half_width + margin
            || y < -half_height - margin
            || y > half_height + margin
        {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Update engine glow sprites
    for (parent, mut sprite) in engines.iter_mut() {
        // Get the parent ship's engine phase
        if let Ok((_, _, ship)) = ships.get(parent.get()) {
            let intensity = 0.7 + 0.3 * ship.engine_phase.sin();
            let current = sprite.color.to_srgba();
            sprite.color = Color::srgba(
                current.red,
                current.green,
                current.blue,
                current.alpha.min(0.8) * intensity,
            );
        }
    }
}
