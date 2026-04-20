//! Faction Select Screen - Caldari vs Gallente
//!
//! Split-screen faction selection with panel UI.

use super::{ActiveModule, CGModeSelect};
use crate::core::{Faction, GameSession, GameState};
use crate::systems::JoystickState;
use bevy::prelude::*;

// Caldari colors
pub const COLOR_CALDARI_PRIMARY: Color = Color::srgb(0.1, 0.29, 0.48);
pub const COLOR_CALDARI_SECONDARY: Color = Color::srgb(0.29, 0.6, 0.79);
pub const COLOR_CALDARI_ACCENT: Color = Color::srgb(0.48, 0.79, 0.79);

// Gallente colors
pub const COLOR_GALLENTE_PRIMARY: Color = Color::srgb(0.16, 0.35, 0.16);
pub const COLOR_GALLENTE_SECONDARY: Color = Color::srgb(0.35, 0.79, 0.35);
pub const COLOR_GALLENTE_ACCENT: Color = Color::srgb(0.54, 0.92, 0.54);

#[derive(Component)]
pub struct FactionSelectRoot;

#[derive(Component)]
pub struct FactionPanel {
    faction: &'static str,
}

#[derive(Resource, Default)]
pub struct FactionSelectState {
    selected: usize, // 0 = Caldari, 1 = Gallente
    cooldown: f32,
}

#[derive(Component)]
pub struct SelectionArrow {
    faction: &'static str,
}

pub fn spawn_faction_select(mut commands: Commands) {
    info!("Spawning faction select screen!");
    commands.init_resource::<FactionSelectState>();

    // Root container - split screen
    commands
        .spawn((
            FactionSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.02, 0.05)),
        ))
        .with_children(|parent| {
            // Left panel - Caldari
            spawn_faction_panel(parent, "caldari", "CALDARI STATE", true);

            // Center divider with VS
            parent
                .spawn((
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
                ))
                .with_children(|divider| {
                    // Top line
                    divider.spawn((
                        Node {
                            width: Val::Px(2.0),
                            height: Val::Percent(35.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                    ));
                    // VS text
                    divider.spawn((
                        Text::new("VS"),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.3, 0.3)),
                        Node {
                            margin: UiRect::axes(Val::ZERO, Val::Px(20.0)),
                            ..default()
                        },
                    ));
                    // Bottom line
                    divider.spawn((
                        Node {
                            width: Val::Px(2.0),
                            height: Val::Percent(35.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                    ));
                });

            // Right panel - Gallente
            spawn_faction_panel(parent, "gallente", "GALLENTE FEDERATION", false);
        });

    // Title overlay
    commands
        .spawn((
            FactionSelectRoot,
            Node {
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("BATTLE OF CALDARI PRIME"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.6)),
            ));
            parent.spawn((
                Text::new("CHOOSE YOUR SIDE"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });

    // Bottom instruction
    commands
        .spawn((
            FactionSelectRoot,
            Node {
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("D-PAD Select  •  A Confirm  •  B Back"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

fn spawn_faction_panel(
    parent: &mut ChildBuilder,
    faction: &'static str,
    name: &str,
    is_caldari: bool,
) {
    let (primary, secondary, accent) = if is_caldari {
        (
            COLOR_CALDARI_PRIMARY,
            COLOR_CALDARI_SECONDARY,
            COLOR_CALDARI_ACCENT,
        )
    } else {
        (
            COLOR_GALLENTE_PRIMARY,
            COLOR_GALLENTE_SECONDARY,
            COLOR_GALLENTE_ACCENT,
        )
    };

    let doctrine = if is_caldari {
        vec!["MISSILES", "SHIELDS", "ECM"]
    } else {
        vec!["DRONES", "ARMOR", "BLASTERS"]
    };

    let tagline = if is_caldari {
        "\"The State Provides\""
    } else {
        "\"Liberty or Death\""
    };

    let description = if is_caldari {
        "Corporate efficiency meets military precision.\nShield-tanked missile platforms\ndominate the battlefield."
    } else {
        "Freedom through firepower.\nArmor-tanked drone and blaster\nplatforms break all opposition."
    };

    // Outer container with border for selection
    parent
        .spawn((
            FactionPanel { faction },
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                border: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor(Color::NONE), // Will be set by selection logic
        ))
        .with_children(|outer| {
            // Inner panel
            outer
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        row_gap: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(primary.with_alpha(0.25)),
                ))
                .with_children(|panel| {
                    // Faction emblem (hexagon-ish shape)
                    panel
                        .spawn((
                            Node {
                                width: Val::Px(140.0),
                                height: Val::Px(140.0),
                                border: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::bottom(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(primary.with_alpha(0.8)),
                            BorderColor(accent),
                        ))
                        .with_children(|emblem| {
                            // Faction symbol
                            let symbol = if is_caldari { "◆" } else { "✦" };
                            emblem.spawn((
                                Text::new(symbol),
                                TextFont {
                                    font_size: 80.0,
                                    ..default()
                                },
                                TextColor(accent),
                            ));
                        });

                    // Faction name
                    panel.spawn((
                        Text::new(name),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Tagline
                    panel.spawn((
                        Text::new(tagline),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(accent),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    // Doctrine tags
                    panel
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },))
                        .with_children(|tags| {
                            for tag in doctrine {
                                tags.spawn((
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(primary.with_alpha(0.6)),
                                    BorderColor(secondary),
                                ))
                                .with_children(|tag_node| {
                                    tag_node.spawn((
                                        Text::new(tag),
                                        TextFont {
                                            font_size: 13.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                        });

                    // Description
                    panel.spawn((
                        Text::new(description),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        Node {
                            max_width: Val::Px(320.0),
                            ..default()
                        },
                    ));

                    // Selection indicator arrow
                    panel.spawn((
                        SelectionArrow { faction },
                        Text::new("▼ SELECT ▼"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::NONE), // Hidden by default
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
                            ..default()
                        },
                    ));
                });
        });
}

pub fn faction_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    time: Res<Time>,
    mut state: ResMut<FactionSelectState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut mode_state: ResMut<NextState<CGModeSelect>>,
    mut active_module: ResMut<ActiveModule>,
    mut session: ResMut<GameSession>,
    mut panels: Query<(&FactionPanel, &mut BorderColor)>,
    mut arrows: Query<(&SelectionArrow, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    state.cooldown = (state.cooldown - dt).max(0.0);

    // Navigation
    if state.cooldown <= 0.0 {
        let move_left = keyboard.pressed(KeyCode::ArrowLeft)
            || keyboard.pressed(KeyCode::KeyA)
            || joystick.dpad_x < 0;
        let move_right = keyboard.pressed(KeyCode::ArrowRight)
            || keyboard.pressed(KeyCode::KeyD)
            || joystick.dpad_x > 0;

        if move_left && state.selected > 0 {
            state.selected = 0;
            state.cooldown = 0.2;
        } else if move_right && state.selected < 1 {
            state.selected = 1;
            state.cooldown = 0.2;
        }
    }

    // Update panel borders for selection
    for (panel, mut border) in panels.iter_mut() {
        let is_selected = (panel.faction == "caldari" && state.selected == 0)
            || (panel.faction == "gallente" && state.selected == 1);

        let accent = if panel.faction == "caldari" {
            COLOR_CALDARI_ACCENT
        } else {
            COLOR_GALLENTE_ACCENT
        };

        *border = if is_selected {
            BorderColor(accent)
        } else {
            BorderColor(Color::NONE)
        };
    }

    // Update selection arrows
    for (arrow, mut color) in arrows.iter_mut() {
        let is_selected = (arrow.faction == "caldari" && state.selected == 0)
            || (arrow.faction == "gallente" && state.selected == 1);

        let accent = if arrow.faction == "caldari" {
            COLOR_CALDARI_ACCENT
        } else {
            COLOR_GALLENTE_ACCENT
        };

        *color = if is_selected {
            TextColor(accent)
        } else {
            TextColor(Color::NONE)
        };
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        let (player_faction, enemy_faction) = if state.selected == 0 {
            (Faction::Caldari, Faction::Gallente)
        } else {
            (Faction::Gallente, Faction::Caldari)
        };

        // Set both ActiveModule and GameSession for compatibility
        active_module.set_faction(player_faction.short_name(), enemy_faction.short_name());
        *session = GameSession::new(player_faction, enemy_faction);

        info!(
            "Selected {} vs {}",
            player_faction.name(),
            enemy_faction.name()
        );

        // Caldari gets mode select (Campaign vs Nightmare)
        // Gallente goes directly to difficulty (no nightmare mode)
        if player_faction == Faction::Caldari {
            mode_state.set(CGModeSelect::Active);
        } else {
            next_state.set(GameState::DifficultySelect);
        }
    }

    // Back to module select
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        active_module.module_id = None;
        next_state.set(GameState::ModuleSelect);
    }
}

pub fn despawn_faction_select(
    mut commands: Commands,
    query: Query<Entity, With<FactionSelectRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<FactionSelectState>();
}
