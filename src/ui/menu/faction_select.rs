//! Faction Select (Elder Fleet - Minmatar vs Amarr)

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::systems::JoystickState;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct FactionSelectRoot;

pub(crate) fn spawn_faction_select(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    mut session: ResMut<GameSession>,
) {
    selection.index = 0;
    selection.total = 2; // Elder Fleet: Minmatar vs Amarr only

    // Default to Minmatar vs Amarr
    *session = GameSession::new(Faction::Minmatar, Faction::Amarr);

    commands
        .spawn((
            FactionSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            // Subtitle
            parent.spawn((
                Text::new("THE ELDER FLEET"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.5, 0.3)),
            ));

            // Title
            parent.spawn((
                Text::new("CHOOSE YOUR FACTION"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Faction row - Minmatar vs Amarr only (horizontal layout)
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(40.0),
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|row| {
                    spawn_faction_card(row, Faction::Minmatar, 0);

                    // VS divider
                    row.spawn((
                        Text::new("VS"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));

                    spawn_faction_card(row, Faction::Amarr, 1);
                });

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Instructions
            parent.spawn((
                Text::new("← → Navigate • A/ENTER Select • B/ESC Back"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn spawn_faction_card(parent: &mut ChildBuilder, faction: Faction, index: usize) {
    let primary = faction.primary_color();
    let secondary = faction.secondary_color();
    let rival = faction.rival();
    let ship_count = faction.player_ships().len();

    // Get first line of lore for preview
    let lore_preview = faction.story_intro().lines().next().unwrap_or("");
    let lore_short = if lore_preview.len() > 60 {
        format!("{}...", &lore_preview[..57])
    } else {
        lore_preview.to_string()
    };

    parent
        .spawn((
            FactionSelectRoot,
            MenuItem { index },
            Node {
                width: Val::Px(320.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(6.0),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.95)),
            BorderColor(primary.with_alpha(0.4)),
        ))
        .with_children(|card| {
            // Header row: Faction name + emblem placeholder
            card.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    Text::new(faction.short_name()),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(primary),
                ));

                // Faction emblem placeholder (colored square)
                header.spawn((
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(secondary.with_alpha(0.6)),
                    BorderColor(primary.with_alpha(0.8)),
                ));
            });

            // Full name
            card.spawn((
                Text::new(faction.name()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Divider
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(primary.with_alpha(0.3)),
            ));

            // Tagline
            card.spawn((
                Text::new(format!("\"{}\"", faction.tagline())),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Lore preview
            card.spawn((
                Text::new(lore_short),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.45, 0.45, 0.5)),
            ));

            // Spacer
            card.spawn(Node {
                height: Val::Px(6.0),
                ..default()
            });

            // Combat stats row
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|stats| {
                // Weapon doctrine
                stats.spawn((
                    Text::new(faction.weapon_type().name()),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(faction.weapon_type().bullet_color()),
                ));

                // Tank doctrine
                let tank_text = match faction.tank_type() {
                    TankDoctrine::Shield => "Shield",
                    TankDoctrine::Armor => "Armor",
                    TankDoctrine::Speed => "Speed",
                };
                stats.spawn((
                    Text::new(tank_text),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.7, 0.9)),
                ));

                // Ship count
                stats.spawn((
                    Text::new(format!("{} Ships", ship_count)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });

            // Divider
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(rival.primary_color().with_alpha(0.3)),
            ));

            // Enemy faction row
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|enemy_row| {
                enemy_row.spawn((
                    Text::new("ENEMY:"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));

                enemy_row.spawn((
                    Text::new(rival.short_name()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(rival.primary_color()),
                ));

                enemy_row.spawn((
                    Text::new(format!("({})", rival.weapon_type().name())),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(rival.primary_color().with_alpha(0.6)),
                ));
            });
        });
}

pub(crate) fn faction_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut session: ResMut<GameSession>,
    endless: Res<crate::core::EndlessMode>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cards: Query<(&MenuItem, &mut BackgroundColor, &mut BorderColor), With<FactionSelectRoot>>,
) {
    selection.cooldown -= time.delta_secs();

    // Elder Fleet: Simple left/right navigation for Minmatar vs Amarr
    // Layout: 0=Minmatar(left), 1=Amarr(right)
    if selection.cooldown <= 0.0 {
        let left = keyboard.pressed(KeyCode::ArrowLeft)
            || keyboard.pressed(KeyCode::KeyA)
            || joystick.dpad_x < 0;
        let right = keyboard.pressed(KeyCode::ArrowRight)
            || keyboard.pressed(KeyCode::KeyD)
            || joystick.dpad_x > 0;

        let mut new_index = selection.index;

        if left && selection.index > 0 {
            new_index = selection.index - 1;
        } else if right && selection.index < selection.total - 1 {
            new_index = selection.index + 1;
        }

        if new_index != selection.index {
            selection.index = new_index;
            selection.cooldown = MENU_NAV_COOLDOWN;
        }
    }

    // Update card highlights - Elder Fleet: Minmatar vs Amarr
    let factions = [Faction::Minmatar, Faction::Amarr];

    for (item, mut bg, mut border) in cards.iter_mut() {
        if item.index >= factions.len() {
            continue;
        }
        let faction = factions[item.index];
        let is_selected = item.index == selection.index;

        if is_selected {
            *bg = BackgroundColor(faction.primary_color().with_alpha(0.4));
            *border = BorderColor(faction.primary_color());
        } else {
            *bg = BackgroundColor(faction.secondary_color().with_alpha(0.6));
            *border = BorderColor(faction.primary_color().with_alpha(0.3));
        }
    }

    // Confirm selection
    if is_confirm(&keyboard, &joystick) {
        let player_faction = factions[selection.index];
        let enemy_faction = player_faction.rival();

        *session = GameSession::new(player_faction, enemy_faction);
        info!(
            "Selected {} vs {}",
            player_faction.name(),
            enemy_faction.name()
        );

        // Endless mode skips stage select, goes to difficulty
        // Campaign mode goes to stage select
        if endless.active {
            next_state.set(GameState::DifficultySelect);
        } else {
            next_state.set(GameState::StageSelect);
        }
    }

    // Back to module select
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::ModuleSelect);
    }
}
