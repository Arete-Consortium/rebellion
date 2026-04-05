//! Shared menu infrastructure used by all menu screens.

#![allow(dead_code)]

use crate::core::*;
use crate::systems::JoystickState;
use bevy::prelude::*;

// ============================================================================
// Menu Selection System (keyboard/joystick navigation)
// ============================================================================

#[derive(Resource, Default)]
pub(crate) struct MenuSelection {
    pub(crate) index: usize,
    pub(crate) total: usize,
    pub(crate) cooldown: f32,
}

pub(crate) const MENU_NAV_COOLDOWN: f32 = 0.15;

/// Menu item that can be selected
#[derive(Component)]
pub(crate) struct MenuItem {
    pub(crate) index: usize,
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
                width: Val::Px(280.0),
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
                    font_size: 22.0,
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

pub(crate) fn get_nav_input(keyboard: &ButtonInput<KeyCode>, joystick: &JoystickState) -> i32 {
    let mut nav = 0;

    // Keyboard (edge triggered) - Up/Down and Left/Right both work
    if keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
        || keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
    {
        nav = -1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
        || keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
    {
        nav = 1;
    }

    // Joystick dpad (edge triggered)
    if joystick.dpad_just_up() || joystick.dpad_just_left() {
        nav = -1;
    }
    if joystick.dpad_just_down() || joystick.dpad_just_right() {
        nav = 1;
    }

    // Analog stick (held state - menu cooldown prevents rapid repeat)
    // Bevy convention: stick up = +Y, but menu nav up = -1 (lower index)
    if joystick.left_y > 0.5 || joystick.left_x < -0.5 {
        nav = -1; // Up / Left = previous item
    }
    if joystick.left_y < -0.5 || joystick.left_x > 0.5 {
        nav = 1; // Down / Right = next item
    }

    nav
}

/// Get horizontal input (-1 left, 0 none, 1 right)
pub(crate) fn get_horizontal_input(
    keyboard: &ButtonInput<KeyCode>,
    joystick: &JoystickState,
) -> i32 {
    let mut h = 0;

    // Keyboard
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        h -= 1;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        h += 1;
    }

    // Joystick d-pad
    if joystick.dpad_x < 0 {
        h -= 1;
    } else if joystick.dpad_x > 0 {
        h += 1;
    }

    // Joystick left stick
    if joystick.left_x < -0.5 {
        h -= 1;
    } else if joystick.left_x > 0.5 {
        h += 1;
    }

    h.clamp(-1, 1)
}

pub(crate) fn is_confirm(keyboard: &ButtonInput<KeyCode>, joystick: &JoystickState) -> bool {
    keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
}

pub(crate) fn despawn_menu<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
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
