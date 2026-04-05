//! Cross-Platform Gamepad Input
//!
//! Uses Bevy's built-in Gamepad API for controller support on all platforms
//! including WASM (browser Gamepad API), Linux, Windows, macOS, and Steam Deck.
//!
//! Provides a unified JoystickState resource consumed by all game systems.

#![allow(dead_code)]

use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};
use bevy::prelude::*;
use std::time::Duration;

/// Default deadzone (used when no profile is detected)
const DEFAULT_DEADZONE: f32 = 0.15;

// =============================================================================
// CONTROLLER PROFILES
// =============================================================================

/// Controller type for profile selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControllerType {
    #[default]
    Unknown,
    SteamDeck,
    Xbox,
    PlayStation,
}

/// Controller-specific tuning parameters
#[derive(Resource, Debug, Clone)]
pub struct ControllerProfile {
    pub controller_type: ControllerType,
    pub name: String,
    pub left_stick_deadzone: f32,
    pub right_stick_deadzone: f32,
    pub trigger_deadzone: f32,
    pub movement_sensitivity: f32,
    pub aim_sensitivity: f32,
    pub rumble_multiplier: f32,
}

impl Default for ControllerProfile {
    fn default() -> Self {
        Self {
            controller_type: ControllerType::Unknown,
            name: String::new(),
            left_stick_deadzone: DEFAULT_DEADZONE,
            right_stick_deadzone: 0.20,
            trigger_deadzone: 0.10,
            movement_sensitivity: 1.0,
            aim_sensitivity: 0.8,
            rumble_multiplier: 1.0,
        }
    }
}

impl ControllerProfile {
    pub fn steam_deck() -> Self {
        Self {
            controller_type: ControllerType::SteamDeck,
            name: "Steam Deck".to_string(),
            left_stick_deadzone: 0.12,
            right_stick_deadzone: 0.15,
            trigger_deadzone: 0.08,
            movement_sensitivity: 1.0,
            aim_sensitivity: 0.9,
            rumble_multiplier: 0.7,
        }
    }

    pub fn xbox() -> Self {
        Self {
            controller_type: ControllerType::Xbox,
            name: "Xbox Controller".to_string(),
            left_stick_deadzone: 0.15,
            right_stick_deadzone: 0.18,
            trigger_deadzone: 0.10,
            movement_sensitivity: 1.0,
            aim_sensitivity: 0.85,
            rumble_multiplier: 1.0,
        }
    }

    pub fn playstation() -> Self {
        Self {
            controller_type: ControllerType::PlayStation,
            name: "PlayStation Controller".to_string(),
            left_stick_deadzone: 0.12,
            right_stick_deadzone: 0.15,
            trigger_deadzone: 0.08,
            movement_sensitivity: 1.0,
            aim_sensitivity: 0.9,
            rumble_multiplier: 0.9,
        }
    }

    pub fn from_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.contains("steam deck") || lower.contains("steamdeck") {
            Self::steam_deck()
        } else if lower.contains("xbox") || lower.contains("microsoft") || lower.contains("xinput")
        {
            Self::xbox()
        } else if lower.contains("playstation")
            || lower.contains("dualshock")
            || lower.contains("dualsense")
            || lower.contains("sony")
        {
            Self::playstation()
        } else {
            Self {
                name: name.to_string(),
                ..Default::default()
            }
        }
    }
}

// =============================================================================
// BACK BUTTON CONFIG
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackButtonAction {
    AmmoPrev,
    AmmoNext,
    SpeedBoost,
    QuickRocket,
    Ability,
    None,
}

#[derive(Resource, Debug, Clone)]
pub struct BackButtonConfig {
    pub l4: BackButtonAction,
    pub l5: BackButtonAction,
    pub r4: BackButtonAction,
    pub r5: BackButtonAction,
}

impl Default for BackButtonConfig {
    fn default() -> Self {
        Self {
            l4: BackButtonAction::AmmoPrev,
            l5: BackButtonAction::SpeedBoost,
            r4: BackButtonAction::AmmoNext,
            r5: BackButtonAction::QuickRocket,
        }
    }
}

// =============================================================================
// RUMBLE
// =============================================================================

#[derive(Resource, Debug)]
pub struct RumbleSettings {
    pub intensity: f32,
}

impl Default for RumbleSettings {
    fn default() -> Self {
        Self { intensity: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RumbleType {
    PlayerHit,
    Explosion,
    BigExplosion,
    SaltMinerActivate,
    PowerupCollect,
    Custom {
        strong: f32,
        weak: f32,
        duration_ms: u64,
    },
}

impl RumbleType {
    fn params(&self) -> (f32, f32, u64) {
        match self {
            RumbleType::PlayerHit => (0.0, 0.4, 80),
            RumbleType::Explosion => (0.5, 0.3, 120),
            RumbleType::BigExplosion => (1.0, 0.5, 250),
            RumbleType::SaltMinerActivate => (0.8, 0.6, 300),
            RumbleType::PowerupCollect => (0.0, 0.25, 60),
            RumbleType::Custom {
                strong,
                weak,
                duration_ms,
            } => (*strong, *weak, *duration_ms),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct RumbleRequest {
    pub rumble_type: RumbleType,
}

impl RumbleRequest {
    pub fn new(rumble_type: RumbleType) -> Self {
        Self { rumble_type }
    }
    pub fn player_hit() -> Self {
        Self::new(RumbleType::PlayerHit)
    }
    pub fn explosion() -> Self {
        Self::new(RumbleType::Explosion)
    }
    pub fn big_explosion() -> Self {
        Self::new(RumbleType::BigExplosion)
    }
    pub fn salt_miner() -> Self {
        Self::new(RumbleType::SaltMinerActivate)
    }
    pub fn powerup() -> Self {
        Self::new(RumbleType::PowerupCollect)
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct BackButtonEvent {
    pub action: BackButtonAction,
}

// =============================================================================
// JOYSTICK STATE (public interface consumed by all game systems)
// =============================================================================

#[derive(Resource, Default, Debug)]
pub struct JoystickState {
    pub left_x: f32,
    pub left_y: f32,
    pub right_x: f32,
    pub right_y: f32,
    pub left_trigger: f32,
    pub right_trigger: f32,
    pub dpad_x: i8,
    pub dpad_y: i8,
    pub prev_dpad_x: i8,
    pub prev_dpad_y: i8,
    pub prev_left_y: f32,
    pub buttons: [bool; 16],
    pub prev_buttons: [bool; 16],
    pub connected: bool,
}

impl JoystickState {
    pub fn just_pressed(&self, button: usize) -> bool {
        button < 16 && self.buttons[button] && !self.prev_buttons[button]
    }

    pub fn dpad_just_up(&self) -> bool {
        self.dpad_y > 0 && self.prev_dpad_y <= 0 // Bevy: up = positive
    }
    pub fn dpad_just_down(&self) -> bool {
        self.dpad_y < 0 && self.prev_dpad_y >= 0 // Bevy: down = negative
    }
    pub fn dpad_just_left(&self) -> bool {
        self.dpad_x < 0 && self.prev_dpad_x >= 0
    }
    pub fn dpad_just_right(&self) -> bool {
        self.dpad_x > 0 && self.prev_dpad_x <= 0
    }
    pub fn stick_just_up(&self) -> bool {
        self.left_y > 0.5 && self.prev_left_y <= 0.5 // Bevy: up = +1.0
    }
    pub fn stick_just_down(&self) -> bool {
        self.left_y < -0.5 && self.prev_left_y >= -0.5 // Bevy: down = -1.0
    }

    pub fn movement(&self) -> Vec2 {
        self.movement_with_deadzone(DEFAULT_DEADZONE)
    }

    pub fn movement_with_deadzone(&self, deadzone: f32) -> Vec2 {
        let mut x = self.left_x;
        let mut y = self.left_y; // Bevy: up = +1.0, matches game coords
        if x.abs() < deadzone {
            x = 0.0;
        }
        if y.abs() < deadzone {
            y = 0.0;
        }
        if self.dpad_x != 0 {
            x = self.dpad_x as f32;
        }
        if self.dpad_y != 0 {
            y = self.dpad_y as f32; // Bevy: up = +1, matches game coords
        }
        Vec2::new(x, y)
    }

    pub fn aim_direction(&self) -> Option<Vec2> {
        self.aim_direction_with_deadzone(0.3)
    }

    pub fn aim_direction_with_deadzone(&self, deadzone: f32) -> Option<Vec2> {
        let aim = Vec2::new(self.right_x, self.right_y); // Bevy: up = +1.0
        if aim.length() > deadzone {
            Some(aim.normalize())
        } else {
            None
        }
    }

    pub fn l4_just_pressed(&self) -> bool {
        self.just_pressed(13)
    }
    pub fn l5_just_pressed(&self) -> bool {
        self.just_pressed(15)
    }
    pub fn r4_just_pressed(&self) -> bool {
        self.just_pressed(14)
    }
    pub fn r5_just_pressed(&self) -> bool {
        self.just_pressed(10) || self.just_pressed(11)
    }

    pub fn fire(&self) -> bool {
        self.aim_direction().is_some()
    }
    pub fn right_trigger_pressed(&self) -> bool {
        self.right_trigger > 0.1
    }
    pub fn context_action(&self) -> bool {
        self.buttons[0]
    }
    pub fn emergency_burn(&self) -> bool {
        self.buttons[1]
    }
    pub fn formation_switch(&self) -> bool {
        self.just_pressed(3)
    }
    pub fn confirm(&self) -> bool {
        self.just_pressed(0)
    }
    pub fn back(&self) -> bool {
        self.just_pressed(1)
    }
    pub fn start(&self) -> bool {
        self.just_pressed(7) || self.just_pressed(9)
    }
    pub fn left_bumper(&self) -> bool {
        self.buttons[4]
    }
    pub fn right_bumper(&self) -> bool {
        self.just_pressed(5)
    }
    pub fn salt_miner(&self) -> bool {
        self.just_pressed(3)
    }
    pub fn x_button(&self) -> bool {
        self.just_pressed(2)
    }
    pub fn y_button(&self) -> bool {
        self.just_pressed(3)
    }
    pub fn left_trigger_pressed(&self) -> bool {
        self.left_trigger > 0.1
    }
}

// =============================================================================
// PLUGIN
// =============================================================================

pub struct JoystickPlugin;

impl Plugin for JoystickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<JoystickState>()
            .init_resource::<RumbleSettings>()
            .init_resource::<ControllerProfile>()
            .init_resource::<BackButtonConfig>()
            .add_event::<RumbleRequest>()
            .add_event::<BackButtonEvent>()
            .add_systems(PreUpdate, (detect_gamepad, poll_gamepad).chain())
            .add_systems(Update, (process_rumble_requests, process_back_buttons));
    }
}

// =============================================================================
// BEVY GAMEPAD POLLING (cross-platform: Linux, Windows, macOS, WASM)
// =============================================================================

/// Detect newly connected gamepads and set up profile
fn detect_gamepad(
    mut state: ResMut<JoystickState>,
    mut profile: ResMut<ControllerProfile>,
    gamepads: Query<(Entity, &Gamepad)>,
    names: Query<&Name>,
) {
    let count = gamepads.iter().count();

    if count > 0 && !state.connected {
        // First detection
        if let Some((entity, _gamepad)) = gamepads.iter().next() {
            state.connected = true;

            let name = names
                .get(entity)
                .map(|n| n.as_str().to_string())
                .unwrap_or_else(|_| "Unknown Controller".to_string());

            info!("Gamepad connected: {} ({} gamepad(s) found)", name, count);
            *profile = ControllerProfile::from_name(&name);
        }
    } else if count == 0 && state.connected {
        state.connected = false;
        info!("Gamepad disconnected");
    }
}

/// Poll Bevy's gamepad state and write to JoystickState
fn poll_gamepad(mut state: ResMut<JoystickState>, gamepads: Query<&Gamepad>) {
    // Save previous state for edge detection
    state.prev_buttons = state.buttons;
    state.prev_dpad_x = state.dpad_x;
    state.prev_dpad_y = state.dpad_y;
    state.prev_left_y = state.left_y;

    // Use first connected gamepad
    let Some(gamepad) = gamepads.iter().next() else {
        // No gamepad connected — keep previous state zeroed
        if state.connected {
            state.connected = false;
            info!("Gamepad disconnected");
        }
        return;
    };

    state.connected = true;

    // Axes — store raw Bevy values (up = +1.0)
    // movement() and aim_direction() handle coordinate conversion
    state.left_x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
    state.left_y = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
    state.right_x = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
    state.right_y = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);

    state.left_trigger = gamepad.get(GamepadAxis::LeftZ).unwrap_or(0.0).max(0.0);
    state.right_trigger = gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0).max(0.0);

    // D-pad
    state.dpad_x = 0;
    state.dpad_y = 0;
    if gamepad.pressed(GamepadButton::DPadLeft) {
        state.dpad_x = -1;
    }
    if gamepad.pressed(GamepadButton::DPadRight) {
        state.dpad_x = 1;
    }
    if gamepad.pressed(GamepadButton::DPadUp) {
        state.dpad_y = 1; // Bevy: up = positive
    }
    if gamepad.pressed(GamepadButton::DPadDown) {
        state.dpad_y = -1; // Bevy: down = negative
    }

    // Buttons — mapped to our index convention:
    // 0=South(A), 1=East(B), 2=West(X), 3=North(Y),
    // 4=LeftTrigger(LB), 5=RightTrigger(RB),
    // 6=LeftTrigger2(LT as button), 7=Select(Back/View),
    // 8=unused, 9=Start(Menu),
    // 10=LeftThumb, 11=RightThumb
    state.buttons[0] = gamepad.pressed(GamepadButton::South); // A
    state.buttons[1] = gamepad.pressed(GamepadButton::East); // B
    state.buttons[2] = gamepad.pressed(GamepadButton::West); // X
    state.buttons[3] = gamepad.pressed(GamepadButton::North); // Y
    state.buttons[4] = gamepad.pressed(GamepadButton::LeftTrigger); // LB
    state.buttons[5] = gamepad.pressed(GamepadButton::RightTrigger); // RB
    state.buttons[6] = gamepad.pressed(GamepadButton::LeftTrigger2); // LT as button
    state.buttons[7] = gamepad.pressed(GamepadButton::Select); // Back/View
    state.buttons[9] = gamepad.pressed(GamepadButton::Start); // Start/Menu
    state.buttons[10] = gamepad.pressed(GamepadButton::LeftThumb); // L3
    state.buttons[11] = gamepad.pressed(GamepadButton::RightThumb); // R3
}

// =============================================================================
// RUMBLE & BACK BUTTON PROCESSING
// =============================================================================

fn process_back_buttons(
    state: Res<JoystickState>,
    config: Res<BackButtonConfig>,
    profile: Res<ControllerProfile>,
    mut events: EventWriter<BackButtonEvent>,
) {
    if profile.controller_type != ControllerType::SteamDeck
        && profile.controller_type != ControllerType::Unknown
    {
        return;
    }

    if state.l4_just_pressed() && config.l4 != BackButtonAction::None {
        events.send(BackButtonEvent { action: config.l4 });
    }
    if state.l5_just_pressed() && config.l5 != BackButtonAction::None {
        events.send(BackButtonEvent { action: config.l5 });
    }
    if state.r4_just_pressed() && config.r4 != BackButtonAction::None {
        events.send(BackButtonEvent { action: config.r4 });
    }
    if state.r5_just_pressed() && config.r5 != BackButtonAction::None {
        events.send(BackButtonEvent { action: config.r5 });
    }
}

fn process_rumble_requests(
    mut rumble_events: EventReader<RumbleRequest>,
    mut rumble_writer: EventWriter<GamepadRumbleRequest>,
    gamepads: Query<Entity, With<Gamepad>>,
    rumble_settings: Res<RumbleSettings>,
) {
    if rumble_settings.intensity <= 0.001 {
        rumble_events.clear();
        return;
    }

    let multiplier = rumble_settings.intensity;

    for request in rumble_events.read() {
        let (strong, weak, duration_ms) = request.rumble_type.params();
        let strong = (strong * multiplier).clamp(0.0, 1.0);
        let weak = (weak * multiplier).clamp(0.0, 1.0);

        for gamepad_entity in gamepads.iter() {
            rumble_writer.send(GamepadRumbleRequest::Add {
                gamepad: gamepad_entity,
                intensity: GamepadRumbleIntensity {
                    strong_motor: strong,
                    weak_motor: weak,
                },
                duration: Duration::from_millis(duration_ms),
            });
        }
    }
}
