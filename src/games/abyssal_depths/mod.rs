//! Abyssal Depths Mode
//!
//! Triglavian Invasion response — EDENCOM and empire forces counter the
//! Collective in Abyssal Deadspace. Three rooms, pure Triglavian enemies,
//! cross-empire player roster including EDENCOM's Skybreaker.

use bevy::prelude::*;

use crate::core::*;
use crate::entities::{Enemy, Player};
use crate::games::{GameModuleInfo, ModuleRegistry};
use crate::systems::JoystickState;

/// Aggregated Endless-mode roster — every player ship in the game.
/// Built once lazily from all 4 empire pools + the Invasion cross-faction set.
pub static ENDLESS_SHIPS: std::sync::LazyLock<Vec<ShipDef>> =
    std::sync::LazyLock::new(|| {
        let mut v: Vec<ShipDef> = Vec::new();
        v.extend_from_slice(crate::core::factions::MINMATAR_SHIPS);
        v.extend_from_slice(crate::core::factions::AMARR_SHIPS);
        v.extend_from_slice(crate::core::factions::CALDARI_SHIPS);
        v.extend_from_slice(crate::core::factions::GALLENTE_SHIPS);
        v.extend_from_slice(TRIGLAVIAN_INVASION_SHIPS);
        v
    });

/// Cross-faction player roster used while the Triglavian Invasion chapter is
/// active. Historically-accurate EVE ships that fought in Invasion-era combat.
pub const TRIGLAVIAN_INVASION_SHIPS: &[ShipDef] = &[
    ShipDef {
        type_id: 54731,
        name: "Skybreaker",
        class: ShipClass::Frigate,
        role: "EDENCOM Vorton Platform",
        health: 90.0,
        speed: 330.0,
        fire_rate: 6.5,
        damage: 12.0,
        special: "Vorton Projector: Chain lightning",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11400,
        name: "Jaguar",
        class: ShipClass::Frigate,
        role: "Republic Assault Frigate",
        health: 120.0,
        speed: 360.0,
        fire_rate: 9.0,
        damage: 11.0,
        special: "Afterburner: Speed burst",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11381,
        name: "Hawk",
        class: ShipClass::Frigate,
        role: "Caldari Assault Frigate",
        health: 120.0,
        speed: 320.0,
        fire_rate: 7.0,
        damage: 13.0,
        special: "Rocket Salvo: Quad missile",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11393,
        name: "Retribution",
        class: ShipClass::Frigate,
        role: "Imperial Assault Frigate",
        health: 130.0,
        speed: 310.0,
        fire_rate: 6.0,
        damage: 15.0,
        special: "Scorch: Range extender",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 52250,
        name: "Nergal",
        class: ShipClass::Frigate,
        role: "Triglavian-Derived AF",
        health: 125.0,
        speed: 340.0,
        fire_rate: 7.5,
        damage: 14.0,
        special: "Entropic: Ramping damage",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 35683,
        name: "Jackdaw",
        class: ShipClass::Destroyer,
        role: "Caldari T3 Tactical",
        health: 180.0,
        speed: 290.0,
        fire_rate: 6.0,
        damage: 17.0,
        special: "Mode-Switch: 3 stances",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 34317,
        name: "Confessor",
        class: ShipClass::Destroyer,
        role: "Amarr T3 Tactical",
        health: 175.0,
        speed: 300.0,
        fire_rate: 5.5,
        damage: 19.0,
        special: "Mode-Switch: Laser stances",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 621,
        name: "Caracal",
        class: ShipClass::Cruiser,
        role: "Caldari Missile Cruiser",
        health: 260.0,
        speed: 240.0,
        fire_rate: 5.0,
        damage: 22.0,
        special: "Heavy Missiles",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11993,
        name: "Muninn",
        class: ShipClass::Cruiser,
        role: "Minmatar HAC",
        health: 300.0,
        speed: 230.0,
        fire_rate: 4.5,
        damage: 26.0,
        special: "Artillery Alpha",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 12019,
        name: "Sacrilege",
        class: ShipClass::Cruiser,
        role: "Amarr HAC",
        health: 320.0,
        speed: 210.0,
        fire_rate: 4.0,
        damage: 28.0,
        special: "Heavy Assault Missiles",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 17713,
        name: "Gila",
        class: ShipClass::Cruiser,
        role: "Guristas Drone Cruiser",
        health: 290.0,
        speed: 250.0,
        fire_rate: 5.5,
        damage: 24.0,
        special: "Drone Swarm: Rapid Missiles",
        unlock_stage: 0,
    },
];

/// Abyssal Depths game module plugin
pub struct AbyssalDepthsPlugin;

impl Plugin for AbyssalDepthsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AbyssalState>()
            .add_event::<AbyssalRoomClearEvent>()
            .add_event::<AbyssalExtractionEvent>()
            .add_systems(Startup, register_module)
            .add_systems(
                OnEnter(GameState::Playing),
                setup_abyssal.run_if(is_abyssal),
            )
            .add_systems(
                OnExit(GameState::Playing),
                cleanup_abyssal.run_if(is_abyssal),
            )
            .add_systems(
                Update,
                (
                    update_abyssal_timer,
                    check_room_clear,
                    update_gate,
                    handle_extraction,
                    abyssal_hud,
                )
                    .run_if(in_state(GameState::Playing))
                    .run_if(is_abyssal),
            );
    }
}

/// Run condition: is the active module Abyssal Depths?
fn is_abyssal(active: Res<crate::games::ActiveModule>) -> bool {
    active.module_id.as_deref() == Some("abyssal_depths")
}

/// Register this module in the module registry
fn register_module(mut registry: ResMut<ModuleRegistry>) {
    registry.register(GameModuleInfo {
        id: "abyssal_depths",
        display_name: "ABYSSAL DEPTHS",
        subtitle: "Triglavian Extraction",
        description: "Enter the Abyss. Survive 3 rooms. Extract or die.",
        factions: vec![], // No faction selection - always Triglavian enemies
    });
}

/// Current room in the abyss
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AbyssalRoom {
    #[default]
    Room1, // Pockets - light enemies
    Room2, // Escalation - medium + hazards
    Room3, // Extraction - boss + extraction gate
}

impl AbyssalRoom {
    pub fn name(&self) -> &'static str {
        match self {
            AbyssalRoom::Room1 => "POCKET",
            AbyssalRoom::Room2 => "ESCALATION",
            AbyssalRoom::Room3 => "EXTRACTION",
        }
    }

    pub fn enemy_count(&self) -> u32 {
        match self {
            AbyssalRoom::Room1 => 8,
            AbyssalRoom::Room2 => 12,
            AbyssalRoom::Room3 => 15,
        }
    }

    pub fn next(&self) -> Option<AbyssalRoom> {
        match self {
            AbyssalRoom::Room1 => Some(AbyssalRoom::Room2),
            AbyssalRoom::Room2 => Some(AbyssalRoom::Room3),
            AbyssalRoom::Room3 => None,
        }
    }
}

/// Abyssal run state
#[derive(Resource, Debug, Clone)]
pub struct AbyssalState {
    /// Is an abyssal run active?
    pub active: bool,
    /// Current room
    pub room: AbyssalRoom,
    /// Time remaining in seconds
    pub time_remaining: f32,
    /// Total time for run
    pub total_time: f32,
    /// Enemies spawned this room
    pub enemies_spawned: u32,
    /// Enemies killed this room
    pub enemies_killed: u32,
    /// Is room cleared?
    pub room_cleared: bool,
    /// Is gate spawned?
    pub gate_spawned: bool,
    /// Is player channeling extraction?
    pub extracting: bool,
    /// Extraction channel progress (0-1)
    pub extraction_progress: f32,
    /// Total loot collected
    pub loot_collected: u64,
    /// Run completed successfully?
    pub extracted: bool,
}

impl Default for AbyssalState {
    fn default() -> Self {
        Self {
            active: false,
            room: AbyssalRoom::Room1,
            time_remaining: 600.0, // 10 minutes
            total_time: 600.0,
            enemies_spawned: 0,
            enemies_killed: 0,
            room_cleared: false,
            gate_spawned: false,
            extracting: false,
            extraction_progress: 0.0,
            loot_collected: 0,
            extracted: false,
        }
    }
}

impl AbyssalState {
    pub fn start_run(&mut self) {
        *self = Self {
            active: true,
            room: AbyssalRoom::Room1,
            time_remaining: 600.0,
            total_time: 600.0,
            ..default()
        };
    }

    pub fn advance_room(&mut self) -> bool {
        if let Some(next) = self.room.next() {
            self.room = next;
            self.enemies_spawned = 0;
            self.enemies_killed = 0;
            self.room_cleared = false;
            self.gate_spawned = false;
            true
        } else {
            false
        }
    }

    pub fn time_percent(&self) -> f32 {
        self.time_remaining / self.total_time
    }

    pub fn is_final_room(&self) -> bool {
        self.room == AbyssalRoom::Room3
    }
}

/// Event: Room has been cleared
#[derive(Event)]
pub struct AbyssalRoomClearEvent {
    pub room: AbyssalRoom,
}

/// Event: Player extracted successfully
#[derive(Event)]
pub struct AbyssalExtractionEvent {
    pub loot: u64,
}

/// Marker for the abyssal gate entity
#[derive(Component)]
pub struct AbyssalGate {
    pub is_extraction: bool,
}

/// Marker for abyssal HUD elements
#[derive(Component)]
pub struct AbyssalHud;

/// Setup abyssal run when entering Playing state
fn setup_abyssal(
    mut state: ResMut<AbyssalState>,
    mut commands: Commands,
    session: Res<GameSession>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
) {
    // Only setup if we're in abyssal module
    state.start_run();
    info!("Abyssal Depths run started - Room 1: POCKET");

    // Spawn initial wave of enemies
    spawn_room_enemies(&mut commands, &state, &session, &sprite_cache);

    // Spawn HUD
    spawn_abyssal_hud(&mut commands);
}

/// Spawn enemies for the current room with escalating difficulty.
/// Room 1: Light rogue drones (Linear/Zigzag) — teaches mechanics
/// Room 2: Mixed fleet (Homing/Sniper + Triglavians) — pressure ramp
/// Room 3: Full Triglavian force (Vedmak/Damavik + Drekavac boss) — survival test
fn spawn_room_enemies(
    commands: &mut Commands,
    state: &AbyssalState,
    _session: &GameSession,
    sprite_cache: &crate::assets::ShipSpriteCache,
) {
    use crate::entities::enemy::{spawn_enemy, spawn_variant, EnemyBehavior, EnemyVariant};
    use crate::entities::triglavian;

    let count = state.room.enemy_count();
    let spawn_y_base = SCREEN_HEIGHT / 2.0 - 50.0;

    // Pre-fetch sprite handles once per room so every spawn gets the right hull.
    let damavik_sprite = sprite_cache.get(triglavian::DAMAVIK);
    let kikimora_sprite = sprite_cache.get(triglavian::KIKIMORA);
    let vedmak_sprite = sprite_cache.get(triglavian::VEDMAK);
    let drekavac_sprite = sprite_cache.get(triglavian::DREKAVAC);

    match state.room {
        AbyssalRoom::Room1 => {
            // Pure Triglavian presence — Damaviks with entry-level behaviors.
            // Lore: Abyssal Deadspace is Triglavian territory. No empire ships.
            let behaviors = [
                EnemyBehavior::Linear,
                EnemyBehavior::Zigzag,
                EnemyBehavior::Weaver,
            ];

            for i in 0..count {
                let x =
                    -SCREEN_WIDTH / 2.0 + (i as f32 + 1.0) * (SCREEN_WIDTH / (count as f32 + 1.0));
                let y = spawn_y_base + fastrand::f32() * 100.0;
                let _ = behaviors[i as usize % behaviors.len()];
                // spawn_variant handles Triglavian-specific stats + behaviour
                spawn_variant(
                    commands,
                    EnemyVariant::Damavik,
                    Vec2::new(x, y),
                    damavik_sprite.clone(),
                    None,
                );
            }
        }
        AbyssalRoom::Room2 => {
            // Escalation — Damaviks + Kikimora destroyers (T2 Triglavian).
            for i in 0..count {
                let x =
                    -SCREEN_WIDTH / 2.0 + (i as f32 + 1.0) * (SCREEN_WIDTH / (count as f32 + 1.0));
                let y = spawn_y_base + fastrand::f32() * 100.0;

                if i % 3 == 0 {
                    // Every 3rd enemy is a Kikimora destroyer
                    spawn_enemy(
                        commands,
                        triglavian::KIKIMORA,
                        Vec2::new(x, y),
                        EnemyBehavior::Sniper,
                        kikimora_sprite.clone(),
                        None,
                    );
                } else {
                    spawn_variant(
                        commands,
                        EnemyVariant::Damavik,
                        Vec2::new(x, y),
                        damavik_sprite.clone(),
                        None,
                    );
                }
            }
        }
        AbyssalRoom::Room3 => {
            // Full Triglavian force — heavy Vedmaks with fast Damavik escorts
            for i in 0..count {
                let x =
                    -SCREEN_WIDTH / 2.0 + (i as f32 + 1.0) * (SCREEN_WIDTH / (count as f32 + 1.0));
                let y = spawn_y_base + fastrand::f32() * 100.0;

                if i % 3 == 0 {
                    // Heavy Vedmak cruisers
                    spawn_variant(
                        commands,
                        EnemyVariant::Vedmak,
                        Vec2::new(x, y),
                        vedmak_sprite.clone(),
                        None,
                    );
                } else {
                    // Fast Starving Damaviks
                    spawn_variant(
                        commands,
                        EnemyVariant::StarvingDamavik,
                        Vec2::new(x, y),
                        damavik_sprite.clone(),
                        None,
                    );
                }
            }

            // Drekavac boss
            let boss_pos = Vec2::new(0.0, spawn_y_base + 50.0);
            spawn_variant(
                commands,
                EnemyVariant::DrekavacBoss,
                boss_pos,
                drekavac_sprite.clone(),
                None,
            );
        }
    }

    let extra = if state.room == AbyssalRoom::Room3 {
        1
    } else {
        0
    };
    info!(
        "Spawned {} enemies for room {:?}",
        count + extra,
        state.room
    );
}

/// Spawn the abyssal HUD overlay
fn spawn_abyssal_hud(commands: &mut Commands) {
    commands
        .spawn((
            AbyssalHud,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Room indicator
            parent.spawn((
                Text::new("POCKET"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.4, 0.4)),
                AbyssalRoomText,
            ));

            // Timer
            parent.spawn((
                Text::new("10:00"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                AbyssalTimerText,
            ));

            // Enemy count
            parent.spawn((
                Text::new("0/8"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                AbyssalEnemyText,
            ));
        });
}

#[derive(Component)]
struct AbyssalRoomText;

#[derive(Component)]
struct AbyssalTimerText;

#[derive(Component)]
struct AbyssalEnemyText;

/// Update the abyssal timer
fn update_abyssal_timer(
    time: Res<Time>,
    mut state: ResMut<AbyssalState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !state.active {
        return;
    }

    state.time_remaining -= time.delta_secs();

    // Time's up - lost in the abyss
    if state.time_remaining <= 0.0 {
        state.time_remaining = 0.0;
        state.active = false;
        info!("LOST IN THE ABYSS - Time ran out!");
        next_state.set(GameState::GameOver);
    }
}

/// Check if room is cleared
fn check_room_clear(
    mut state: ResMut<AbyssalState>,
    enemy_query: Query<Entity, With<Enemy>>,
    mut clear_events: EventWriter<AbyssalRoomClearEvent>,
) {
    if !state.active || state.room_cleared {
        return;
    }

    let enemy_count = enemy_query.iter().count();
    state.enemies_killed = state.room.enemy_count().saturating_sub(enemy_count as u32);

    if enemy_count == 0 && state.enemies_spawned > 0 {
        state.room_cleared = true;
        clear_events.send(AbyssalRoomClearEvent { room: state.room });
        info!("Room {:?} CLEARED!", state.room);
    }

    // Mark enemies as spawned after first frame
    if state.enemies_spawned == 0 {
        state.enemies_spawned = state.room.enemy_count();
    }
}

/// Update gate spawning and interaction
fn update_gate(
    mut commands: Commands,
    mut state: ResMut<AbyssalState>,
    gate_query: Query<(Entity, &Transform, &AbyssalGate)>,
    player_query: Query<&Transform, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    time: Res<Time>,
) {
    if !state.active || !state.room_cleared {
        return;
    }

    // Spawn gate if not spawned yet
    if !state.gate_spawned {
        let is_extraction = state.is_final_room();
        let gate_color = if is_extraction {
            Color::srgb(0.2, 1.0, 0.5) // Green for extraction
        } else {
            Color::srgb(0.5, 0.5, 1.0) // Blue for transition
        };

        commands.spawn((
            AbyssalGate { is_extraction },
            Sprite {
                color: gate_color,
                custom_size: Some(Vec2::new(80.0, 80.0)),
                ..default()
            },
            Transform::from_xyz(0.0, SCREEN_HEIGHT / 2.0 - 100.0, LAYER_EFFECTS),
        ));

        state.gate_spawned = true;
        info!(
            "Gate spawned - {}",
            if is_extraction {
                "EXTRACTION GATE"
            } else {
                "Room transition"
            }
        );
    }

    // Check player proximity to gate
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (_entity, gate_transform, gate) in gate_query.iter() {
        let gate_pos = gate_transform.translation.truncate();
        let distance = (player_pos - gate_pos).length();

        if distance < 60.0 {
            // Player is near gate
            let interact = keyboard.pressed(KeyCode::KeyE)
                || keyboard.pressed(KeyCode::Space)
                || joystick.confirm();

            if interact {
                if gate.is_extraction {
                    // Channel extraction
                    state.extracting = true;
                    state.extraction_progress += time.delta_secs() / 2.0; // 2 second channel
                } else if keyboard.just_pressed(KeyCode::KeyE)
                    || keyboard.just_pressed(KeyCode::Space)
                    || joystick.confirm()
                {
                    // Transition to next room
                    state.advance_room();
                    state.gate_spawned = false;

                    // Despawn gate
                    for (entity, _, _) in gate_query.iter() {
                        commands.entity(entity).despawn_recursive();
                    }

                    // Spawn new enemies
                    // (will be spawned next frame via a separate system)
                    info!("Transitioning to room {:?}", state.room);
                }
            } else {
                state.extracting = false;
                state.extraction_progress = 0.0;
            }
        }
    }
}

/// Spawn enemies when entering a new room
fn handle_extraction(
    mut state: ResMut<AbyssalState>,
    mut commands: Commands,
    session: Res<GameSession>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    mut extraction_events: EventWriter<AbyssalExtractionEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    gate_query: Query<Entity, With<AbyssalGate>>,
    enemy_query: Query<Entity, With<Enemy>>,
) {
    if !state.active {
        return;
    }

    // Check if extraction complete
    if state.extraction_progress >= 1.0 {
        state.extracted = true;
        state.active = false;

        // Calculate loot based on rooms completed
        let loot = 500 + (state.room as u64 * 500);
        state.loot_collected = loot;

        extraction_events.send(AbyssalExtractionEvent { loot });
        info!("EXTRACTION SUCCESSFUL! Loot: {} ISK", loot);

        next_state.set(GameState::Victory);
        return;
    }

    // Spawn enemies for new room if needed
    if state.room_cleared && !state.gate_spawned {
        // Room just changed, wait for gate spawn
    } else if !state.room_cleared && enemy_query.iter().count() == 0 && state.enemies_spawned == 0 {
        // Need to spawn enemies for current room
        spawn_room_enemies(&mut commands, &state, &session, &sprite_cache);

        // Cleanup old gate if exists
        for entity in gate_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Update abyssal HUD
fn abyssal_hud(
    state: Res<AbyssalState>,
    mut room_query: Query<
        &mut Text,
        (
            With<AbyssalRoomText>,
            Without<AbyssalTimerText>,
            Without<AbyssalEnemyText>,
        ),
    >,
    mut timer_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<AbyssalTimerText>,
            Without<AbyssalRoomText>,
            Without<AbyssalEnemyText>,
        ),
    >,
    mut enemy_query: Query<
        &mut Text,
        (
            With<AbyssalEnemyText>,
            Without<AbyssalRoomText>,
            Without<AbyssalTimerText>,
        ),
    >,
) {
    // Update room text
    if let Ok(mut text) = room_query.get_single_mut() {
        text.0 = format!("ROOM: {}", state.room.name());
    }

    // Update timer
    if let Ok((mut text, mut color)) = timer_query.get_single_mut() {
        let mins = (state.time_remaining / 60.0) as u32;
        let secs = (state.time_remaining % 60.0) as u32;
        text.0 = format!("{:02}:{:02}", mins, secs);

        // Color based on time remaining
        if state.time_remaining < 60.0 {
            color.0 = Color::srgb(1.0, 0.2, 0.2); // Red - critical
        } else if state.time_remaining < 180.0 {
            color.0 = Color::srgb(1.0, 0.6, 0.2); // Orange - warning
        } else {
            color.0 = Color::srgb(1.0, 0.8, 0.2); // Yellow - normal
        }
    }

    // Update enemy count
    if let Ok(mut text) = enemy_query.get_single_mut() {
        let total = state.room.enemy_count();
        text.0 = format!("{}/{}", state.enemies_killed, total);
    }
}

/// Cleanup abyssal state when leaving
fn cleanup_abyssal(
    mut commands: Commands,
    mut state: ResMut<AbyssalState>,
    hud_query: Query<Entity, With<AbyssalHud>>,
    gate_query: Query<Entity, With<AbyssalGate>>,
) {
    state.active = false;

    // Despawn HUD
    for entity in hud_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Despawn gates
    for entity in gate_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
