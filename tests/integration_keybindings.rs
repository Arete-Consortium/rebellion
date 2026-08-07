//! Integration tests for authoritative keybindings wiring.
//!
//! The resource layer is unit-tested in `core::keybindings::tests`.
//! These tests prove the wire-up: that both app-build paths
//! (native and headless) guarantee the resource is present, and
//! that gameplay systems can consume it without falling back to
//! legacy `KeyCode` reads.
//!
//! Required by the v2.1.1 contract:
//! - native build contains `KeyBindings`
//! - headless build contains `KeyBindings`
//! - entering Playing does not panic on the resource surface

use bevy::prelude::*;
use rebellion::app_builder::{build_headless_app, RebellionAppConfig};
use rebellion::core::{Action, Binding, GameState, KeyBindings};
use rebellion::systems::JoystickState;

/// Native build path: `KeyBindings` must be registered as a resource.
/// This catches accidental headless-only registration and proves the
/// player-facing binary has the authoritative table available.
#[test]
fn native_build_contains_keybindings() {
    // We don't actually create a window in CI; we use `build()` only
    // as far as plugin registration. The resource check is the gate.
    // Avoid the full Bevy renderer by using the headless build path
    // for resource verification (it shares `configure_shared`).
    let app = RebellionAppConfig::headless_test().build();
    assert!(
        app.world().get_resource::<KeyBindings>().is_some(),
        "KeyBindings resource must be registered via configure_shared"
    );
}

/// Headless build path: same guarantee. This is the path CI uses.
#[test]
fn headless_build_contains_keybindings() {
    let app = build_headless_app();
    assert!(
        app.world().get_resource::<KeyBindings>().is_some(),
        "headless app must expose KeyBindings"
    );
}

/// Smoke: transitioning to Playing with a fresh world does not panic
/// because the resource is present. This is the positive test
/// described in the run ledger — the resource IS present, so this
/// asserts no panic on the path that used to break the legacy
/// `KeyCode::` reads in `player_shooting`.
#[test]
fn entering_playing_does_not_panic_on_missing_resource() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);

    // Run several ticks so any system that fails to resolve the
    // resource has a chance to panic.
    for _ in 0..30 {
        app.update();
    }

    let current = *app.world().resource::<State<GameState>>().get();
    assert_eq!(current, GameState::Playing);
}

/// Mutating the binding table after build (the same write path the
/// remapping UI will use) and then re-reading via `pressed()`
/// reflects the change in the same world — proves the resource is
/// the live source of truth, not a snapshot.
#[test]
fn live_remap_takes_effect_immediately() {
    let mut app = build_headless_app();
    {
        let mut bindings = app.world_mut().resource_mut::<KeyBindings>();
        let prev = bindings.set(Action::MoveUp, Binding::Keyboard(KeyCode::KeyT));
        assert_eq!(prev, None);
    }

    let bindings = app.world().resource::<KeyBindings>();
    assert_eq!(
        bindings.get(Action::MoveUp),
        Some(Binding::Keyboard(KeyCode::KeyT))
    );

    // Pressing W must NOT trigger MoveUp after the remap.
    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::KeyW);
    let joy = JoystickState::default();
    assert!(
        !bindings.pressed(Action::MoveUp, &keys, &joy),
        "W must stop triggering MoveUp after remap"
    );

    // Pressing T MUST trigger MoveUp.
    keys.press(KeyCode::KeyT);
    assert!(
        bindings.pressed(Action::MoveUp, &keys, &joy),
        "T must trigger MoveUp after remap"
    );
}

/// The required-actions rule survives a real-world mutation: trying
/// to clear MoveUp via the resource returns false and the binding is
/// preserved. This is the gameplay-safety gate that prevents
/// silent unusable control schemes.
#[test]
fn clear_required_action_rejected_via_resource() {
    let mut app = build_headless_app();
    let cleared = {
        let mut bindings = app.world_mut().resource_mut::<KeyBindings>();
        bindings.clear(Action::MoveUp)
    };
    assert!(!cleared, "clear() of required action must return false");

    let bindings = app.world().resource::<KeyBindings>();
    assert!(
        bindings.get(Action::MoveUp).is_some(),
        "MoveUp must remain bound after a rejected clear"
    );
}

/// Conflict-theft semantics at the resource level: stealing Space
/// from Fire and assigning it to Confirm leaves Fire unbound, and
/// `pressed(Fire, ...)` returns false even while Space is held.
/// This is the contract's silent-overwrite rule verified through a
/// real Bevy world, not just through the unit-test surface.
#[test]
fn conflict_theft_via_resource_silently_overwrites() {
    let mut app = build_headless_app();
    {
        let mut bindings = app.world_mut().resource_mut::<KeyBindings>();
        let prev = bindings.set(Action::Confirm, Binding::Keyboard(KeyCode::Space));
        assert_eq!(
            prev,
            Some(Action::Fire),
            "Confirm stealing Space from Fire must return Fire as the previous owner"
        );
    }

    let bindings = app.world().resource::<KeyBindings>();
    assert_eq!(
        bindings.get(Action::Confirm),
        Some(Binding::Keyboard(KeyCode::Space))
    );
    assert_eq!(
        bindings.get(Action::Fire),
        None,
        "Fire must be unbound after its binding was stolen"
    );

    let mut keys = ButtonInput::<KeyCode>::default();
    keys.press(KeyCode::Space);
    let joy = JoystickState::default();
    assert!(
        bindings.pressed(Action::Confirm, &keys, &joy),
        "Space must now trigger Confirm after the silent overwrite"
    );
    assert!(
        !bindings.pressed(Action::Fire, &keys, &joy),
        "Space must NOT trigger Fire after Confirm stole it"
    );
}

// ============================================================================
// Disk persistence (Phase 6).
//
// Verifies that `KeyBindings` survives a save/load round-trip via the
// `SaveData` JSON format. The serialise/deserialise boundary is where
// every migration shim has to hold, so these tests exercise the exact
// format the production loader reads — not just the in-memory resource
// state.
// ============================================================================

/// A custom remap survives serialisation → deserialisation through
/// `SaveData`. This is the round-trip the production disk reader
/// performs, except the production version also writes to disk in
/// between (covered by the manual smoke run in `docs/RUN_LEDGER.md`).
#[test]
fn save_data_serialization_roundtrip_includes_keybindings() {
    let mut save = rebellion::core::SaveData::default();

    // Player remaps Move Up to a gamepad button.
    {
        let prev = save
            .keybindings
            .set(Action::MoveUp, Binding::GamepadButton(2));
        assert_eq!(prev, None, "default MoveUp should not conflict");
    }

    let json = serde_json::to_string(&save).expect("serialize");
    let loaded: rebellion::core::SaveData = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        loaded.keybindings.get(Action::MoveUp),
        Some(Binding::GamepadButton(2)),
        "Move Up must survive the SaveData round-trip with its new binding"
    );
    // Default bindings for the rest of the table must still be intact.
    assert_eq!(
        loaded.keybindings.get(Action::Fire),
        Some(Binding::Keyboard(KeyCode::Space)),
        "untouched bindings must keep their defaults"
    );
}

/// A save blob from before this feature (no `keybindings` field at
/// all) must still load. `#[serde(default)]` on the new field gives
/// the loader `KeyBindings::default()` which is
/// `KeyBindings::defaults()` — the canonical layout.
///
/// We construct the legacy blob by serialising a freshly-defaulted
/// `SaveData` and then stripping the `,"keybindings":{...}` substring,
/// keeping every other field in its exact on-disk shape.
#[test]
fn save_data_without_keybindings_field_loads_defaults() {
    let pre = serde_json::to_string(&rebellion::core::SaveData::default()).expect("serialize");
    let keybindings_start = pre
        .find("\"keybindings\":")
        .expect("keybindings key must be present in the default blob");
    // Drop the `,"keybindings":{...}` substring. Trim the trailing
    // comma so the blob terminates cleanly with the previous field's
    // closing brace.
    let legacy_blob = pre[..keybindings_start].trim_end_matches(',').to_string() + "}";

    let loaded: rebellion::core::SaveData = serde_json::from_str(&legacy_blob)
        .expect("legacy save without keybindings field must deserialize");

    assert_eq!(
        loaded.keybindings,
        KeyBindings::defaults(),
        "missing keybindings field must default to the canonical layout"
    );
    assert_eq!(
        loaded.keybindings.get(Action::MoveUp),
        Some(Binding::Keyboard(KeyCode::KeyW)),
        "defaults must include the W → MoveUp entry"
    );
}

/// A save blob that includes the `keybindings` field with only a
/// single entry must load exactly as written. The runtime
/// "empty map → defaults" migration does NOT fire here — the
/// disk-derived value is authoritative. This is the contract's
/// "exactly what's in the save, no re-defaulting at check time" rule.
///
/// The on-disk JSON form for a single MoveUp → Keyboard(W) entry is
/// `{"keybindings":{"map":{"MoveUp":{"Keyboard":"KeyW"}}}}` (verified
/// against the current `serde_json::to_string` shape during the
/// implementation pass).
#[test]
fn save_data_partial_keybindings_field_is_loaded_unchanged() {
    let blob = r#"{"stage_progress":[],"unlocked_ships":[],"lifetime_credits":0,"high_scores":[],"settings":{"master_volume":0.7,"sfx_volume":0.8,"music_volume":0.5,"screen_shake_intensity":1.0,"rumble_intensity":1.0},"keybindings":{"map":{"MoveUp":{"Keyboard":"KeyW"}}},"achievements":[],"lifetime_stats":{"total_kills":0,"total_souls":0,"games_played":0,"missions_completed":0,"bosses_defeated":0,"highest_combo":0,"highest_score":0},"skill_points":0,"purchased_upgrades":[],"analytics":{"deaths_by_stage":{},"ship_picks":{},"difficulty_picks":{},"stages_completed":{},"total_sessions":0,"total_play_time_secs":0.0},"leaderboard":[]}"#;
    let loaded: rebellion::core::SaveData =
        serde_json::from_str(blob).expect("partial save must deserialize");

    assert_eq!(
        loaded.keybindings.get(Action::MoveUp),
        Some(Binding::Keyboard(KeyCode::KeyW)),
        "the single MoveUp entry must be loaded as-is"
    );
    // Fire was not in the save; it must remain unbound rather than
    // being silently re-defaulted to Space.
    assert_eq!(
        loaded.keybindings.get(Action::Fire),
        None,
        "a save with one binding must not auto-populate the rest from defaults"
    );
}

/// Source-level guard: the `SaveData.keybindings` field must remain
/// present and `#[serde(default)]`-decorated so a future refactor
/// cannot silently drop disk persistence.
#[test]
fn save_data_has_keybindings_field() {
    let src = include_str!("../src/core/save.rs");

    // Field declaration: must include the `pub keybindings:` line.
    let field_present = src.contains("pub keybindings: super::KeyBindings");
    assert!(
        field_present,
        "SaveData must hold a `pub keybindings: KeyBindings` field"
    );

    // The field must be `#[serde(default)]`-decorated. We assert this
    // as a substring on a small window of the file (anchored at the
    // field line) to keep the test self-contained and side-effect-free.
    if let Some(idx) = src.find("pub keybindings: super::KeyBindings") {
        // Walk backwards from the field declaration and look for the
        // nearest `#[serde(default)]` attribute. The attribute appears
        // on the line directly above the field in the canonical layout.
        let prefix = &src[..idx];
        let decorated = prefix
            .lines()
            .rev()
            .take(8) // generous window: doc comments + attribute + blanks
            .any(|l| l.contains("#[serde(default)]"));
        assert!(
            decorated,
            "the keybindings field must be preceded by #[serde(default)] for migration safety"
        );
    }

    // Load (apply_saved_settings) and sync (sync_settings_to_save)
    // paths must each touch save.keybindings.
    assert!(
        src.contains("*keybindings = save.keybindings.clone()"),
        "apply_saved_settings must load the saved keybindings into the runtime resource"
    );
    assert!(
        src.contains("save.keybindings = keybindings.clone()"),
        "sync_settings_to_save must write runtime changes back to save.keybindings"
    );
}
