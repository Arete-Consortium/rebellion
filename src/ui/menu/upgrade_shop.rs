//! Upgrade Shop

#![allow(dead_code)]

use bevy::prelude::*;
use crate::core::*;
use crate::systems::JoystickState;
use super::common::*;

#[derive(Component)]
pub(crate) struct UpgradeShopRoot;

#[derive(Component)]
pub(crate) struct UpgradeItem {
    pub(crate) upgrade: crate::core::Upgrade,
}

#[derive(Component)]
pub(crate) struct UpgradeDescriptionPanel;

#[derive(Component)]
pub(crate) struct UpgradeDescriptionText;

pub(crate) fn spawn_upgrade_shop(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    save_data: Res<SaveData>,
) {
    use crate::core::Upgrade;

    let upgrades = Upgrade::all();
    selection.index = 0;
    selection.total = upgrades.len();

    commands
        .spawn((
            UpgradeShopRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.95)),
        ))
        .with_children(|parent| {
            // Header row with title and SP
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                })
                .with_children(|header| {
                    // Title
                    header.spawn((
                        Text::new("SKILL UPGRADES"),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.7, 0.3)),
                    ));

                    // SP display
                    header
                        .spawn((
                            Node {
                                padding: UiRect::new(
                                    Val::Px(15.0),
                                    Val::Px(15.0),
                                    Val::Px(8.0),
                                    Val::Px(8.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor(Color::srgb(0.5, 0.3, 0.7)),
                            BackgroundColor(Color::srgba(0.2, 0.1, 0.3, 0.8)),
                        ))
                        .with_children(|sp_box| {
                            sp_box.spawn((
                                Text::new(format!("SP: {}", save_data.skill_points)),
                                TextFont {
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.7, 1.0)),
                            ));
                        });
                });

            // Upgrade list container
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(70.0),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip_y(),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.5)),
                ))
                .with_children(|list| {
                    for (i, &upgrade) in upgrades.iter().enumerate() {
                        let purchased = save_data.has_upgrade(upgrade);
                        let can_buy = save_data.can_purchase_upgrade(upgrade);
                        let locked = if let Some(req) = upgrade.requires() {
                            !save_data.has_upgrade(req)
                        } else {
                            false
                        };

                        let (name_color, status_text, status_color) = if purchased {
                            (
                                Color::srgb(0.5, 0.5, 0.5),
                                "PURCHASED".to_string(),
                                Color::srgb(0.4, 0.6, 0.4),
                            )
                        } else if locked {
                            (
                                Color::srgb(0.4, 0.4, 0.4),
                                format!("Requires: {}", upgrade.requires().unwrap().name()),
                                Color::srgb(0.6, 0.4, 0.4),
                            )
                        } else if can_buy {
                            (
                                Color::WHITE,
                                format!("Cost: {} SP", upgrade.cost()),
                                Color::srgb(0.4, 0.8, 0.4),
                            )
                        } else {
                            (
                                Color::srgb(0.7, 0.7, 0.7),
                                format!("Cost: {} SP", upgrade.cost()),
                                Color::srgb(0.8, 0.5, 0.5),
                            )
                        };

                        list.spawn((
                            UpgradeItem { upgrade },
                            MenuItem { index: i },
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(15.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
                            BorderColor(Color::srgb(0.2, 0.2, 0.3)),
                        ))
                        .with_children(|row| {
                            // Upgrade name
                            row.spawn((
                                Text::new(upgrade.name()),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(name_color),
                            ));

                            // Status/Cost
                            row.spawn((
                                Text::new(status_text),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(status_color),
                            ));
                        });
                    }
                });

            // Description panel (bottom)
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(80.0),
                        margin: UiRect::top(Val::Px(20.0)),
                        padding: UiRect::all(Val::Px(15.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BorderColor(Color::srgb(0.3, 0.3, 0.4)),
                    BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.9)),
                    UpgradeDescriptionPanel,
                ))
                .with_children(|desc_panel| {
                    let first_upgrade = upgrades.first().copied().unwrap_or(Upgrade::ShieldBoost1);
                    desc_panel.spawn((
                        Text::new(first_upgrade.description()),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        UpgradeDescriptionText,
                    ));
                });

            // Controls hint
            parent.spawn((
                Text::new("↑/↓ Navigate  •  Enter/Space Purchase  •  Esc Back"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(15.0)),
                    ..default()
                },
            ));
        });
}

pub(crate) fn update_upgrade_shop_selection(
    selection: Res<MenuSelection>,
    mut item_query: Query<
        (
            &MenuItem,
            &UpgradeItem,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        With<UpgradeItem>,
    >,
    mut desc_query: Query<&mut Text, With<UpgradeDescriptionText>>,
) {
    for (item, upgrade_item, mut border, mut bg) in item_query.iter_mut() {
        if item.index == selection.index {
            border.0 = Color::srgb(0.7, 0.5, 0.9); // Purple highlight
            bg.0 = Color::srgba(0.2, 0.15, 0.25, 0.95);

            // Update description
            if let Ok(mut desc_text) = desc_query.get_single_mut() {
                desc_text.0 = upgrade_item.upgrade.description().to_string();
            }
        } else {
            border.0 = Color::srgb(0.2, 0.2, 0.3);
            bg.0 = Color::srgba(0.1, 0.1, 0.15, 0.8);
        }
    }
}

pub(crate) fn upgrade_shop_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut save_data: ResMut<SaveData>,
    mut next_state: ResMut<NextState<GameState>>,
    item_query: Query<(&MenuItem, &UpgradeItem)>,
    mut commands: Commands,
    sp_query: Query<Entity, With<UpgradeShopRoot>>,
) {
    // Navigation
    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.total > 0 {
        let new_index = (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.index = new_index;
    }

    // Purchase
    if is_confirm(&keyboard, &joystick) {
        for (item, upgrade_item) in item_query.iter() {
            if item.index == selection.index {
                if save_data.purchase_upgrade(upgrade_item.upgrade) {
                    info!("Purchased upgrade: {}", upgrade_item.upgrade.name());
                    // Respawn the shop to update the UI
                    for entity in sp_query.iter() {
                        commands.entity(entity).despawn_recursive();
                    }
                    // Force re-spawn on next frame by temporarily going to a different state
                    // Actually, let's just respawn inline
                    next_state.set(GameState::UpgradeShop);
                }
                break;
            }
        }
    }

    // Back
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
    }
}
