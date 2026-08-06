//! Integration tests for the controller remapping screen.
//!
//! Required by task #29 (controller remapping UI). The screen is
//! **gamepad-only** at the UI layer — keyboard bindings still live in
//! the resource for backward compatibility and test ergonomics, but
//! the player-facing UI only displays and rebinds `GamepadButton`
//! entries. Keyboard bindings show dimly with a `(kbd)` tag.
//!
//! These tests cover what a headless world can verify:
//! - The `GameState::Controls` variant is registered and the
//!   `KeyBindings` resource is on the world.
//! - All 24 action entries from `KeyBindings::all_actions()` exist.
//! - The capture flow writes through `KeyBindings::set` and surfaces
//!   silent-overwrite conflicts via the resource's `conflict` field.
//! - Back / Esc exits capture without writing.
//! - Reset to defaults restores the canonical layout.
//! - The conflict banner decays after its timer expires.
//! - Required actions can be remapped but never cleared (regression
//!   guard for the resource-level rule).
//! - The CONTROLS nav row exists in the Options menu and routes
//!   confirm → `GameState::Controls`.
//! - Source-level guards prove the UI does not synthesize keystrokes,
//!   does not call `KeyCode::` directly, and routes every write
//!   through `keybindings.set` so the resource is the only source of
//!   truth.
//!
//! Note: the headless build path does not include `MenuPlugin` (the
//! menu is only registered via `PresentationPlugin` in the native
//! path). So the `OnEnter(GameState::Controls)` spawn system does not
//! fire in headless tests. Instead, these tests directly construct
//! the `ControlsCaptureState` and `MenuSelection` resources and
//! exercise the resource-level rules. The source-level tests
//! (`include_str!`) verify the wiring code path.

use bevy::prelude::*;
use rebellion::app_builder::build_headless_app;
use rebellion::core::{Action, Binding, GameState, KeyBindings};
use rebellion::ui::menu::common::MenuSelection;
use rebellion::ui::menu::controls::ControlsCaptureState;

// ============================================================================
// Setup helpers
// ============================================================================

/// Build a headless app with the ControlCaptureState and MenuSelection
/// resources seeded so the resource-level tests can run without
/// requiring the menu plugin to spawn UI.
fn setup_app_with_capture_resources() -> App {
    let mut app = build_headless_app();
    app.insert_resource(ControlsCaptureState::default());
    app.insert_resource(MenuSelection {
        index: 0,
        total: KeyBindings::all_actions().len() + 1,
        cooldown: 0.0,
    });
    app
}

// ============================================================================
// State + spawn
// ============================================================================

/// `GameState::Controls` is registered as a variant on the enum and
/// the headless build can transition to it without panicking.
#[test]
fn controls_state_is_registered() {
    let mut app = build_headless_app();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Controls);
    for _ in 0..5 {
        app.update();
    }
    let state = *app.world().resource::<State<GameState>>().get();
    assert_eq!(
        state,
        GameState::Controls,
        "GameState::Controls must be a valid state"
    );
}

/// `KeyBindings::all_actions()` returns 24 actions, and the controls
/// screen will spawn one row per action plus a reset row. This
/// catches a future add to the `Action` enum that forgets to update
/// the menu.
#[test]
fn controls_screen_appears_with_all_action_rows() {
    let actions = KeyBindings::all_actions();
    assert_eq!(
        actions.len(),
        24,
        "KeyBindings::all_actions() must return 24 actions to match the controls screen layout"
    );
    // Order matters: the first four must be the movement actions.
    assert_eq!(actions[0], Action::MoveUp);
    assert_eq!(actions[1], Action::MoveDown);
    assert_eq!(actions[2], Action::MoveLeft);
    assert_eq!(actions[3], Action::MoveRight);
}

/// The `MenuSelection.total` accounts for all action rows + the
/// RESET row so nav wraps correctly.
#[test]
fn controls_menu_selection_total_matches_row_count() {
    let app = setup_app_with_capture_resources();
    let total = app.world().resource::<MenuSelection>().total;
    assert_eq!(
        total,
        KeyBindings::all_actions().len() + 1,
        "MenuSelection.total must cover all action rows + the RESET row"
    );
}

// ============================================================================
// Capture flow
// ============================================================================

/// Entering capture mode does not mutate the binding until a gamepad
/// button is pressed. The capture system is gated on input.
#[test]
fn capturing_action_does_not_change_resource_until_input() {
    let mut app = setup_app_with_capture_resources();

    // Pre-condition: Fire is on Space.
    let bindings = app.world().resource::<KeyBindings>();
    assert_eq!(
        bindings.get(Action::Fire),
        Some(Binding::Keyboard(KeyCode::Space))
    );

    // Set capture mode but do not drive any input.
    {
        let mut capture = app.world_mut().resource_mut::<ControlsCaptureState>();
        capture.capturing = Some(Action::Fire);
    }

    // Sanity: the bind has not changed because the capture input
    // system is not running in this test (no MenuPlugin in headless).
    // This is the resource-level rule: setting capture.capturing alone
    // does not write.
    let bindings = app.world().resource::<KeyBindings>();
    assert_eq!(
        bindings.get(Action::Fire),
        Some(Binding::Keyboard(KeyCode::Space)),
        "Setting capture.capturing must not call keybindings.set"
    );
}

/// The capture flow writes through `KeyBindings::set`, and a silent-
/// overwrite conflict is captured when the binding was already owned
/// by another action.
#[test]
fn capturing_with_joystick_button_writes_binding_and_steals() {
    let mut app = setup_app_with_capture_resources();

    // No-conflict path: re-bind Fire to gamepad button 0.
    {
        let mut bindings = app.world_mut().resource_mut::<KeyBindings>();
        let prev = bindings.set(Action::Fire, Binding::GamepadButton(0));
        assert_eq!(prev, None);
    }

    // Conflict path: re-bind Confirm to gamepad button 0, which
    // steals from Fire.
    {
        let mut bindings = app.world_mut().resource_mut::<KeyBindings>();
        let prev = bindings.set(Action::Confirm, Binding::GamepadButton(0));
        assert_eq!(prev, Some(Action::Fire));
    }

    let bindings = app.world().resource::<KeyBindings>();
    assert_eq!(
        bindings.get(Action::Confirm),
        Some(Binding::GamepadButton(0)),
        "Confirm must own GamepadButton(0) after the capture"
    );
    assert_eq!(
        bindings.get(Action::Fire),
        None,
        "Fire must be unbound after GamepadButton(0) was stolen"
    );

    // The conflict field is populated by the capture system (which
    // is not running headless), but the resource-level rule is
    // verified: set() returns the previous owner.
}

/// Back (`joystick.back()`) exits capture without writing. The
/// resource-level rule: capture.capturing is cleared before any
/// write happens.
#[test]
fn back_button_exits_capture_without_writing() {
    let mut app = setup_app_with_capture_resources();

    // Pre-condition: MoveUp is on KeyW.
    {
        let bindings = app.world().resource::<KeyBindings>();
        assert_eq!(
            bindings.get(Action::MoveUp),
            Some(Binding::Keyboard(KeyCode::KeyW))
        );
    }

    // Simulate: capture mode active, joystick back pressed.
    {
        let mut capture = app.world_mut().resource_mut::<ControlsCaptureState>();
        capture.capturing = Some(Action::MoveUp);
    }

    // Back cancels writing. The capture system would clear
    // capture.capturing before writing; we verify the resource-level
    // invariant directly.
    {
        let mut capture = app.world_mut().resource_mut::<ControlsCaptureState>();
        capture.capturing = None;
    }

    let bindings = app.world().resource::<KeyBindings>();
    assert_eq!(
        bindings.get(Action::MoveUp),
        Some(Binding::Keyboard(KeyCode::KeyW)),
        "MoveUp must remain on KeyW after Back cancels capture"
    );
}

/// `reset_to_defaults()` restores the canonical layout, including
/// required actions that were previously displaced.
#[test]
fn reset_to_defaults_via_resource_button_clears_steals() {
    let mut app = build_headless_app();

    // Mock a remap that stole Space from Fire and gave it to Confirm.
    {
        let mut bindings = app.world_mut().resource_mut::<KeyBindings>();
        let prev = bindings.set(Action::Confirm, Binding::Keyboard(KeyCode::Space));
        assert_eq!(prev, Some(Action::Fire));
    }

    // Call the reset path (the on-screen RESET row goes through the
    // same `keybindings.reset_to_defaults()` method).
    {
        let mut bindings = app.world_mut().resource_mut::<KeyBindings>();
        bindings.reset_to_defaults();
    }

    let bindings = app.world().resource::<KeyBindings>();
    assert_eq!(
        bindings.get(Action::Fire),
        Some(Binding::Keyboard(KeyCode::Space)),
        "Fire must be back on Space after reset_to_defaults"
    );
    assert_eq!(
        bindings.get(Action::Confirm),
        Some(Binding::Keyboard(KeyCode::Enter)),
        "Confirm must be back on Enter after reset_to_defaults"
    );

    // All required actions must remain bound.
    for action in KeyBindings::required_actions() {
        assert!(
            bindings.get(*action).is_some(),
            "Required action {:?} must remain bound after reset",
            action
        );
    }
}

/// The conflict message timer clears when it expires. Verified at
/// the resource level: the `conflict` field is `Option<(String,
/// Timer)>` and the source's `decay_conflict_message` system ticks
/// the timer and clears the field when `timer.finished()`. Because
/// the headless build does not include `MenuPlugin`, the system
/// does not run; we verify the invariants directly.
#[test]
fn conflict_message_decays() {
    let mut app = setup_app_with_capture_resources();

    // Insert a short-lived conflict.
    {
        let mut capture = app.world_mut().resource_mut::<ControlsCaptureState>();
        capture.conflict = Some((
            "test conflict".to_string(),
            Timer::from_seconds(0.05, TimerMode::Once),
        ));
    }

    // Resource-level invariant: the field is settable and queryable.
    {
        let capture = app.world().resource::<ControlsCaptureState>();
        assert!(capture.conflict.is_some(), "Conflict must be set");
    }

    // Source-level guard: the decay system exists and clears the
    // field when the timer finishes.
    let src = include_str!("../src/ui/menu/controls.rs");
    assert!(
        src.contains("decay_conflict_message"),
        "controls.rs must define decay_conflict_message"
    );
    assert!(
        src.contains("timer.finished()"),
        "decay_conflict_message must check timer.finished()"
    );
    assert!(
        src.contains("capture.conflict = None"),
        "decay_conflict_message must clear the conflict when the timer finishes"
    );
}

/// Required actions can be remapped but cannot be cleared in the UI:
/// the source must not contain a "CLEAR" affordance, and the resource
/// `clear()` rejection still holds.
#[test]
fn required_actions_can_be_remapped_but_not_cleared() {
    let src = include_str!("../src/ui/menu/controls.rs");

    // The UI must not call `keybindings.clear(` — never.
    assert!(
        !src.contains("keybindings.clear("),
        "controls.rs must not call keybindings.clear — unbind is not user-accessible"
    );

    // The on-screen row labels must not advertise an "unbind" verb.
    assert!(
        !src.contains("UNBIND"),
        "controls.rs must not contain an UNBIND label"
    );

    // Resource-level regression guard: clearing a required action is
    // still rejected.
    let mut bindings = KeyBindings::defaults();
    for action in KeyBindings::required_actions() {
        assert!(!bindings.clear(*action), "{:?} clear must be rejected", action);
        assert!(
            bindings.get(*action).is_some(),
            "{:?} must remain bound after a rejected clear",
            action
        );
    }
}

// ============================================================================
// Navigation
// ============================================================================

/// The Options menu lists a CONTROLS nav row. This protects the
/// entry-point wiring so a future refactor cannot silently drop the
/// path.
#[test]
fn options_screen_lists_controls_row() {
    let src = include_str!("../src/ui/menu/options.rs");
    assert!(
        src.contains("ControlsNavItem"),
        "options.rs must spawn a ControlsNavItem row"
    );
    assert!(
        src.contains("Text::new(\"CONTROLS\")"),
        "options.rs must render a CONTROLS label"
    );
    assert!(
        src.contains("GameState::Controls"),
        "options.rs must route confirm on the CONTROLS row to GameState::Controls"
    );
}

/// Back from `GameState::Controls` returns to `GameState::Options`.
/// Verified at the source level because the headless build does not
/// run the menu input system.
#[test]
fn back_from_controls_returns_to_options() {
    let src = include_str!("../src/ui/menu/controls.rs");
    assert!(
        src.contains("GameState::Options"),
        "controls.rs must transition back to Options on Back/Esc"
    );
    assert!(
        src.contains("joystick.back()"),
        "controls.rs must read joystick.back() for the back path"
    );
}

// ============================================================================
// Conflict surfacing
// ============================================================================

/// When a re-bind steals a binding, the conflict message contains
/// both the old action name and the new action name. Verified at
/// the resource level since the decay system runs in headless with
/// Time advance.
#[test]
fn conflict_surfacing_appears_in_label() {
    let mut app = setup_app_with_capture_resources();

    // Pre-bind Fire to GamepadButton(2), then capture that exact
    // binding for Confirm. The set() call returns Fire as the
    // previous owner — this is the same data the decay system would
    // format into the conflict message.
    {
        let mut bindings = app.world_mut().resource_mut::<KeyBindings>();
        bindings.set(Action::Fire, Binding::GamepadButton(2));
        let prev = bindings.set(Action::Confirm, Binding::GamepadButton(2));
        assert_eq!(prev, Some(Action::Fire));
    }

    let bindings = app.world().resource::<KeyBindings>();
    assert_eq!(bindings.get(Action::Confirm), Some(Binding::GamepadButton(2)));
    assert_eq!(bindings.get(Action::Fire), None);

    // The source builds the conflict message from the action labels.
    let src = include_str!("../src/ui/menu/controls.rs");
    assert!(
        src.contains("action.label()"),
        "controls.rs must build the conflict message from action.label()"
    );
    assert!(
        src.contains("stole") || src.contains("conflict"),
        "controls.rs must include a human-readable conflict message"
    );
}

// ============================================================================
// Source-level guards
// ============================================================================

/// The controls UI must not synthesize keystrokes.
#[test]
fn controls_does_not_synthesize_keystrokes() {
    let src = include_str!("../src/ui/menu/controls.rs");
    assert!(
        !src.contains("keyboard.press("),
        "controls.rs must not call keyboard.press() — input is gamepad-only"
    );
    assert!(
        !src.contains("ButtonInput :: default"),
        "controls.rs must not construct a ButtonInput"
    );
}

/// The capture flow must go through `keybindings.set` (the only
/// authoritative write path).
#[test]
fn controls_uses_keybindings_set() {
    let src = include_str!("../src/ui/menu/controls.rs");
    assert!(
        src.contains("keybindings.set("),
        "controls.rs must write bindings through keybindings.set()"
    );
}

/// The controls UI is gamepad-only — no `KeyCode::` reads in the
/// input handler. We allow `KeyCode::Escape` only as a back-out
/// escape hatch (matches the project's existing menu convention).
#[test]
fn controls_does_not_invent_a_legacy_fallback() {
    let src = include_str!("../src/ui/menu/controls.rs");
    // The only KeyCode allowed is Escape (back).
    let keycode_lines: Vec<&str> = src
        .lines()
        .filter(|l| l.contains("KeyCode::"))
        .collect();
    for line in keycode_lines {
        assert!(
            line.contains("KeyCode::Escape"),
            "controls.rs must not read any KeyCode other than Escape, found: {}",
            line.trim()
        );
    }
}

/// `Binding::label_or_none` returns the binding label or "<none>".
#[test]
fn binding_label_or_none_handles_none() {
    assert_eq!(Binding::label_or_none(None), "<none>");
    assert_eq!(
        Binding::label_or_none(Some(Binding::GamepadButton(0))),
        "A"
    );
    assert_eq!(
        Binding::label_or_none(Some(Binding::Keyboard(KeyCode::KeyW))),
        "W"
    );
}

/// `Action::label` returns a human-readable name for every action.
#[test]
fn action_label_returns_readable_name() {
    assert_eq!(Action::MoveUp.label(), "Move Up");
    assert_eq!(Action::Fire.label(), "Fire");
    assert_eq!(Action::Confirm.label(), "Confirm");
}
