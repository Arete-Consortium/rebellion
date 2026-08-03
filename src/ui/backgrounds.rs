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
            .add_systems(OnEnter(GameState::Playing), spawn_starfield)
            .add_systems(
                Update,
                (
                    update_starfield,
                    update_background_ship_spawning,
                    update_background_ships,
                )
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
            )
            .add_systems(
                OnExit(GameState::Playing),
                despawn_starfield
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
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
            spawn_interval: 120, // ~2 seconds at 60fps
            spawn_chance: 0.5,
        }
    }
}

/// Spawn timer for background ships
#[derive(Resource, Default)]
pub struct BackgroundShipSpawnTimer {
    pub frames: u32,
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
fn spawn_background_ship(commands: &mut Commands, window: &Window) {
    let mut rng = rand::thread_rng();

    let faction = BackgroundShipFaction::random();
    let ship_class = BackgroundShipClass::random();

    // Distance affects size and alpha
    let distance = rng.gen_range(0.3..0.8);
    let size = ship_class.base_size() * distance;
    let alpha = 0.25 + distance * 0.5; // 0.25-0.65 range

    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;

    // Starting position based on faction
    let (x, vx) = if faction.flies_right() {
        // Minmatar: start left, fly right
        let x = -half_width - size;
        let vx = rng.gen_range(0.8..1.5) * distance * 60.0; // pixels/sec
        (x, vx)
    } else {
        // Amarr: start right, fly left
        let x = half_width + size;
        let vx = rng.gen_range(-1.5..-0.8) * distance * 60.0;
        (x, vx)
    };

    let y = rng.gen_range(-half_height + 50.0..half_height - 50.0);
    let vy = rng.gen_range(-0.15..0.15) * 60.0;

    let ship = BackgroundShip {
        faction,
        ship_class,
        distance,
        size,
        alpha,
        velocity: Vec2::new(vx, vy),
        engine_phase: rng.gen_range(0.0..TAU),
    };

    // Ship dimensions
    let ship_width = size;
    let ship_height = size * 0.35;

    // Create hull color with transparency
    let hull_color = faction.hull_color();
    let hull_rgba = hull_color.to_srgba();
    let color_with_alpha = Color::srgba(hull_rgba.red, hull_rgba.green, hull_rgba.blue, alpha);

    // Engine glow color
    let engine_color = faction.engine_color();
    let engine_rgba = engine_color.to_srgba();
    let engine_glow = Color::srgba(
        engine_rgba.red,
        engine_rgba.green,
        engine_rgba.blue,
        alpha * 0.8,
    );

    // Direction multiplier for nose/engine position
    let dir = if faction.flies_right() { 1.0 } else { -1.0 };

    // Spawn ship as parent with children for detailed silhouette
    commands
        .spawn((
            ship,
            Starfield, // Use same marker for cleanup
            Transform::from_xyz(x, y, -98.0),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Main hull body (center mass)
            parent.spawn((
                BackgroundShipHull,
                Sprite {
                    color: color_with_alpha,
                    custom_size: Some(Vec2::new(ship_width * 0.65, ship_height)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));

            // Nose section (pointed front)
            let nose_width = match ship_class {
                BackgroundShipClass::Frigate => ship_width * 0.35,
                BackgroundShipClass::Cruiser => ship_width * 0.38,
                BackgroundShipClass::Battleship => ship_width * 0.4,
            };
            let nose_height = match ship_class {
                BackgroundShipClass::Frigate => ship_height * 0.6,
                BackgroundShipClass::Cruiser => ship_height * 0.7,
                BackgroundShipClass::Battleship => ship_height * 0.65,
            };

            parent.spawn((
                Sprite {
                    color: color_with_alpha,
                    custom_size: Some(Vec2::new(nose_width, nose_height)),
                    ..default()
                },
                Transform::from_xyz(ship_width * 0.4 * dir, 0.0, 0.01)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4 * dir)),
            ));

            // Engine block (rear section)
            let engine_width = ship_width * 0.2;
            let engine_height = ship_height * 0.8;

            parent.spawn((
                Sprite {
                    color: color_with_alpha,
                    custom_size: Some(Vec2::new(engine_width, engine_height)),
                    ..default()
                },
                Transform::from_xyz(-ship_width * 0.38 * dir, 0.0, 0.01),
            ));

            // Engine glow (bright spot at rear)
            parent.spawn((
                BackgroundShipEngine,
                Sprite {
                    color: engine_glow,
                    custom_size: Some(Vec2::new(engine_width * 0.6, engine_height * 0.5)),
                    ..default()
                },
                Transform::from_xyz(-ship_width * 0.48 * dir, 0.0, 0.02),
            ));

            // Add wing/fin for cruisers and battleships
            if matches!(
                ship_class,
                BackgroundShipClass::Cruiser | BackgroundShipClass::Battleship
            ) {
                let fin_width = ship_width * 0.15;
                let fin_height = ship_height * 1.3;

                // Upper fin
                parent.spawn((
                    Sprite {
                        color: color_with_alpha.with_alpha(alpha * 0.8),
                        custom_size: Some(Vec2::new(fin_width, fin_height * 0.4)),
                        ..default()
                    },
                    Transform::from_xyz(-ship_width * 0.1 * dir, ship_height * 0.5, 0.0),
                ));

                // Lower fin
                parent.spawn((
                    Sprite {
                        color: color_with_alpha.with_alpha(alpha * 0.8),
                        custom_size: Some(Vec2::new(fin_width, fin_height * 0.4)),
                        ..default()
                    },
                    Transform::from_xyz(-ship_width * 0.1 * dir, -ship_height * 0.5, 0.0),
                ));
            }

            // Add extra detail for battleships
            if matches!(ship_class, BackgroundShipClass::Battleship) {
                // Bridge/superstructure
                parent.spawn((
                    Sprite {
                        color: color_with_alpha.with_alpha(alpha * 0.9),
                        custom_size: Some(Vec2::new(ship_width * 0.12, ship_height * 0.3)),
                        ..default()
                    },
                    Transform::from_xyz(ship_width * 0.1 * dir, ship_height * 0.35, 0.01),
                ));

                // Secondary engine glow
                parent.spawn((
                    BackgroundShipEngine,
                    Sprite {
                        color: engine_glow.with_alpha(alpha * 0.5),
                        custom_size: Some(Vec2::new(engine_width * 0.4, engine_height * 0.3)),
                        ..default()
                    },
                    Transform::from_xyz(-ship_width * 0.48 * dir, ship_height * 0.25, 0.02),
                ));
                parent.spawn((
                    BackgroundShipEngine,
                    Sprite {
                        color: engine_glow.with_alpha(alpha * 0.5),
                        custom_size: Some(Vec2::new(engine_width * 0.4, engine_height * 0.3)),
                        ..default()
                    },
                    Transform::from_xyz(-ship_width * 0.48 * dir, -ship_height * 0.25, 0.02),
                ));
            }
        });
}

/// Update background ship spawn timer and spawn new ships
fn update_background_ship_spawning(
    mut commands: Commands,
    mut timer: ResMut<BackgroundShipSpawnTimer>,
    config: Res<BackgroundShipConfig>,
    ships: Query<&BackgroundShip>,
    windows: Query<&Window>,
) {
    timer.frames += 1;

    if timer.frames >= config.spawn_interval {
        timer.frames = 0;

        // Check if we can spawn more
        if ships.iter().count() < config.max_ships {
            // Random chance to actually spawn
            if fastrand::f32() < config.spawn_chance {
                if let Ok(window) = windows.get_single() {
                    spawn_background_ship(&mut commands, window);
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
