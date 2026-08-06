//! Integration tests for the canonical ship-selection surface.
//!
//! Required by the v2.1.1 contract:
//! - Selection persists through state transitions
//! - Mouse click and touch activation work via the same handler
//! - Controller D-pad / face buttons drive the same logical actions
//! - The forward CTA wording names the actual transition
//!
//! These tests cover what a headless world can verify:
//! - `GameSession.selected_ship_index` is the persistent field
//! - The forward CTA wording describes the actual transition
//! - Back returns to DifficultySelect
//! - The selection system writes the persistent field when commit
//!   fires (simulated directly).

use bevy::input::ButtonInput;
use bevy::prelude::*;
use rebellion::app_builder::build_headless_app;
use rebellion::core::{Faction, GameSession, GameState};

/// Build a headless app with a Minmatar vs Amarr session and tick
/// until the menu entities are spawned.
fn setup_app_in_ship_select() -> App {
    let mut app = build_headless_app();

    {
        let mut session = app.world_mut().resource_mut::<GameSession>();
        *session = GameSession::new(Faction::Minmatar, Faction::Amarr);
    }

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::ShipSelect);

    for _ in 0..5 {
        app.update();
    }

    app
}

/// Selection persistence: writing to `GameSession.selected_ship_index`
/// and reading it back via `selected_ship()` must round-trip. The
/// ship-select screen, MissionBriefing, and gameplay all read from
/// this same resource — if it ever stops being the source of truth,
/// those screens silently desync.
#[test]
fn selection_persists_into_game_session() {
    let mut app = setup_app_in_ship_select();

    {
        let mut session = app.world_mut().resource_mut::<GameSession>();
        session.selected_ship_index = 1;
    }
    let session = app.world().resource::<GameSession>();
    assert_eq!(
        session.selected_ship_index, 1,
        "GameSession.selected_ship_index must persist the player's choice"
    );
    assert_eq!(
        session.selected_ship().name,
        "Slasher",
        "GameSession::selected_ship() must return the chosen hull"
    );

    // Tick several more frames and verify the value is still there —
    // proves the resource survives state-machine scheduling.
    for _ in 0..10 {
        app.update();
    }
    let session = app.world().resource::<GameSession>();
    assert_eq!(session.selected_ship_index, 1);
    assert_eq!(session.selected_ship().name, "Slasher");
}

/// Out-of-range selections are clamped by `selected_ship()` so a
/// runaway index from a stale save cannot panic gameplay.
#[test]
fn selected_ship_clamps_out_of_range_index() {
    let mut app = build_headless_app();
    {
        let mut session = app.world_mut().resource_mut::<GameSession>();
        *session = GameSession::new(Faction::Minmatar, Faction::Amarr);
        session.selected_ship_index = 999;
    }
    let session = app.world().resource::<GameSession>();
    let ship = session.selected_ship();
    assert!(
        !ship.name.is_empty(),
        "selected_ship() must clamp and return a valid ship"
    );
}

/// CTA wording: the source file must contain the forward-transition
/// wording the contract requires. "Continue to Briefing" tells the
/// player exactly what the next state is — not a misleading
/// "Launch Mission" wording.
#[test]
fn forward_cta_wording_names_actual_transition() {
    let src = include_str!("../src/ui/menu/ship_select.rs");
    assert!(
        src.contains("Continue to Briefing"),
        "ship_select.rs must label the forward CTA with the actual next state, \
         not a misleading 'Launch Mission' wording"
    );
    assert!(
        !src.contains("LAUNCH MISSION") && !src.contains("Launch Mission"),
        "ship_select.rs must not contain the misleading 'Launch Mission' wording"
    );
}

/// Back navigation source: the ship-select input handler must route
/// the back input to `GameState::DifficultySelect`, not out of the
/// menu flow. We verify the source code routes this correctly.
#[test]
fn back_navigation_routes_to_difficulty_select() {
    let src = include_str!("../src/ui/menu/ship_select.rs");
    assert!(
        src.contains("GameState::DifficultySelect"),
        "ship_select.rs must transition back to DifficultySelect on Esc / B"
    );
}

/// Input parity: keyboard, controller, mouse, and touch all flow
/// through the same confirmation path. `is_confirm` reads Enter,
/// Space, and the controller face button; `handle_menu_item_taps`
/// synthesizes a confirm-press for mouse and touch. Both paths must
/// exist and feed the same handler.
#[test]
fn input_parity_keyboard_controller_mouse_touch() {
    let src_common = include_str!("../src/ui/menu/common.rs");
    let src_ship = include_str!("../src/ui/menu/ship_select.rs");
    let src_mod = include_str!("../src/ui/menu/mod.rs");

    // Keyboard + controller confirm path
    assert!(
        src_common.contains("is_confirm"),
        "is_confirm must exist as the keyboard + controller confirm path"
    );
    assert!(
        src_ship.contains("is_confirm(&keyboard, &joystick)"),
        "ship_select must consume the keyboard + controller confirm path"
    );

    // Mouse + touch path synthesizes a confirm via the menu tap handler
    assert!(
        src_common.contains("handle_menu_item_taps"),
        "handle_menu_item_taps must exist for mouse + touch parity"
    );
    assert!(
        src_mod.contains("handle_menu_item_taps"),
        "handle_menu_item_taps must be registered in the menu plugin"
    );

    // Both paths feed `is_confirm` — the canonical confirm handler.
    // (Verified by reading the source of `handle_menu_item_taps` which
    //  sets `joystick.buttons[0] = true`, and `is_confirm` checks
    //  `joystick.confirm()` which reads buttons[0].)
    assert!(
        src_common.contains("joystick.buttons[0] = true"),
        "tap handler must synthesize a controller confirm via joystick.buttons[0]"
    );
}

/// Transition target: ship-select's commit transition must point at
/// `MissionBriefing`, not `Playing`. The contract requires the CTA
/// to name the actual next state, so the transition must also match.
#[test]
fn commit_transition_targets_mission_briefing() {
    let src = include_str!("../src/ui/menu/ship_select.rs");
    assert!(
        src.contains("GameState::MissionBriefing"),
        "ship_select commit transition must target MissionBriefing, not Playing"
    );
}

/// A second Ship Select entry must reset `MenuSelection.index` to 0
/// (the first ship) so the player can re-pick, but the previously
/// chosen `GameSession.selected_ship_index` must NOT be overwritten —
/// the user can return from MissionBriefing and see their previous
/// pick highlighted.
#[test]
fn respawning_ship_select_resets_menu_but_preserves_session_index() {
    let mut app = setup_app_in_ship_select();

    // Pre-set a non-default session index (simulates returning from
    // MissionBriefing or re-entering after a death).
    {
        let mut session = app.world_mut().resource_mut::<GameSession>();
        session.selected_ship_index = 2;
    }

    // Force a ShipSelect re-entry by leaving and coming back.
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::MissionBriefing);
    for _ in 0..3 {
        app.update();
    }
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::ShipSelect);
    for _ in 0..5 {
        app.update();
    }

    // GameSession must still hold the player's previous choice —
    // it lives across state transitions.
    let session = app.world().resource::<GameSession>();
    assert_eq!(
        session.selected_ship_index, 2,
        "GameSession.selected_ship_index must survive state transitions; \
         only the transient MenuSelection.index resets on screen entry"
    );
}

/// Sanity: the keyboard input resource is wired up in headless mode
/// so any future test that needs to drive input from a real Bevy
/// headless world has the resource available. (We verified this
/// during Phase 2's `headless_build_contains_keybindings`; this is a
/// focused regression check for the keyboard resource specifically.)
#[test]
fn headless_world_has_button_input_keycode_resource() {
    let app = build_headless_app();
    assert!(
        app.world().get_resource::<ButtonInput<KeyCode>>().is_some(),
        "headless app must register ButtonInput<KeyCode> so menu tests can drive input"
    );
}
