//! Stage Select - 13 Stages across 3 Acts

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::systems::JoystickState;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct StageSelectRoot;

#[derive(Component)]
pub(crate) struct StageCard {
    pub(crate) stage: u32,
    pub(crate) locked: bool,
}

/// Stage definition for UI display
struct StageInfo {
    stage: u32,
    name: &'static str,
    act: u32,
    boss_name: &'static str,
    waves: u32,
}

const STAGES: [StageInfo; 13] = [
    // Act 1: The Call
    StageInfo {
        stage: 1,
        name: "Border Patrol",
        act: 1,
        boss_name: "Bestower",
        waves: 5,
    },
    StageInfo {
        stage: 2,
        name: "Slave Convoy",
        act: 1,
        boss_name: "Navy Omen",
        waves: 6,
    },
    StageInfo {
        stage: 3,
        name: "Outpost Assault",
        act: 1,
        boss_name: "Platform",
        waves: 7,
    },
    StageInfo {
        stage: 4,
        name: "Holder's Guard",
        act: 1,
        boss_name: "Mixed Fleet",
        waves: 8,
    },
    // Act 2: The Storm
    StageInfo {
        stage: 5,
        name: "Deep Patrol",
        act: 2,
        boss_name: "Prophecy",
        waves: 8,
    },
    StageInfo {
        stage: 6,
        name: "Inquisitor",
        act: 2,
        boss_name: "Prophecy",
        waves: 9,
    },
    StageInfo {
        stage: 7,
        name: "Strike Group",
        act: 2,
        boss_name: "Harbinger",
        waves: 10,
    },
    StageInfo {
        stage: 8,
        name: "Gate Defense",
        act: 2,
        boss_name: "Stargate",
        waves: 10,
    },
    StageInfo {
        stage: 9,
        name: "Station Core",
        act: 2,
        boss_name: "Station",
        waves: 12,
    },
    // Act 3: Liberation
    StageInfo {
        stage: 10,
        name: "Admiral's Ship",
        act: 3,
        boss_name: "Armageddon",
        waves: 12,
    },
    StageInfo {
        stage: 11,
        name: "Divine Carrier",
        act: 3,
        boss_name: "Archon",
        waves: 14,
    },
    StageInfo {
        stage: 12,
        name: "Lord Admiral",
        act: 3,
        boss_name: "Apocalypse",
        waves: 15,
    },
    StageInfo {
        stage: 13,
        name: "Avatar Titan",
        act: 3,
        boss_name: "AVATAR",
        waves: 20,
    },
];

pub(crate) fn spawn_stage_select(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    session: Res<GameSession>,
    save_data: Res<crate::core::SaveData>,
) {
    let faction = session.player_faction;
    let enemy = session.enemy_faction;
    let highest = save_data.get_highest_stage(faction.short_name(), enemy.short_name());

    selection.index = 0;
    selection.total = 13;

    commands
        .spawn((
            StageSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SELECT STAGE"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.7, 0.3)),
            ));

            // Progress indicator
            parent.spawn((
                Text::new(format!("Progress: Stage {} / 13", highest.min(13))),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            parent.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });

            // Act 1 row
            spawn_act_row(parent, 1, "THE CALL", &STAGES[0..4], highest);

            // Act 2 row
            spawn_act_row(parent, 2, "THE STORM", &STAGES[4..9], highest);

            // Act 3 row
            spawn_act_row(parent, 3, "LIBERATION", &STAGES[9..13], highest);

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Instructions
            parent.spawn((
                Text::new("D-PAD Navigate  •  A Select  •  B Back"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn spawn_act_row(
    parent: &mut ChildBuilder,
    act: u32,
    act_name: &str,
    stages: &[StageInfo],
    highest_cleared: u32,
) {
    let act_color = match act {
        1 => Color::srgb(0.8, 0.5, 0.2), // Orange - Rifter
        2 => Color::srgb(0.6, 0.3, 0.1), // Brown - Wolf
        3 => Color::srgb(0.9, 0.7, 0.3), // Gold - Jaguar
        _ => Color::WHITE,
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|act_col| {
            // Act header
            act_col.spawn((
                Text::new(format!("ACT {}: {}", act, act_name)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(act_color),
            ));

            // Stage cards row — flex-wrap so the 5-card act fits on
            // narrow portrait phones (375px) without clipping.
            act_col
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(10.0),
                    row_gap: Val::Px(10.0),
                    max_width: Val::Percent(94.0),
                    ..default()
                })
                .with_children(|row| {
                    for stage in stages {
                        let locked = stage.stage > highest_cleared + 1;
                        spawn_stage_card(row, stage, locked, act_color);
                    }
                });
        });
}

fn spawn_stage_card(parent: &mut ChildBuilder, stage: &StageInfo, locked: bool, act_color: Color) {
    let bg_color = if locked {
        Color::srgba(0.2, 0.2, 0.2, 0.8)
    } else {
        act_color.with_alpha(0.3)
    };

    let border_color = if locked {
        Color::srgb(0.3, 0.3, 0.3)
    } else {
        act_color
    };

    let text_color = if locked {
        Color::srgb(0.4, 0.4, 0.4)
    } else {
        Color::WHITE
    };

    parent
        .spawn((
            MenuItem {
                index: (stage.stage - 1) as usize,
            },
            StageCard {
                stage: stage.stage,
                locked,
            },
            Node {
                // Smaller cards so a 5-card act wraps to 3+2 on a 375px
                // portrait viewport (3 × 105 + 2 × 10 = 335px).
                width: Val::Px(105.0),
                height: Val::Px(82.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
        ))
        .with_children(|card| {
            // Stage number
            card.spawn((
                Text::new(format!("STAGE {}", stage.stage)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if locked {
                    Color::srgb(0.5, 0.5, 0.5)
                } else {
                    act_color
                }),
            ));

            // Stage name
            card.spawn((
                Text::new(stage.name),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(text_color),
            ));

            // Boss or lock icon
            if locked {
                card.spawn((
                    Text::new("🔒"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            } else {
                card.spawn((
                    Text::new(format!("vs {}", stage.boss_name)),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            }
        });
}

pub(crate) fn stage_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut campaign: ResMut<CampaignState>,
    session: Res<GameSession>,
    save_data: Res<crate::core::SaveData>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cards: Query<(
        &MenuItem,
        &StageCard,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    selection.cooldown -= time.delta_secs();

    let faction = session.player_faction;
    let enemy = session.enemy_faction;
    let highest = save_data.get_highest_stage(faction.short_name(), enemy.short_name());

    // Navigation - horizontal within acts
    let nav_h = get_nav_input(&keyboard, &joystick);
    // Vertical navigation between acts
    let nav_v = if keyboard.just_pressed(KeyCode::ArrowUp) || joystick.dpad_just_up() {
        -1
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || joystick.dpad_just_down() {
        1
    } else {
        0
    };

    if selection.cooldown <= 0.0 {
        if nav_h != 0 {
            // Move within the same row where possible
            let new_idx = (selection.index as i32 + nav_h).clamp(0, 12) as usize;
            selection.index = new_idx;
            selection.cooldown = MENU_NAV_COOLDOWN;
        }
        if nav_v != 0 {
            // Jump between acts (4 stages in act 1, 5 in act 2, 4 in act 3)
            let current = selection.index;
            let new_idx = if nav_v < 0 {
                // Up
                match current {
                    0..=3 => current,      // Act 1, stay
                    4..=8 => current - 4,  // Act 2 -> Act 1
                    9..=12 => current - 5, // Act 3 -> Act 2
                    _ => current,
                }
            } else {
                // Down
                match current {
                    0..=3 => (current + 4).min(8),  // Act 1 -> Act 2
                    4..=8 => (current + 5).min(12), // Act 2 -> Act 3
                    9..=12 => current,              // Act 3, stay
                    _ => current,
                }
            };
            selection.index = new_idx;
            selection.cooldown = MENU_NAV_COOLDOWN;
        }
    }

    // Update card highlights
    let act_colors = [
        Color::srgb(0.8, 0.5, 0.2), // Act 1
        Color::srgb(0.6, 0.3, 0.1), // Act 2
        Color::srgb(0.9, 0.7, 0.3), // Act 3
    ];

    for (item, card, mut bg, mut border) in cards.iter_mut() {
        let act_idx = if card.stage <= 4 {
            0
        } else if card.stage <= 9 {
            1
        } else {
            2
        };
        let act_color = act_colors[act_idx];
        let is_selected = item.index == selection.index;

        if card.locked {
            if is_selected {
                *bg = BackgroundColor(Color::srgba(0.3, 0.2, 0.2, 0.9));
                *border = BorderColor(Color::srgb(0.5, 0.3, 0.3));
            } else {
                *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
                *border = BorderColor(Color::srgb(0.3, 0.3, 0.3));
            }
        } else if is_selected {
            *bg = BackgroundColor(act_color.with_alpha(0.6));
            *border = BorderColor(Color::WHITE);
        } else {
            *bg = BackgroundColor(act_color.with_alpha(0.3));
            *border = BorderColor(act_color);
        }
    }

    // Confirm selection
    if is_confirm(&keyboard, &joystick) {
        let stage = (selection.index + 1) as u32;
        let locked = stage > highest + 1;

        if !locked {
            // Set campaign state to selected stage
            let act = if stage <= 4 {
                crate::core::Act::Act1
            } else if stage <= 9 {
                crate::core::Act::Act2
            } else {
                crate::core::Act::Act3
            };

            let mission_idx = match act {
                crate::core::Act::Act1 => (stage - 1) as usize,
                crate::core::Act::Act2 => (stage - 5) as usize,
                crate::core::Act::Act3 => (stage - 10) as usize,
            };

            campaign.act = act;
            campaign.mission_index = mission_idx;

            info!(
                "Selected Stage {} (Act {:?}, Mission {})",
                stage,
                act,
                mission_idx + 1
            );
            next_state.set(GameState::ShipSelect);
        }
    }

    // Back
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::FactionSelect);
    }
}
