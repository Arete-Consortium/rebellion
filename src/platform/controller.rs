//! Controller-only player input and safe connection/reconnection handling.
//! Keyboard-driven headless fixtures opt in to this plugin when testing the
//! shipped input boundary; ordinary simulation tests keep their input helpers.
use crate::core::{Action, KeyBindings};
use crate::systems::JoystickState;
use bevy::prelude::*;

/// Fixed player layout. The Controls screen uses the same labels.
pub const CONTROL_GUIDE: &[(&str, &str)] = &[
    ("MOVE", "Left stick"),
    ("AIM", "Right stick"),
    ("FIRE", "RT"),
    ("HULL ABILITY", "LT"),
    ("THRUST", "LB"),
    ("DODGE", "RB + left stick"),
    ("AMMUNITION", "D-pad left / right"),
    ("INTERACT / CONFIRM", "A"),
    ("OVERLOAD", "Y"),
    ("PAUSE", "Menu"),
    ("BACK", "B"),
];

/// Resolve supported discrete actions independently of old keyboard saves.
pub fn action_pressed(action: Action, pad: &JoystickState, edge: bool) -> bool {
    let button = |i| {
        if edge {
            pad.just_pressed(i)
        } else {
            pad.buttons[i]
        }
    };
    match action {
        Action::Fire => pad.right_trigger_pressed(),
        Action::ActivateAbility => {
            if edge {
                pad.ability_just_pressed()
            } else {
                pad.left_trigger_pressed()
            }
        }
        Action::Confirm => button(0),
        Action::Cancel => button(1),
        Action::Pause => button(9),
        Action::CycleAmmoPrev | Action::MenuLeft => {
            if edge {
                pad.dpad_just_left()
            } else {
                pad.dpad_x < 0
            }
        }
        Action::CycleAmmoNext | Action::MenuRight => {
            if edge {
                pad.dpad_just_right()
            } else {
                pad.dpad_x > 0
            }
        }
        Action::MenuUp => {
            if edge {
                pad.dpad_just_up()
            } else {
                pad.dpad_y > 0
            }
        }
        Action::MenuDown => {
            if edge {
                pad.dpad_just_down()
            } else {
                pad.dpad_y < 0
            }
        }
        // Sticks are read as vectors; direct numbered ammunition selection is
        // replaced by cycling. These legacy actions are absent from the guide.
        _ => false,
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionPhase {
    #[default]
    ReleaseControls,
    Confirm,
    ReleaseConfirm,
    Playing,
}

#[derive(Resource, Default)]
pub struct ControllerGate {
    phase: ConnectionPhase,
    device: Option<Entity>,
    paused_clock: bool,
}

impl ControllerGate {
    pub fn is_blocked(&self) -> bool {
        self.phase != ConnectionPhase::Playing
    }
}

#[derive(Component)]
struct ConnectionOverlay;
#[derive(Component)]
struct ConnectionMessage;

pub struct ControllerOnlyPlugin;
impl Plugin for ControllerOnlyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControllerGate>()
            .add_systems(
                Update,
                super::pause_trigger_system.run_if(
                    in_state(crate::core::GameState::Playing)
                        .or(in_state(crate::core::GameState::BossFight)),
                ),
            )
            .add_systems(Startup, spawn_connection_overlay)
            .add_systems(
                PreUpdate,
                controller_boundary
                    .after(crate::systems::joystick::poll_gamepad)
                    .after(bevy::ui::UiSystem::Focus)
                    .before(crate::ui::menu::common::handle_menu_item_taps),
            )
            .add_systems(Update, update_connection_overlay);
    }
}

fn neutral(pad: &JoystickState, allow_confirm: bool) -> bool {
    pad.stick_movement() == Vec2::ZERO
        && pad.aim_direction().is_none()
        && pad.left_trigger <= 0.1
        && pad.right_trigger <= 0.1
        && pad.dpad_x == 0
        && pad.dpad_y == 0
        && pad
            .buttons
            .iter()
            .enumerate()
            .all(|(i, &held)| !held || (allow_confirm && i == 0))
}

#[allow(clippy::too_many_arguments)]
fn controller_boundary(
    mut gate: ResMut<ControllerGate>,
    mut pad: ResMut<JoystickState>,
    mut bindings: ResMut<KeyBindings>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mouse: Option<ResMut<ButtonInput<MouseButton>>>,
    mut interactions: Query<&mut Interaction>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut time: ResMut<Time>,
    mut windows: Query<&mut Window>,
) {
    // This is a runtime policy, never a save migration. Reassert after loading
    // an old profile so it cannot restore desktop controls in a player build.
    if !bindings.controller_only {
        bindings.controller_only = true;
    }
    keyboard.reset_all();
    if let Some(mut mouse) = mouse {
        mouse.reset_all();
    }
    for mut interaction in &mut interactions {
        if *interaction != Interaction::None {
            *interaction = Interaction::None;
        }
    }
    for mut window in &mut windows {
        if window.cursor_options.visible {
            window.cursor_options.visible = false;
        }
    }

    if !pad.connected || gate.device != pad.active_gamepad {
        gate.phase = ConnectionPhase::ReleaseControls;
        gate.device = pad.active_gamepad;
    }
    if pad.connected {
        gate.phase = match gate.phase {
            ConnectionPhase::ReleaseControls if neutral(&pad, false) => ConnectionPhase::Confirm,
            ConnectionPhase::Confirm if pad.confirm() && neutral(&pad, true) => {
                ConnectionPhase::ReleaseConfirm
            }
            ConnectionPhase::ReleaseConfirm if neutral(&pad, false) => ConnectionPhase::Playing,
            phase => phase,
        };
    }
    if gate.is_blocked() {
        if !gate.paused_clock {
            gate.paused_clock = !virtual_time.is_paused();
        }
        virtual_time.pause();
        // First has already advanced the frame clock. Zero this frame's delta
        // before FixedMainLoop too, so disconnect causes no final damage tick.
        virtual_time.advance_by(std::time::Duration::ZERO);
        *time = virtual_time.as_generic();
        let device = pad.active_gamepad;
        let connected = pad.connected;
        *pad = JoystickState {
            active_gamepad: device,
            connected,
            ..default()
        };
    } else if gate.paused_clock {
        virtual_time.unpause();
        gate.paused_clock = false;
    }
}

fn spawn_connection_overlay(mut commands: Commands) {
    commands
        .spawn((
            ConnectionOverlay,
            GlobalZIndex(10000),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(24.0),
                padding: UiRect::all(Val::Px(32.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.015, 0.025, 0.04)),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("REBELLION"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.85, 1.0)),
            ));
            p.spawn((
                ConnectionMessage,
                Text::new("CONNECT A CONTROLLER"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            p.spawn((
                Text::new("Left stick moves. Right stick aims. RT fires."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.7, 0.78)),
            ));
        });
}

fn update_connection_overlay(
    gate: Res<ControllerGate>,
    pad: Res<JoystickState>,
    mut roots: Query<&mut Node, With<ConnectionOverlay>>,
    mut labels: Query<&mut Text, With<ConnectionMessage>>,
) {
    for mut node in &mut roots {
        let display = if gate.is_blocked() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for mut text in &mut labels {
        let label = if !pad.connected {
            "CONNECT A CONTROLLER"
        } else {
            match gate.phase {
                ConnectionPhase::ReleaseControls => "RELEASE THE STICKS AND BUTTONS",
                ConnectionPhase::Confirm => "PRESS A TO CONTINUE",
                ConnectionPhase::ReleaseConfirm => "RELEASE A TO CONTINUE",
                ConnectionPhase::Playing => "",
            }
        };
        if **text != label {
            **text = label.into();
        }
    }
}
