//! Main Menu

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::games::ActiveModule;
use crate::games::caldari_gallente::{CGCampaignState, VerticalSliceMode};
use crate::systems::JoystickState;
use crate::ui::TransitionEvent;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct MainMenuRoot;

pub(crate) fn spawn_main_menu(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    save_data: Res<SaveData>,
) {
    selection.index = 0;
    selection.total = 4; // PLAY SLICE, UPGRADES, OPTIONS, QUIT

    // Get best high score across all faction pairs
    let best_score = save_data
        .high_scores
        .iter()
        .map(|hs| hs.score)
        .max()
        .unwrap_or(0);

    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("REBELLION"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.5, 0.2)), // Orange/gold
            ));

            parent.spawn((
                Text::new("THE ELDER FLEET RISES"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.4, 0.2)), // Bronze/copper
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(40.0),
                ..default()
            });

            // Menu buttons
            spawn_menu_item(parent, "PLAY VERTICAL SLICE", 0);
            spawn_menu_item(parent, "UPGRADES", 1);
            spawn_menu_item(parent, "OPTIONS", 2);
            spawn_menu_item(parent, "QUIT", 3);

            // High score display
            if best_score > 0 {
                parent.spawn(Node {
                    height: Val::Px(20.0),
                    ..default()
                });

                parent
                    .spawn((
                        Node {
                            padding: UiRect::new(
                                Val::Px(20.0),
                                Val::Px(20.0),
                                Val::Px(8.0),
                                Val::Px(8.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::srgb(0.3, 0.25, 0.15)),
                        BackgroundColor(Color::srgba(0.1, 0.08, 0.05, 0.8)),
                    ))
                    .with_children(|score_box| {
                        score_box.spawn((
                            Text::new("HIGH SCORE"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.4, 0.3)),
                        ));
                        score_box.spawn((
                            Text::new(format_score(best_score)),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.85, 0.3)),
                        ));
                    });
            }

            // Footer
            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            parent.spawn((
                Text::new("D-PAD Navigate  •  A Select  •  START Quit"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            parent.spawn((
                Text::new("v2.1.0 — Vertical Slice"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

pub(crate) fn main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    time: Res<Time>,
    mut active_module: ResMut<ActiveModule>,
    mut game_session: ResMut<GameSession>,
    mut difficulty: ResMut<Difficulty>,
    mut cg_campaign: ResMut<CGCampaignState>,
    mut slice_mode: ResMut<VerticalSliceMode>,
    itch_mode: Res<ItchMode>,
    mut exit: EventWriter<AppExit>,
    mut transitions: EventWriter<TransitionEvent>,
) {
    selection.cooldown -= time.delta_secs();

    // Navigation
    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    // Selection
    if is_confirm(&keyboard, &joystick) {
        match selection.index {
            0 => {
                // PLAY
                if itch_mode.enabled && !itch_mode.completed_first_run {
                    // ItchMode: skip all selection screens and auto-configure
                    // Archive 01 — Caldari Prime vertical slice
                    active_module.set_module("caldari_gallente");
                    active_module.set_faction("caldari", "gallente");
                    *game_session = GameSession::new(
                        crate::core::Faction::Caldari,
                        crate::core::Faction::Gallente,
                    );
                    game_session.selected_ship_index = 0; // Kestrel
                    *difficulty = Difficulty::Newbro;
                    cg_campaign.mission_index = 0;
                    cg_campaign.current_wave = 1;
                    cg_campaign.in_mission = false;
                    cg_campaign.boss_spawned = false;
                    cg_campaign.boss_defeated = false;
                    cg_campaign.t3_unlocked = false;
                    *slice_mode = VerticalSliceMode::Slice;
                    transitions.send(TransitionEvent::to(GameState::MissionBriefing));
                } else {
                    transitions.send(TransitionEvent::to(GameState::ModuleSelect));
                }
            }
            1 => {
                // UPGRADES - go to upgrade shop
                transitions.send(TransitionEvent::to(GameState::UpgradeShop));
            }
            2 => {
                // OPTIONS - go to options menu
                transitions.send(TransitionEvent::to(GameState::Options));
            }
            3 => {
                exit.send(AppExit::Success);
            }
            _ => {}
        }
    }

    // Quick quit
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        exit.send(AppExit::Success);
    }
}
