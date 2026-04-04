//! Module Select

#![allow(dead_code)]

use bevy::prelude::*;
use crate::core::*;
use crate::games::ActiveModule;
use crate::systems::JoystickState;
use crate::ui::TransitionEvent;
use super::common::*;

#[derive(Component)]
pub(crate) struct ModuleSelectRoot;

/// Run condition: is the active module Elder Fleet (default)?
pub(crate) fn is_elder_fleet(active_module: Res<ActiveModule>) -> bool {
    active_module.is_elder_fleet()
}

pub(crate) fn spawn_module_select(mut commands: Commands, mut selection: ResMut<MenuSelection>) {
    selection.index = 0;
    selection.total = 4; // Elder Fleet, Caldari vs Gallente, Abyssal Depths, Endless

    commands
        .spawn((
            ModuleSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SELECT CAMPAIGN"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Module cards container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(30.0),
                    ..default()
                })
                .with_children(|row| {
                    // Elder Fleet card
                    spawn_module_card(
                        row,
                        0,
                        "THE ELDER FLEET",
                        "Minmatar Liberation",
                        "Play as any faction against their rival.\n13 missions across 3 acts.",
                        Color::srgb(0.8, 0.5, 0.2), // Minmatar orange
                        "⚔",
                    );

                    // Caldari vs Gallente card
                    spawn_module_card(
                        row,
                        1,
                        "CALDARI PRIME",
                        "Faction Warfare",
                        "Caldari vs Gallente conflict.\n5 missions of brutal combat.",
                        Color::srgb(0.2, 0.4, 0.7), // Caldari blue
                        "◆",
                    );

                    // Abyssal Depths card
                    spawn_module_card(
                        row,
                        2,
                        "ABYSSAL DEPTHS",
                        "Triglavian Extraction",
                        "3 rooms. Limited time.\nExtract or die in the Abyss.",
                        Color::srgb(0.6, 0.2, 0.6), // Triglavian purple
                        "◈",
                    );

                    // Endless Mode card
                    spawn_module_card(
                        row,
                        3,
                        "ENDLESS",
                        "Survival Mode",
                        "Infinite waves of enemies.\nSurvive as long as you can!",
                        Color::srgb(0.7, 0.2, 0.2), // Red for danger
                        "∞",
                    );
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

fn spawn_module_card(
    parent: &mut ChildBuilder,
    index: usize,
    title: &str,
    subtitle: &str,
    description: &str,
    color: Color,
    symbol: &str,
) {
    parent
        .spawn((
            MenuItem { index },
            Node {
                width: Val::Px(280.0),
                height: Val::Px(320.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(3.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(color.with_alpha(0.2)),
            BorderColor(color.with_alpha(0.5)),
        ))
        .with_children(|card| {
            // Symbol
            card.spawn((
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(color.with_alpha(0.4)),
                BorderColor(color),
            ))
            .with_children(|emblem| {
                emblem.spawn((
                    Text::new(symbol),
                    TextFont {
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(color),
                ));
            });

            // Title
            card.spawn((
                Text::new(title),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Subtitle
            card.spawn((
                Text::new(subtitle),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
            ));

            // Description
            card.spawn((
                Text::new(description),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    max_width: Val::Px(240.0),
                    ..default()
                },
            ));
        });
}

pub(crate) fn module_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut active_module: ResMut<ActiveModule>,
    mut endless: ResMut<crate::core::EndlessMode>,
    mut abyssal: ResMut<crate::games::abyssal_depths::AbyssalState>,
    time: Res<Time>,
    mut transitions: EventWriter<TransitionEvent>,
    mut cards: Query<(&MenuItem, &mut BackgroundColor, &mut BorderColor)>,
) {
    selection.cooldown -= time.delta_secs();

    // Navigation
    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    // Update card highlights
    let colors = [
        Color::srgb(0.8, 0.5, 0.2), // Elder Fleet orange
        Color::srgb(0.2, 0.4, 0.7), // Caldari blue
        Color::srgb(0.6, 0.2, 0.6), // Abyssal purple
        Color::srgb(0.7, 0.2, 0.2), // Endless red
    ];

    for (item, mut bg, mut border) in cards.iter_mut() {
        let color = colors.get(item.index).copied().unwrap_or(colors[0]);
        let is_selected = item.index == selection.index;

        if is_selected {
            *bg = BackgroundColor(color.with_alpha(0.4));
            *border = BorderColor(color);
        } else {
            *bg = BackgroundColor(color.with_alpha(0.2));
            *border = BorderColor(color.with_alpha(0.5));
        }
    }

    // Confirm selection
    if is_confirm(&keyboard, &joystick) {
        match selection.index {
            0 => {
                // Elder Fleet
                active_module.set_module("elder_fleet");
                endless.active = false;
                abyssal.active = false;
                info!("Selected Elder Fleet campaign");
                transitions.send(TransitionEvent::to(GameState::FactionSelect));
            }
            1 => {
                // Caldari vs Gallente
                active_module.set_module("caldari_gallente");
                endless.active = false;
                abyssal.active = false;
                info!("Selected Caldari vs Gallente campaign");
                transitions.send(TransitionEvent::to(GameState::FactionSelect));
            }
            2 => {
                // Abyssal Depths
                active_module.set_module("abyssal_depths");
                endless.active = false;
                abyssal.active = true; // Set BEFORE entering Playing state
                info!("Selected ABYSSAL DEPTHS!");
                // Skip faction select, go straight to ship select
                transitions.send(TransitionEvent::to(GameState::ShipSelect));
            }
            3 => {
                // Endless Mode
                active_module.set_module("elder_fleet"); // Use Elder Fleet enemies
                endless.active = true;
                abyssal.active = false;
                info!("Selected ENDLESS MODE!");
                transitions.send(TransitionEvent::to(GameState::FactionSelect));
            }
            _ => {}
        }
    }

    // Back to main menu
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        transitions.send(TransitionEvent::to(GameState::MainMenu));
    }
}
