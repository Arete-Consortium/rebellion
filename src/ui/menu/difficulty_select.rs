//! Difficulty Select

#![allow(dead_code)]

use bevy::prelude::*;
use crate::core::*;
use crate::systems::JoystickState;
use super::common::*;

#[derive(Component)]
pub(crate) struct DifficultyMenuRoot;

pub(crate) fn spawn_difficulty_menu(mut commands: Commands, mut selection: ResMut<MenuSelection>) {
    selection.index = 1; // Default to Normal
    selection.total = 4;

    commands
        .spawn((
            DifficultyMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)), // Semi-transparent to show background
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SELECT DIFFICULTY"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Difficulty options
            for (i, diff) in Difficulty::all().iter().enumerate() {
                spawn_difficulty_item(parent, *diff, i);
            }

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            parent.spawn((
                Text::new("↑↓ Navigate • A/ENTER Select • B/ESC Back"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn spawn_difficulty_item(parent: &mut ChildBuilder, diff: Difficulty, index: usize) {
    parent
        .spawn((
            DifficultyMenuRoot, // Marker for update_menu_selection query
            MenuItem { index },
            Node {
                width: Val::Px(450.0),
                height: Val::Px(85.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|btn| {
            // Difficulty name
            btn.spawn((
                Text::new(diff.name()),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(diff.color()),
            ));
            // Tagline
            btn.spawn((
                Text::new(format!("\"{}\"", diff.tagline())),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            // Description
            btn.spawn((
                Text::new(diff.description()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

pub(crate) fn difficulty_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut difficulty: ResMut<Difficulty>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    selection.cooldown -= time.delta_secs();

    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    if is_confirm(&keyboard, &joystick) {
        *difficulty = Difficulty::all()[selection.index.min(3)];
        info!(
            "Selected difficulty: {} - {}",
            difficulty.name(),
            difficulty.tagline()
        );
        next_state.set(GameState::ShipSelect);
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
    }
}
