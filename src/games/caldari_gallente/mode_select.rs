//! Mode Select Screen - Campaign vs The Last Stand
//!
//! Caldari-only mode selection after faction select.

use super::faction_select::{COLOR_CALDARI_ACCENT, COLOR_CALDARI_PRIMARY};
use super::last_stand::LastStandState;
use super::CGModeSelect;
use crate::core::GameState;
use crate::systems::JoystickState;
use bevy::prelude::*;

#[derive(Component)]
pub struct ModeSelectRoot;

#[derive(Component)]
pub struct ModeOption {
    is_nightmare: bool,
}

#[derive(Resource, Default)]
pub struct ModeSelectState {
    selected: usize, // 0 = Campaign, 1 = Nightmare
    cooldown: f32,
}

pub fn spawn_mode_select(mut commands: Commands) {
    info!("Spawning mode select screen (Campaign vs Nightmare)");
    commands.init_resource::<ModeSelectState>();

    // Root container
    commands
        .spawn((
            ModeSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.02, 0.05)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("CALDARI STATE"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(COLOR_CALDARI_ACCENT),
            ));
            parent.spawn((
                Text::new("SELECT MODE"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Campaign option
            spawn_mode_option(parent, false, "CAMPAIGN", "5 Mission Story Arc", true);

            // The Last Stand option
            spawn_mode_option(
                parent,
                true,
                "THE LAST STAND",
                "CNS Kairiola • Fixed Platform Defense",
                false,
            );

            // Instructions
            parent.spawn((
                Text::new("[↑/↓] Select   [SPACE/ENTER] Confirm   [ESC] Back"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_mode_option(
    parent: &mut ChildBuilder,
    is_nightmare: bool,
    title: &str,
    subtitle: &str,
    selected: bool,
) {
    let border_color = if selected {
        if is_nightmare {
            Color::srgb(0.9, 0.2, 0.2) // Red for nightmare
        } else {
            COLOR_CALDARI_ACCENT
        }
    } else {
        Color::srgb(0.2, 0.2, 0.3)
    };

    let bg_color = if is_nightmare {
        Color::srgb(0.15, 0.05, 0.05) // Dark red tint
    } else {
        COLOR_CALDARI_PRIMARY.with_alpha(0.3)
    };

    parent
        .spawn((
            ModeOption { is_nightmare },
            Node {
                width: Val::Px(400.0),
                height: Val::Px(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
        ))
        .with_children(|card| {
            // Title
            let title_color = if is_nightmare {
                Color::srgb(1.0, 0.4, 0.4)
            } else {
                Color::WHITE
            };
            card.spawn((
                Text::new(title),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(title_color),
            ));
            // Subtitle
            card.spawn((
                Text::new(subtitle),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

pub fn mode_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    time: Res<Time>,
    mut state: ResMut<ModeSelectState>,
    mut last_stand: ResMut<LastStandState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut mode_state: ResMut<NextState<CGModeSelect>>,
    mut options: Query<(&ModeOption, &mut BorderColor)>,
) {
    let dt = time.delta_secs();
    state.cooldown = (state.cooldown - dt).max(0.0);

    // Navigation
    if state.cooldown <= 0.0 {
        let move_up = keyboard.pressed(KeyCode::ArrowUp)
            || keyboard.pressed(KeyCode::KeyW)
            || joystick.dpad_y > 0;
        let move_down = keyboard.pressed(KeyCode::ArrowDown)
            || keyboard.pressed(KeyCode::KeyS)
            || joystick.dpad_y < 0;

        if move_up && state.selected > 0 {
            state.selected = 0;
            state.cooldown = 0.2;
        } else if move_down && state.selected < 1 {
            state.selected = 1;
            state.cooldown = 0.2;
        }
    }

    // Update option borders
    for (option, mut border) in options.iter_mut() {
        let is_selected = (!option.is_nightmare && state.selected == 0)
            || (option.is_nightmare && state.selected == 1);

        let color = if is_selected {
            if option.is_nightmare {
                Color::srgb(1.0, 0.6, 0.2) // Orange for Last Stand (sacrifice)
            } else {
                COLOR_CALDARI_ACCENT
            }
        } else {
            Color::srgb(0.2, 0.2, 0.3)
        };
        *border = BorderColor(color);
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        if state.selected == 1 {
            // Last Stand mode selected - skip difficulty and ship select, go straight to Playing
            last_stand.start();
            info!("Starting THE LAST STAND - CNS Kairiola defense!");
            mode_state.set(CGModeSelect::Inactive);
            next_state.set(GameState::Playing);
        } else {
            info!("Starting Campaign mode");
            mode_state.set(CGModeSelect::Inactive);
            next_state.set(GameState::DifficultySelect);
        }
    }

    // Back to faction select
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        mode_state.set(CGModeSelect::Inactive);
        next_state.set(GameState::FactionSelect);
    }
}

pub fn despawn_mode_select(mut commands: Commands, query: Query<Entity, With<ModeSelectRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<ModeSelectState>();
}
