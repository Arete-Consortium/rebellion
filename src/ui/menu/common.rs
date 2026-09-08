//! Shared menu infrastructure used by all menu screens.

#![allow(dead_code)]

use crate::core::*;
use crate::systems::JoystickState;
use bevy::prelude::*;

// ============================================================================
// Menu Selection System (keyboard/joystick navigation)
// ============================================================================

#[derive(Resource, Default)]
pub struct MenuSelection {
    pub index: usize,
    pub total: usize,
    pub cooldown: f32,
}

pub(crate) const MENU_NAV_COOLDOWN: f32 = 0.15;

/// Menu item that can be selected
#[derive(Component)]
pub(crate) struct MenuItem {
    pub(crate) index: usize,
}

/// Adds a `Button` component to every freshly-spawned `MenuItem` so it
/// receives `Interaction` updates for mouse + touch hit-testing. One-shot
/// fix-up — runs every frame but only writes to entities that don't yet
/// have a `Button`. Cheap.
pub(crate) fn ensure_menu_items_interactive(
    mut commands: Commands,
    items: Query<Entity, (With<MenuItem>, Without<Button>)>,
) {
    for entity in &items {
        commands.entity(entity).insert(Button);
    }
}

/// Tap or click on a `MenuItem` → set `MenuSelection.index` to that item
/// and synthesize a confirm-button press for one frame so the existing
/// menu input handlers fire with the freshly-clicked index. Mobile and
/// desktop both flow through this — desktop gets click-to-go for free.
///
/// Debounce: after a `GameState` change, a touch that was already down
/// during the previous screen will fire `Interaction::Pressed` on the
/// just-spawned default item of the next screen, auto-confirming it.
/// Suppress all tap-confirms for ~250ms after a state change.
pub(crate) fn handle_menu_item_taps(
    state: Res<State<GameState>>,
    mut selection: ResMut<MenuSelection>,
    mut joystick: ResMut<JoystickState>,
    items: Query<(&MenuItem, &Interaction), Changed<Interaction>>,
    mut last_state: Local<Option<GameState>>,
    time: Res<Time>,
    mut debounce: Local<f32>,
    mut pointer_confirmed: Local<bool>,
) {
    // A synthetic press lasts one frame even when no physical pad is present.
    if *pointer_confirmed {
        if !joystick.connected {
            joystick.buttons[0] = false;
        }
        *pointer_confirmed = false;
    }
    let current = *state.get();
    let state_changed = match *last_state {
        Some(prev) => prev != current,
        None => true,
    };
    *last_state = Some(current);

    if state_changed {
        // Force a quarter-second debounce so a finger held during the
        // previous screen can't immediately auto-fire the new screen's
        // default item.
        *debounce = 0.25;
        return;
    }
    *debounce = (*debounce - time.delta_secs()).max(0.0);
    if *debounce > 0.0 {
        return;
    }

    for (item, interaction) in &items {
        if matches!(interaction, Interaction::Pressed) {
            selection.index = item.index;
            joystick.buttons[0] = true;
            joystick.prev_buttons[0] = false;
            *pointer_confirmed = true;
        }
    }
}

/// Marker for selected menu item highlight
#[derive(Component)]
pub(crate) struct SelectionIndicator;

// ============================================================================
// Helper Functions
// ============================================================================

pub(crate) fn spawn_menu_item(parent: &mut ChildBuilder, text: &str, index: usize) {
    parent
        .spawn((
            super::main_menu::MainMenuRoot, // Marker for update_menu_selection query
            MenuItem { index },
            Node {
                width: Val::Px(360.0),
                height: Val::Px(55.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size: 19.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub(crate) fn update_menu_selection<T: Component>(
    selection: Res<MenuSelection>,
    mut query: Query<(&MenuItem, &mut BorderColor, &mut BackgroundColor), With<T>>,
) {
    for (item, mut border, mut bg) in query.iter_mut() {
        if item.index == selection.index {
            border.0 = COLOR_MINMATAR;
            bg.0 = Color::srgba(0.25, 0.15, 0.1, 0.95);
        } else {
            border.0 = Color::srgb(0.3, 0.3, 0.3);
            bg.0 = Color::srgba(0.1, 0.1, 0.1, 0.9);
        }
    }
}

/// List navigation accepts either menu axis. Grids/sliders use the axis
/// helpers separately so one key cannot move both row and column.
pub(crate) fn get_nav_input(
    keyboard: &ButtonInput<KeyCode>,
    joystick: &JoystickState,
    bindings: &KeyBindings,
) -> i32 {
    let vertical = get_vertical_input(keyboard, joystick, bindings);
    if vertical != 0 {
        vertical
    } else {
        get_horizontal_input(keyboard, joystick, bindings)
    }
}

pub(crate) fn get_vertical_input(
    keyboard: &ButtonInput<KeyCode>,
    joystick: &JoystickState,
    bindings: &KeyBindings,
) -> i32 {
    let up = bindings.just_pressed(Action::MenuUp, keyboard, joystick)
        || joystick.dpad_just_up()
        || joystick.left_y > 0.5;
    let down = bindings.just_pressed(Action::MenuDown, keyboard, joystick)
        || joystick.dpad_just_down()
        || joystick.left_y < -0.5;
    i32::from(down) - i32::from(up)
}

/// Held horizontal input supports controlled slider/key repeat.
pub(crate) fn get_horizontal_input(
    keyboard: &ButtonInput<KeyCode>,
    joystick: &JoystickState,
    bindings: &KeyBindings,
) -> i32 {
    let left = bindings.pressed(Action::MenuLeft, keyboard, joystick)
        || bindings.just_pressed(Action::MenuLeft, keyboard, joystick)
        || joystick.dpad_x < 0
        || joystick.left_x < -0.5;
    let right = bindings.pressed(Action::MenuRight, keyboard, joystick)
        || bindings.just_pressed(Action::MenuRight, keyboard, joystick)
        || joystick.dpad_x > 0
        || joystick.left_x > 0.5;
    i32::from(right) - i32::from(left)
}

pub(crate) fn is_confirm(
    keyboard: &ButtonInput<KeyCode>,
    joystick: &JoystickState,
    bindings: &KeyBindings,
) -> bool {
    bindings.just_pressed(Action::Confirm, keyboard, joystick) || joystick.confirm()
}

pub(crate) fn is_cancel(
    keyboard: &ButtonInput<KeyCode>,
    joystick: &JoystickState,
    bindings: &KeyBindings,
) -> bool {
    bindings.just_pressed(Action::Cancel, keyboard, joystick) || joystick.back()
}

pub(crate) fn menu_hint(bindings: &KeyBindings, confirm: &str, cancel: &str) -> String {
    let label = |action| {
        bindings
            .get(action)
            .map(|binding| match binding {
                Binding::Keyboard(KeyCode::ArrowUp) => "Up".into(),
                Binding::Keyboard(KeyCode::ArrowDown) => "Down".into(),
                Binding::Keyboard(KeyCode::ArrowLeft) => "Left".into(),
                Binding::Keyboard(KeyCode::ArrowRight) => "Right".into(),
                _ => binding.label(),
            })
            .unwrap_or_else(|| "Unbound".into())
    };
    format!(
        "{}/{} Navigate  |  {} / Pad A {}  |  {} / Pad B {}",
        label(Action::MenuUp),
        label(Action::MenuDown),
        label(Action::Confirm),
        confirm,
        label(Action::Cancel),
        cancel
    )
}

/// Safely despawn menu root entities, skipping any that no longer exist.
pub(crate) fn despawn_menu<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        if let Some(ec) = commands.get_entity(entity) {
            ec.despawn_recursive();
        }
    }
}

pub(crate) fn format_score(score: u64) -> String {
    let s = score.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remapped_menu_keys_replace_default_keys() {
        let mut bindings = KeyBindings::default();
        let pad = JoystickState::default();
        bindings.set(Action::Confirm, Binding::Keyboard(KeyCode::KeyR));
        bindings.set(Action::Cancel, Binding::Keyboard(KeyCode::KeyT));
        bindings.set(Action::MenuDown, Binding::Keyboard(KeyCode::KeyF));
        let mut keys = ButtonInput::default();
        for key in [
            KeyCode::Enter,
            KeyCode::Space,
            KeyCode::Escape,
            KeyCode::ArrowDown,
        ] {
            keys.press(key);
        }
        assert!(!is_confirm(&keys, &pad, &bindings));
        assert!(!is_cancel(&keys, &pad, &bindings));
        assert_eq!(get_nav_input(&keys, &pad, &bindings), 0);
        keys.reset_all();
        for key in [KeyCode::KeyR, KeyCode::KeyT, KeyCode::KeyF] {
            keys.press(key);
        }
        assert!(is_confirm(&keys, &pad, &bindings));
        assert!(is_cancel(&keys, &pad, &bindings));
        assert_eq!(get_vertical_input(&keys, &pad, &bindings), 1);
        assert_eq!(get_horizontal_input(&keys, &pad, &bindings), 0);
    }
}

#[cfg(test)]
mod pointer_tests {
    use super::*;

    #[test]
    fn repeated_mouse_activation_works_without_a_connected_gamepad() {
        let mut world = World::new();
        world.insert_resource(State::new(GameState::MainMenu));
        world.init_resource::<MenuSelection>();
        world.init_resource::<JoystickState>();
        world.init_resource::<Time>();
        world
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_millis(300));
        let button = world.spawn((MenuItem { index: 2 }, Interaction::None)).id();
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                crate::systems::joystick::poll_gamepad,
                handle_menu_item_taps,
            )
                .chain(),
        );
        schedule.run(&mut world); // state-entry debounce
        schedule.run(&mut world); // debounce expires
        for _ in 0..2 {
            *world.get_mut::<Interaction>(button).unwrap() = Interaction::Pressed;
            schedule.run(&mut world);
            assert!(world.resource::<JoystickState>().confirm());
            assert_eq!(world.resource::<MenuSelection>().index, 2);
            *world.get_mut::<Interaction>(button).unwrap() = Interaction::None;
            schedule.run(&mut world);
            assert!(
                !world.resource::<JoystickState>().buttons[0],
                "pointer release must clear the synthetic button"
            );
        }
    }
    #[test]
    fn horizontal_navigation_preserves_a_tap_released_in_the_same_frame() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::ArrowRight);
        keys.release(KeyCode::ArrowRight);
        assert_eq!(
            get_horizontal_input(&keys, &JoystickState::default(), &KeyBindings::default()),
            1
        );
    }
}
