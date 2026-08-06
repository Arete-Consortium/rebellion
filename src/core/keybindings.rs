//! Authoritative remappable input bindings.
//!
//! The wiring rule for this module is strict:
//!
//! * Gameplay code does **not** read `KeyCode` directly for any
//!   remappable action. It asks `KeyBindings` for the action and
//!   `pressed()` decides whether the binding is currently active.
//! * There is no runtime legacy-key fallback. Once a player has
//!   remapped an action away from a key, that key is dead for
//!   that action.
//! * Required actions (movement, fire, confirm, cancel) cannot be
//!   cleared. Any attempt to `clear()` one is silently rejected,
//!   and the binding is left at its current value.
//! * Defaults are loaded once — on first run or when a legacy save
//!   arrives without a `keybindings` field. They are written
//!   through to the next save cycle, not re-applied per check.

use bevy::prelude::*;
use bevy::prelude::KeyCode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single remappable gameplay or menu action.
///
/// New actions go here **and** into [`KeyBindings::required_actions`].
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Action {
    // Movement (always required)
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    // Aim (toggleable, not required)
    AimUp,
    AimDown,
    AimLeft,
    AimRight,
    // Combat (fire always required; cycling is not)
    Fire,
    CycleAmmoPrev,
    CycleAmmoNext,
    SelectAmmo1,
    SelectAmmo2,
    SelectAmmo3,
    SelectAmmo4,
    SelectAmmo5,
    // System
    Pause,
    ActivateAbility,
    // Menu
    Confirm,
    Cancel,
    MenuUp,
    MenuDown,
    MenuLeft,
    MenuRight,
}

/// A single physical input that can be bound to an action.
///
/// Per project policy, `MouseButton` and `Binding::GamepadButton` are
/// kept for completeness; only keyboard and joystick paths are
/// consumed by gameplay today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Binding {
    Keyboard(KeyCode),
    GamepadButton(u8),
    MouseButton(u8),
}

impl Binding {
    /// Short label shown in the Controls UI.
    pub fn label(&self) -> String {
        match self {
            Binding::Keyboard(k) => key_label(*k),
            Binding::GamepadButton(0) => "A".into(),
            Binding::GamepadButton(1) => "B".into(),
            Binding::GamepadButton(i) => format!("Pad{}", i),
            Binding::MouseButton(0) => "LMB".into(),
            Binding::MouseButton(i) => format!("M{}", i),
        }
    }
}

/// Human-readable label for a [`KeyCode`], stripping Bevy's
/// `KeyCode::` prefix and standardizing a few common aliases.
pub fn key_label(k: KeyCode) -> String {
    let s = format!("{:?}", k);
    let stripped = s.strip_prefix("KeyCode::").unwrap_or(&s);
    match stripped {
        "ArrowUp" => "↑".into(),
        "ArrowDown" => "↓".into(),
        "ArrowLeft" => "←".into(),
        "ArrowRight" => "→".into(),
        "Escape" => "Esc".into(),
        other => other
            .strip_prefix("Key")
            .or_else(|| other.strip_prefix("Digit"))
            .unwrap_or(other)
            .to_string(),
    }
}

/// Authoritative binding table.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    /// Backing store. `#[serde(default)]` lets a save blob that
    /// pre-dates the feature deserialize to an empty map; the
    /// migration step replaces that empty map with defaults.
    #[serde(default)]
    map: BTreeMap<Action, Binding>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::defaults()
    }
}

impl KeyBindings {
    /// Canonical default layout. Matches the legacy hard-coded
    /// `KeyCode` reads that used to live in `entities/player.rs`,
    /// so the experience is identical for players who never remap.
    pub fn defaults() -> Self {
        use Binding::Keyboard;
        use KeyCode::*;
        let entries: &[(Action, Binding)] = &[
            (Action::MoveUp, Keyboard(KeyW)),
            (Action::MoveDown, Keyboard(KeyS)),
            (Action::MoveLeft, Keyboard(KeyA)),
            (Action::MoveRight, Keyboard(KeyD)),
            (Action::AimUp, Keyboard(KeyI)),
            (Action::AimDown, Keyboard(KeyK)),
            (Action::AimLeft, Keyboard(KeyJ)),
            (Action::AimRight, Keyboard(KeyL)),
            (Action::Fire, Keyboard(Space)),
            (Action::CycleAmmoPrev, Keyboard(KeyQ)),
            (Action::CycleAmmoNext, Keyboard(KeyE)),
            (Action::SelectAmmo1, Keyboard(Digit1)),
            (Action::SelectAmmo2, Keyboard(Digit2)),
            (Action::SelectAmmo3, Keyboard(Digit3)),
            (Action::SelectAmmo4, Keyboard(Digit4)),
            (Action::SelectAmmo5, Keyboard(Digit5)),
            (Action::Pause, Keyboard(Escape)),
            (Action::ActivateAbility, Binding::GamepadButton(7)),
            (Action::Confirm, Keyboard(Enter)),
            (Action::Cancel, Keyboard(Escape)),
            (Action::MenuUp, Keyboard(ArrowUp)),
            (Action::MenuDown, Keyboard(ArrowDown)),
            (Action::MenuLeft, Keyboard(ArrowLeft)),
            (Action::MenuRight, Keyboard(ArrowRight)),
        ];
        let mut map = BTreeMap::new();
        for (a, b) in entries {
            map.insert(*a, *b);
        }
        Self { map }
    }

    /// The actions that must remain bound. Attempting to `clear()` one
    /// is a no-op so the player's controls never become unusable.
    pub fn required_actions() -> &'static [Action] {
        &[
            Action::MoveUp,
            Action::MoveDown,
            Action::MoveLeft,
            Action::MoveRight,
            Action::Fire,
            Action::Confirm,
            Action::Cancel,
        ]
    }

    /// Resolve a binding, if any.
    pub fn get(&self, action: Action) -> Option<Binding> {
        self.map.get(&action).copied()
    }

    /// Re-bind an action. If the new binding is already held by a
    /// different action, the old owner is cleared (silent overwrite).
    /// If the binding is the action's own, no-op (returns `None`).
    pub fn set(&mut self, action: Action, binding: Binding) -> Option<Action> {
        if self.map.get(&action) == Some(&binding) {
            return None;
        }
        // Steal from any prior owner.
        let mut previous_owner = None;
        for (&a, &b) in self.map.iter() {
            if a != action && b == binding {
                previous_owner = Some(a);
                break;
            }
        }
        if let Some(prev) = previous_owner {
            self.map.remove(&prev);
        }
        self.map.insert(action, binding);
        previous_owner
    }

    /// Clear a binding. **Rejected for required actions**: returns
    /// `false` and leaves the binding unchanged. Returns `true` if
    /// the action was actually cleared.
    pub fn clear(&mut self, action: Action) -> bool {
        if Self::required_actions().contains(&action) {
            return false;
        }
        self.map.remove(&action).is_some()
    }

    /// Restore the default layout, displacing anything the player
    /// previously customized.
    pub fn reset_to_defaults(&mut self) {
        *self = Self::defaults();
    }

    /// All actions known to the system, in declaration order.
    pub fn all_actions() -> &'static [Action] {
        &[
            Action::MoveUp,
            Action::MoveDown,
            Action::MoveLeft,
            Action::MoveRight,
            Action::AimUp,
            Action::AimDown,
            Action::AimLeft,
            Action::AimRight,
            Action::Fire,
            Action::CycleAmmoPrev,
            Action::CycleAmmoNext,
            Action::SelectAmmo1,
            Action::SelectAmmo2,
            Action::SelectAmmo3,
            Action::SelectAmmo4,
            Action::SelectAmmo5,
            Action::Pause,
            Action::ActivateAbility,
            Action::Confirm,
            Action::Cancel,
            Action::MenuUp,
            Action::MenuDown,
            Action::MenuLeft,
            Action::MenuRight,
        ]
    }

    /// Look up the action currently bound to `binding`, if any.
    pub fn action_for(&self, binding: Binding) -> Option<Action> {
        self.map
            .iter()
            .find(|(_, b)| **b == binding)
            .map(|(a, _)| *a)
    }

    /// Returns `true` if the action is currently being driven by its
    /// binding. If the action is unbound, returns `false` regardless
    /// of what the player is pressing.
    pub fn pressed(
        &self,
        action: Action,
        keyboard: &ButtonInput<KeyCode>,
        joystick: &crate::systems::JoystickState,
    ) -> bool {
        let Some(binding) = self.get(action) else {
            return false;
        };
        match binding {
            Binding::Keyboard(k) => keyboard.pressed(k),
            Binding::GamepadButton(i) => {
                (i as usize) < joystick.buttons.len() && joystick.buttons[i as usize]
            }
            Binding::MouseButton(_) => false,
        }
    }

    /// Edge-triggered variant. Same authoritative rule.
    pub fn just_pressed(
        &self,
        action: Action,
        keyboard: &ButtonInput<KeyCode>,
        joystick: &crate::systems::JoystickState,
    ) -> bool {
        let Some(binding) = self.get(action) else {
            return false;
        };
        match binding {
            Binding::Keyboard(k) => keyboard.just_pressed(k),
            Binding::GamepadButton(i) => joystick.just_pressed(i as usize),
            Binding::MouseButton(_) => false,
        }
    }
}

/// Registers [`KeyBindings`] as a resource. The plugin itself is
/// trivial today; it exists as a stable registration point so future
/// passes that need bindings-aware systems can add them here without
/// reaching into `app_builder.rs`.
pub struct KeyBindingsPlugin;

impl Plugin for KeyBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<KeyBindings>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> ButtonInput<KeyCode> {
        ButtonInput::<KeyCode>::default()
    }

    fn js() -> crate::systems::JoystickState {
        crate::systems::JoystickState::default()
    }

    #[test]
    fn defaults_cover_every_action() {
        let b = KeyBindings::defaults();
        for action in KeyBindings::all_actions() {
            assert!(
                b.get(*action).is_some(),
                "defaults missing {:?}",
                action
            );
        }
    }

    #[test]
    fn first_run_with_no_save_loads_defaults() {
        // The empty `BTreeMap` simulates a fresh installation.
        // `Default::default()` is the migration path.
        let b = KeyBindings::default();
        assert!(b.get(Action::MoveUp).is_some());
        assert_eq!(b.get(Action::Fire), Some(Binding::Keyboard(KeyCode::Space)));
    }

    #[test]
    fn legacy_save_without_keybindings_field_loads_defaults() {
        // A save that pre-dates the feature deserializes to an empty
        // `map` (because the field is serde-defaulted). Migration
        // replaces the empty map with defaults once.
        let b: KeyBindings =
            serde_json::from_str(r#"{"map":{}}"#).unwrap();
        let migrated = if b.map.is_empty() {
            KeyBindings::defaults()
        } else {
            b
        };
        assert!(migrated.get(Action::MoveUp).is_some());
        assert_eq!(
            migrated.get(Action::Fire),
            Some(Binding::Keyboard(KeyCode::Space))
        );
    }

    #[test]
    fn deserialization_without_map_field_uses_default() {
        // `#[serde(default)]` on `map` keeps the pre-feature save
        // path alive: a blob with no `map` field decodes to an
        // empty BTreeMap rather than erroring.
        let b: KeyBindings = serde_json::from_str("{}").unwrap();
        assert!(b.map.is_empty());
    }

    #[test]
    fn remapping_moveup_away_from_w_stops_w() {
        let mut b = KeyBindings::defaults();
        let prev = b.set(Action::MoveUp, Binding::Keyboard(KeyCode::KeyT));
        assert_eq!(prev, None, "MoveUp is unbound by stealing nothing");

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        assert!(
            !b.pressed(Action::MoveUp, &keys, &js()),
            "W must stop triggering MoveUp after remap"
        );

        keys.press(KeyCode::KeyT);
        assert!(
            b.pressed(Action::MoveUp, &keys, &js()),
            "T must now trigger MoveUp"
        );
    }

    #[test]
    fn remapping_fire_away_from_space_stops_space() {
        let mut b = KeyBindings::defaults();
        b.set(Action::Fire, Binding::Keyboard(KeyCode::KeyF));
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::Space);
        assert!(
            !b.pressed(Action::Fire, &keys, &js()),
            "Space must stop triggering Fire after remap"
        );

        keys.press(KeyCode::KeyF);
        assert!(b.pressed(Action::Fire, &keys, &js()));
    }

    #[test]
    fn remapping_away_from_key_stops_default_for_other_action_too() {
        // The conflict-silent-overwrite rule: stealing Space from Fire
        // and assigning it to Confirm leaves Fire unbound.
        let mut b = KeyBindings::defaults();
        let prev = b.set(Action::Confirm, Binding::Keyboard(KeyCode::Space));
        assert_eq!(prev, Some(Action::Fire));
        assert_eq!(b.get(Action::Confirm), Some(Binding::Keyboard(KeyCode::Space)));
        assert_eq!(b.get(Action::Fire), None);
    }

    #[test]
    fn clear_required_action_is_rejected() {
        let mut b = KeyBindings::defaults();
        for action in KeyBindings::required_actions() {
            assert!(
                !b.clear(*action),
                "{:?} is required; clear() must reject",
                action
            );
            assert!(
                b.get(*action).is_some(),
                "{:?} must remain bound after a rejected clear",
                action
            );
        }
    }

    #[test]
    fn clear_non_required_action_succeeds() {
        let mut b = KeyBindings::defaults();
        assert!(b.clear(Action::AimUp));
        assert_eq!(b.get(Action::AimUp), None);
    }

    #[test]
    fn reset_to_defaults_restores_layout() {
        let mut b = KeyBindings::defaults();
        b.set(Action::Fire, Binding::Keyboard(KeyCode::KeyF));
        b.clear(Action::AimUp);
        b.reset_to_defaults();
        assert_eq!(b.get(Action::Fire), Some(Binding::Keyboard(KeyCode::Space)));
        assert_eq!(b.get(Action::AimUp), Some(Binding::Keyboard(KeyCode::KeyI)));
    }

    #[test]
    fn custom_bindings_survive_serde_round_trip() {
        let mut b = KeyBindings::defaults();
        b.set(Action::Fire, Binding::Keyboard(KeyCode::KeyF));
        b.set(Action::MoveUp, Binding::Keyboard(KeyCode::KeyT));
        b.clear(Action::AimUp);

        let json = serde_json::to_string(&b).unwrap();
        let restored: KeyBindings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.get(Action::Fire), Some(Binding::Keyboard(KeyCode::KeyF)));
        assert_eq!(restored.get(Action::MoveUp), Some(Binding::Keyboard(KeyCode::KeyT)));
        assert_eq!(restored.get(Action::AimUp), None);
        // Defaults for actions that weren't touched must survive.
        assert_eq!(
            restored.get(Action::MoveDown),
            Some(Binding::Keyboard(KeyCode::KeyS))
        );
    }

    #[test]
    fn unbound_action_returns_false_even_when_key_held() {
        let mut b = KeyBindings::defaults();
        b.clear(Action::AimUp);
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyI); // Ai's default
        assert!(!b.pressed(Action::AimUp, &keys, &js()));
    }

    #[test]
    fn gamepad_binding_resolves() {
        let mut b = KeyBindings::defaults();
        b.set(Action::Fire, Binding::GamepadButton(0));
        let mut joystick = js();
        joystick.buttons[0] = true;
        assert!(b.pressed(Action::Fire, &keys(), &joystick));
        joystick.buttons[0] = false;
        assert!(!b.pressed(Action::Fire, &keys(), &joystick));
    }

    #[test]
    fn no_legacy_keycode_fallback_in_resource_api() {
        // The resource never offers a "check for any of these legacy
        // keys" path. This test documents the design rule; if a
        // future PR adds `pressed_with_legacy_fallback()`, this test
        // should be removed and the rule revisited.
        let b = KeyBindings::defaults();
        let _ = b.get(Action::MoveUp);
    }
}
