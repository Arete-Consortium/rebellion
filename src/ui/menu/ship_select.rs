//! Ship Select

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::systems::JoystickState;
use crate::ui::TransitionEvent;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct ShipMenuRoot;

/// Marker for the selected ship detail panel
#[derive(Component)]
pub(crate) struct ShipDetailPanel;

/// Marker for ship detail text elements
#[derive(Component)]
pub(crate) struct ShipDetailName;
#[derive(Component)]
pub(crate) struct ShipDetailClass;
#[derive(Component)]
pub(crate) struct ShipDetailRole;
#[derive(Component)]
pub(crate) struct ShipDetailSpecial;
#[derive(Component)]
pub(crate) struct ShipDetailWeapon;
/// Preview image node that shows the selected hull's sprite.
#[derive(Component)]
pub(crate) struct ShipDetailSprite;

/// Stat bar markers
#[derive(Component)]
pub(crate) struct StatBarFill(pub(crate) StatType);

#[derive(Clone, Copy)]
pub(crate) enum StatType {
    Speed,
    Damage,
    Health,
    FireRate,
}

pub(crate) fn spawn_ship_menu(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    difficulty: Res<Difficulty>,
    session: Res<GameSession>,
    save_data: Res<crate::core::SaveData>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
) {
    let ships = session.player_ships();
    let faction = session.player_faction;
    let enemy = session.enemy_faction;
    let faction_color = faction.primary_color();

    selection.index = 0;
    selection.total = ships.len();

    // Calculate stat ranges for normalization
    let max_speed = ships.iter().map(|s| s.speed).fold(0.0_f32, f32::max);
    let max_damage = ships.iter().map(|s| s.damage).fold(0.0_f32, f32::max);
    let max_health = ships.iter().map(|s| s.health).fold(0.0_f32, f32::max);
    let max_fire_rate = ships.iter().map(|s| s.fire_rate).fold(0.0_f32, f32::max);

    commands
        .spawn((
            ShipMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            // Title with faction name
            parent.spawn((
                Text::new(format!("{} FLEET - SELECT SHIP", faction.short_name())),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            // Subtitle with weapon doctrine and difficulty
            parent.spawn((
                Text::new(format!(
                    "{} Doctrine • {} Mode",
                    faction.weapon_type().name(),
                    difficulty.name()
                )),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.65, 0.65)),
            ));

            // Main content: Detail panel (left) + Ship list (right)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    max_width: Val::Px(900.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(30.0),
                    ..default()
                })
                .with_children(|content| {
                    // Left: Selected ship detail panel
                    spawn_ship_detail_panel(
                        content,
                        &ships[0],
                        faction_color,
                        max_speed,
                        max_damage,
                        max_health,
                        max_fire_rate,
                        sprite_cache.get(ships[0].type_id),
                    );

                    // Right: Ship list
                    content
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|list| {
                            for (i, ship) in ships.iter().enumerate() {
                                let is_unlocked = save_data.is_ship_unlocked(
                                    ship.type_id,
                                    ship.unlock_stage,
                                    faction.short_name(),
                                    enemy.short_name(),
                                );
                                spawn_ship_list_item(list, ship, i, is_unlocked, faction_color);
                            }
                        });
                });

            // Navigation hint
            parent.spawn((
                Text::new("D-PAD Navigate  •  A Select  •  B Back"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

/// Spawn the detailed ship info panel (left side)
fn spawn_ship_detail_panel(
    parent: &mut ChildBuilder,
    ship: &ShipDef,
    faction_color: Color,
    max_speed: f32,
    max_damage: f32,
    max_health: f32,
    max_fire_rate: f32,
    ship_image: Option<Handle<Image>>,
) {
    parent
        .spawn((
            ShipDetailPanel,
            Node {
                width: Val::Px(380.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|panel| {
            // Ship name (large)
            panel.spawn((
                ShipDetailName,
                Text::new(ship.name),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            // Class and role
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        ShipDetailClass,
                        Text::new(ship.class.name()),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                    row.spawn((
                        Text::new("•"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                    row.spawn((
                        ShipDetailRole,
                        Text::new(ship.role),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });

            // Divider
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
            ));

            // Stat bars section
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|stats| {
                    spawn_stat_bar(
                        stats,
                        "SPEED",
                        ship.speed,
                        max_speed,
                        Color::srgb(0.3, 0.8, 0.3),
                        StatType::Speed,
                    );
                    spawn_stat_bar(
                        stats,
                        "DAMAGE",
                        ship.damage,
                        max_damage,
                        Color::srgb(0.9, 0.3, 0.3),
                        StatType::Damage,
                    );
                    spawn_stat_bar(
                        stats,
                        "HEALTH",
                        ship.health,
                        max_health,
                        Color::srgb(0.3, 0.6, 0.9),
                        StatType::Health,
                    );
                    spawn_stat_bar(
                        stats,
                        "FIRE RATE",
                        ship.fire_rate,
                        max_fire_rate,
                        Color::srgb(0.9, 0.7, 0.3),
                        StatType::FireRate,
                    );
                });

            // Divider
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
            ));

            // Special ability
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|special| {
                    special.spawn((
                        Text::new("SPECIAL ABILITY"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                    special.spawn((
                        ShipDetailSpecial,
                        Text::new(ship.special),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.8, 1.0)),
                    ));
                });

            // Hull preview — fills the empty space under special ability.
            // Centered, fixed aspect, faction-tinted backdrop.
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(220.0),
                        margin: UiRect::top(Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                    BorderRadius::all(Val::Px(6.0)),
                ))
                .with_children(|frame| {
                    let mut image_node = ImageNode::default();
                    if let Some(handle) = ship_image {
                        image_node.image = handle;
                    }
                    frame.spawn((
                        ShipDetailSprite,
                        Node {
                            width: Val::Px(200.0),
                            height: Val::Px(200.0),
                            ..default()
                        },
                        image_node,
                    ));
                });
        });
}

/// Spawn a stat bar with label and fill
fn spawn_stat_bar(
    parent: &mut ChildBuilder,
    label: &str,
    value: f32,
    max_value: f32,
    color: Color,
    stat_type: StatType,
) {
    let percent = (value / max_value * 100.0).clamp(0.0, 100.0);

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            ..default()
        })
        .with_children(|stat| {
            // Label row with value
            stat.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(label),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(color),
                ));
                row.spawn((
                    Text::new(format!("{:.0}", value)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            });

            // Bar background
            stat.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
                BorderRadius::all(Val::Px(2.0)),
            ))
            .with_children(|bar| {
                // Bar fill
                bar.spawn((
                    StatBarFill(stat_type),
                    Node {
                        width: Val::Percent(percent),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderRadius::all(Val::Px(2.0)),
                ));
            });
        });
}

/// Spawn a compact ship list item (right side)
fn spawn_ship_list_item(
    parent: &mut ChildBuilder,
    ship: &ShipDef,
    index: usize,
    is_unlocked: bool,
    faction_color: Color,
) {
    let name_color = if is_unlocked {
        faction_color
    } else {
        Color::srgb(0.35, 0.35, 0.35)
    };
    let bg_color = if is_unlocked {
        Color::srgba(0.1, 0.1, 0.12, 0.9)
    } else {
        Color::srgba(0.06, 0.06, 0.08, 0.9)
    };
    let border_color = if is_unlocked {
        Color::srgb(0.25, 0.25, 0.3)
    } else {
        Color::srgb(0.15, 0.15, 0.18)
    };

    parent
        .spawn((
            ShipMenuRoot,
            MenuItem { index },
            Node {
                width: Val::Px(280.0),
                padding: UiRect::all(Val::Px(12.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|btn| {
            // Left: Name and class
            btn.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|left| {
                let name_text = if is_unlocked {
                    ship.name.to_string()
                } else {
                    format!("🔒 {}", ship.name)
                };
                left.spawn((
                    Text::new(name_text),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(name_color),
                ));
                left.spawn((
                    Text::new(ship.class.name()),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            });

            // Right: Quick stats
            if is_unlocked {
                btn.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                })
                .with_children(|right| {
                    right.spawn((
                        Text::new(format!("DMG {:.0}", ship.damage)),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.4, 0.4)),
                    ));
                    right.spawn((
                        Text::new(format!("SPD {:.0}", ship.speed)),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.6, 0.4)),
                    ));
                });
            } else {
                btn.spawn((
                    Text::new(format!("Stage {}", ship.unlock_stage)),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.3, 0.3)),
                ));
            }
        });
}

/// Update detail panel when selection changes
pub(crate) fn update_ship_detail_panel(
    selection: Res<MenuSelection>,
    session: Res<GameSession>,
    mut name_query: Query<
        &mut Text,
        (
            With<ShipDetailName>,
            Without<ShipDetailClass>,
            Without<ShipDetailRole>,
            Without<ShipDetailSpecial>,
        ),
    >,
    mut class_query: Query<
        &mut Text,
        (
            With<ShipDetailClass>,
            Without<ShipDetailName>,
            Without<ShipDetailRole>,
            Without<ShipDetailSpecial>,
        ),
    >,
    mut role_query: Query<
        &mut Text,
        (
            With<ShipDetailRole>,
            Without<ShipDetailName>,
            Without<ShipDetailClass>,
            Without<ShipDetailSpecial>,
        ),
    >,
    mut special_query: Query<
        &mut Text,
        (
            With<ShipDetailSpecial>,
            Without<ShipDetailName>,
            Without<ShipDetailClass>,
            Without<ShipDetailRole>,
        ),
    >,
    mut stat_bars: Query<(&StatBarFill, &mut Node)>,
    mut sprite_query: Query<&mut ImageNode, With<ShipDetailSprite>>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
) {
    if !selection.is_changed() {
        return;
    }

    let ships = session.player_ships();
    if selection.index >= ships.len() {
        return;
    }

    let ship = &ships[selection.index];

    // Calculate stat ranges for normalization
    let max_speed = ships.iter().map(|s| s.speed).fold(0.0_f32, f32::max);
    let max_damage = ships.iter().map(|s| s.damage).fold(0.0_f32, f32::max);
    let max_health = ships.iter().map(|s| s.health).fold(0.0_f32, f32::max);
    let max_fire_rate = ships.iter().map(|s| s.fire_rate).fold(0.0_f32, f32::max);

    // Update text fields
    for mut text in name_query.iter_mut() {
        **text = ship.name.to_string();
    }
    for mut text in class_query.iter_mut() {
        **text = ship.class.name().to_string();
    }
    for mut text in role_query.iter_mut() {
        **text = ship.role.to_string();
    }
    for mut text in special_query.iter_mut() {
        **text = ship.special.to_string();
    }

    // Update stat bars
    for (stat_fill, mut node) in stat_bars.iter_mut() {
        let (value, max) = match stat_fill.0 {
            StatType::Speed => (ship.speed, max_speed),
            StatType::Damage => (ship.damage, max_damage),
            StatType::Health => (ship.health, max_health),
            StatType::FireRate => (ship.fire_rate, max_fire_rate),
        };
        let percent = (value / max * 100.0).clamp(0.0, 100.0);
        node.width = Val::Percent(percent);
    }

    // Swap preview sprite to the newly-selected hull.
    if let Some(handle) = sprite_cache.get(ship.type_id) {
        for mut image_node in sprite_query.iter_mut() {
            image_node.image = handle.clone();
        }
    }
}

pub(crate) fn ship_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut session: ResMut<GameSession>,
    time: Res<Time>,
    mut transitions: EventWriter<TransitionEvent>,
    save_data: Res<crate::core::SaveData>,
) {
    selection.cooldown -= time.delta_secs();

    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    let ships = session.player_ships();
    let faction = session.player_faction;
    let enemy = session.enemy_faction;

    if is_confirm(&keyboard, &joystick) && selection.index < ships.len() {
        let ship = &ships[selection.index];
        let is_unlocked = save_data.is_ship_unlocked(
            ship.type_id,
            ship.unlock_stage,
            faction.short_name(),
            enemy.short_name(),
        );

        if is_unlocked {
            session.selected_ship_index = selection.index;
            info!("Selected ship: {} ({})", ship.name, ship.class.name());
            // Show mission briefing before gameplay
            transitions.send(TransitionEvent::quick(GameState::MissionBriefing));
        } else {
            info!(
                "Ship {} is locked - complete Stage {} to unlock",
                ship.name, ship.unlock_stage
            );
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        transitions.send(TransitionEvent::quick(GameState::DifficultySelect));
    }
}
