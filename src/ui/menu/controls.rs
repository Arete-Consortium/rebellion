//! Controller Remapping Screen
//!
//! Reached from Options. The player-facing UI is **gamepad-only**:
//! every row re-binds by capturing the next gamepad button press.
//! Keyboard bindings are shown as informational labels (dim, with a
//! `(kbd)` tag) but are not editable here. The `KeyBindings` resource
//! keeps both keyboard and gamepad paths for backward compatibility
//! and test ergonomics — see the locked scope call in the planning
//! notes for the rationale.
//!
//! The layout is intentionally single-file: a 24-action list + reset
//! row + transient conflict banner. If the screen grows beyond ~500
//! lines or the conflict / capture state machines each gain their own
//! subroutines, split into `controls/{state, layout, interaction,
//! capture}.rs` mirroring the frozen Phase-3 module-split pattern.

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::systems::JoystickState;
use bevy::prelude::*;

// ============================================================================
// Components
// ============================================================================

#[derive(Component)]
pub struct ControlsMenuRoot;

#[derive(Component)]
pub struct ControlsRow {
    pub action: Action,
    pub index: usize,
}

#[derive(Component)]
pub struct ControlsActionLabel;

#[derive(Component)]
pub struct ControlsBindingLabel;

#[derive(Component)]
pub struct ControlsResetItem;

#[derive(Component)]
pub struct ControlsMessageText;

// ============================================================================
// Resource — capture-mode state machine
// ============================================================================

/// State for the capture flow within the Controls screen.
///
/// `capturing` holds the action being re-bound. While it is `Some`,
/// the capture system absorbs the next gamepad button press and writes
/// it through `KeyBindings::set`. Back/Esc clears the capture without
/// writing.
///
/// `conflict` carries a 2-second amber banner message when a re-bind
/// stole a binding from another action.
///
/// Exposed as `pub` so integration tests in `tests/integration_controls.rs`
/// can drive the capture state directly. The fields are also `pub` for
/// the same reason; the type is not a durable surface and can change
/// without breaking the public API beyond the Rust visibility rules.
#[derive(Resource, Default)]
pub struct ControlsCaptureState {
    pub capturing: Option<Action>,
    pub conflict: Option<(String, Timer)>,
}

const CONFLICT_BANNER_SECONDS: f32 = 2.0;
const CAPTURE_DEBOUNCE_SECONDS: f32 = 0.5;

// ============================================================================
// Spawn
// ============================================================================

pub(crate) fn spawn_controls_menu(mut commands: Commands, keybindings: Res<KeyBindings>) {
    commands.init_resource::<ControlsCaptureState>();

    // Build the row list outside the closure so we can also compute
    // the total count for `MenuSelection`.
    let actions = KeyBindings::all_actions();
    let total_items = actions.len() + 1; // +1 for the RESET row

    commands.insert_resource(MenuSelection {
        index: 0,
        total: total_items,
        cooldown: 0.0,
    });

    commands
        .spawn((
            ControlsMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(Val::Px(40.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.95)),
        ))
        .with_children(|parent| {
            // TITLE
            parent.spawn((
                Text::new("CONTROLS"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // SECTION HEADER
            parent.spawn((
                Text::new("GAMEPAD BINDINGS"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            // 24 action rows
            for (i, action) in actions.iter().enumerate() {
                spawn_action_row(parent, *action, i, &keybindings);
            }

            // RESET row
            spawn_reset_row(parent, actions.len());

            // FOOTER
            parent.spawn((
                Text::new("A Rebind  •  B Back  •  Enter / Space Activate"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(24.0)),
                    ..default()
                },
            ));

            // CONFLICT BANNER (initially empty)
            parent.spawn((
                ControlsMessageText,
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.6, 0.2)),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_action_row(
    parent: &mut ChildBuilder,
    action: Action,
    index: usize,
    keybindings: &KeyBindings,
) {
    let binding = keybindings.get(action);
    let (label_text, dim) = match binding {
        Some(Binding::Keyboard(k)) => (format!("{} (kbd)", key_label(k)), true),
        Some(other) => (other.label(), false),
        None => ("<none>".to_string(), true),
    };

    parent
        .spawn((
            ControlsRow { action, index },
            MenuItem { index },
            Node {
                width: Val::Px(540.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor(Color::srgba(0.3, 0.3, 0.4, 0.5)),
        ))
        .with_children(|row| {
            // Action label
            row.spawn((
                ControlsActionLabel,
                Text::new(action.label()),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Binding label
            row.spawn((
                ControlsBindingLabel,
                Text::new(label_text),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(if dim {
                    Color::srgb(0.5, 0.5, 0.5)
                } else {
                    Color::srgb(0.9, 0.9, 0.9)
                }),
            ));
        });
}

fn spawn_reset_row(parent: &mut ChildBuilder, index: usize) {
    parent
        .spawn((
            ControlsResetItem,
            MenuItem { index },
            Node {
                width: Val::Px(540.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::top(Val::Px(16.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.1, 0.05, 0.9)),
            BorderColor(Color::srgba(0.4, 0.3, 0.2, 0.5)),
        ))
        .with_children(|row| {
            row.spawn((
                Text::new("RESET TO DEFAULTS"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.7, 0.5)),
            ));
        });
}

// ============================================================================
// Input handlers
// ============================================================================

/// Navigation + back + confirm handler. When the capture resource is
/// active, this system returns early so the capture system can run
/// in isolation.
pub(crate) fn controls_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    time: Res<Time>,
    mut selection: ResMut<MenuSelection>,
    mut capture: ResMut<ControlsCaptureState>,
    mut keybindings: ResMut<KeyBindings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Capture gate: nav/confirm are inert while waiting for a button.
    if capture.capturing.is_some() {
        return;
    }

    let dt = time.delta_secs();
    selection.cooldown = (selection.cooldown - dt).max(0.0);

    // Navigation
    if selection.cooldown <= 0.0 {
        let nav = get_nav_input(&keyboard, &joystick);
        if nav != 0 {
            let total = selection.total as i32;
            selection.index = (selection.index as i32 + nav).rem_euclid(total) as usize;
            selection.cooldown = MENU_NAV_COOLDOWN;
        }
    }

    // Confirm
    if selection.cooldown <= 0.0 && is_confirm(&keyboard, &joystick) {
        let actions = KeyBindings::all_actions();
        if selection.index < actions.len() {
            capture.capturing = Some(actions[selection.index]);
            selection.cooldown = CAPTURE_DEBOUNCE_SECONDS;
        } else {
            // RESET row
            keybindings.reset_to_defaults();
            selection.cooldown = CAPTURE_DEBOUNCE_SECONDS;
        }
    }

    // Back → Options
    if joystick.back() || keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Options);
    }
}

/// Capture-mode input handler. Writes the next gamepad button press
/// through `KeyBindings::set` and clears the capture state. Back/Esc
/// exits capture without writing.
pub(crate) fn controls_capture_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut capture: ResMut<ControlsCaptureState>,
    mut keybindings: ResMut<KeyBindings>,
) {
    let Some(action) = capture.capturing else {
        return;
    };

    // Back cancels writing.
    if joystick.back() || keyboard.just_pressed(KeyCode::Escape) {
        capture.capturing = None;
        return;
    }

    // Scan for the first rising-edge gamepad button.
    for i in 0..joystick.buttons.len() {
        if joystick.just_pressed(i) {
            let binding = Binding::GamepadButton(i as u8);
            let prev = keybindings.set(action, binding);
            if let Some(p) = prev {
                capture.conflict = Some((
                    format!(
                        "{} stole {} from {}",
                        action.label(),
                        binding.label(),
                        p.label()
                    ),
                    Timer::from_seconds(CONFLICT_BANNER_SECONDS, TimerMode::Once),
                ));
            }
            capture.capturing = None;
            return;
        }
    }
}

// ============================================================================
// Visual updates
// ============================================================================

/// Re-reads `KeyBindings` and writes each row's binding label, so a
/// successful re-bind or a reset is visible immediately. Dim color
/// (the `(kbd)` tag) is set at spawn time and not refreshed — the
/// game's `Action::GamepadButton` captures always produce a normal
/// gamepad binding, so the dim path is only reached at spawn or
/// after a reset that puts a required keyboard default back in
/// place.
pub(crate) fn refresh_binding_labels(
    keybindings: Res<KeyBindings>,
    rows: Query<(&ControlsRow, &Children), With<ControlsRow>>,
    mut labels: Query<&mut Text, With<ControlsBindingLabel>>,
) {
    for (row, children) in rows.iter() {
        let binding = keybindings.get(row.action);
        let label_text = match binding {
            Some(Binding::Keyboard(k)) => format!("{} (kbd)", key_label(k)),
            Some(other) => other.label(),
            None => "<none>".to_string(),
        };
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(*child) {
                **text = label_text.clone();
            }
        }
    }
}

/// Tick the conflict message timer and clear when it expires.
pub(crate) fn decay_conflict_message(
    time: Res<Time>,
    mut capture: ResMut<ControlsCaptureState>,
    mut messages: Query<&mut Text, With<ControlsMessageText>>,
) {
    let Some((ref msg, ref mut timer)) = capture.conflict else {
        return;
    };
    timer.tick(time.delta());
    if timer.finished() {
        capture.conflict = None;
        for mut text in messages.iter_mut() {
            **text = String::new();
        }
    } else {
        for mut text in messages.iter_mut() {
            **text = msg.clone();
        }
    }
}
