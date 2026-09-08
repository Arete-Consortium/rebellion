//! Faction Select (Elder Fleet - Minmatar vs Amarr)

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::games::ActiveModule;
use crate::systems::JoystickState;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct FactionSelectRoot;

fn chapter_factions(module: &ActiveModule) -> [Faction; 2] {
    if module.is_caldari_gallente() {
        [Faction::Caldari, Faction::Gallente]
    } else {
        [Faction::Minmatar, Faction::Amarr]
    }
}

pub(crate) fn spawn_faction_select(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    mut session: ResMut<GameSession>,
    active_module: Res<ActiveModule>,
    bindings: Res<KeyBindings>,
    icons: Res<crate::assets::FactionIconCache>,
) {
    selection.index = 0;
    selection.total = 2;
    let factions = chapter_factions(&active_module);

    // Default to Minmatar vs Amarr
    *session = GameSession::new(factions[0], factions[1]);

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
                Text::new(if active_module.is_caldari_gallente() {
                    "CALDARI PRIME"
                } else {
                    "THE ELDER FLEET"
                }),
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
                    spawn_faction_card(row, factions[0], 0, &icons);

                    // VS divider
                    row.spawn((
                        Text::new("VS"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));

                    spawn_faction_card(row, factions[1], 1, &icons);
                });

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Instructions
            parent.spawn((
                Text::new(menu_hint(&bindings, "Select", "Back")),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn spawn_faction_card(
    parent: &mut ChildBuilder,
    faction: Faction,
    index: usize,
    icons: &crate::assets::FactionIconCache,
) {
    let primary = faction.primary_color();
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

                if let Some(image) = icons.get(faction) {
                    header.spawn((
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                        ImageNode { image, ..default() },
                    ));
                }
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
    bindings: Res<KeyBindings>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut session: ResMut<GameSession>,
    mut active_module: ResMut<ActiveModule>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cards: Query<(&MenuItem, &mut BackgroundColor, &mut BorderColor), With<FactionSelectRoot>>,
) {
    selection.cooldown -= time.delta_secs();

    // Elder Fleet: Simple left/right navigation for Minmatar vs Amarr
    // Layout: 0=Minmatar(left), 1=Amarr(right)
    if selection.cooldown <= 0.0 {
        let direction = get_horizontal_input(&keyboard, &joystick, &bindings);
        let left = direction < 0;
        let right = direction > 0;

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
    let factions = chapter_factions(&active_module);

    for (item, mut bg, mut border) in cards.iter_mut() {
        if item.index >= factions.len() {
            continue;
        }
        let faction = factions[item.index];
        let is_selected = item.index == selection.index;

        if is_selected {
            *bg = BackgroundColor(Color::srgba(0.08, 0.09, 0.12, 0.97));
            *border = BorderColor(faction.primary_color());
        } else {
            *bg = BackgroundColor(Color::srgba(0.035, 0.04, 0.065, 0.97));
            *border = BorderColor(faction.primary_color().with_alpha(0.3));
        }
    }

    // Confirm selection
    if is_confirm(&keyboard, &joystick, &bindings) {
        let player_faction = factions[selection.index];
        let enemy_faction = player_faction.rival();

        *session = GameSession::new(player_faction, enemy_faction);
        info!(
            "Selected {} vs {}",
            player_faction.name(),
            enemy_faction.name()
        );

        active_module.set_faction(player_faction.short_name(), enemy_faction.short_name());
        next_state.set(GameState::DifficultySelect);
    }

    // Back to module select
    if is_cancel(&keyboard, &joystick, &bindings) {
        next_state.set(GameState::ModuleSelect);
    }
}
