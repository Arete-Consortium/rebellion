//! Death Screen (frozen corpse in wreckage)

#![allow(dead_code)]

use bevy::prelude::*;
use crate::core::*;
use crate::systems::JoystickState;
use crate::ui::TransitionEvent;
use super::common::*;

#[derive(Component)]
pub(crate) struct GameOverRoot;

/// Death screen floating debris
#[derive(Component)]
pub(crate) struct DeathDebris {
    pub(crate) velocity: Vec2,
    pub(crate) spin: f32,
}

/// Death screen corpse
#[derive(Component)]
pub(crate) struct DeathCorpse {
    pub(crate) velocity: Vec2,
    pub(crate) spin: f32,
}

/// Death screen button
#[derive(Component)]
pub(crate) struct DeathButton {
    pub(crate) action: DeathAction,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum DeathAction {
    Retry,
    Exit,
}

/// Death screen selection state
#[derive(Resource)]
pub(crate) struct DeathSelection {
    pub(crate) selected: DeathAction,
}

impl Default for DeathSelection {
    fn default() -> Self {
        Self {
            selected: DeathAction::Retry,
        }
    }
}

/// UI amber color
const COLOR_AMBER: Color = Color::srgb(0.83, 0.66, 0.29);
const COLOR_AMBER_BRIGHT: Color = Color::srgb(1.0, 0.8, 0.0);

pub(crate) fn spawn_death_screen(
    mut commands: Commands,
    score: Res<ScoreSystem>,
    campaign: Res<CampaignState>,
    mut endless: ResMut<crate::core::EndlessMode>,
    mut nightmare: ResMut<crate::games::caldari_gallente::ShiigeruNightmare>,
    session: Res<GameSession>,
    save_data: Res<SaveData>,
) {
    // Initialize selection resource
    commands.insert_resource(DeathSelection::default());

    // End endless run if active
    let was_endless = endless.active;
    if was_endless {
        endless.end_run();
    }

    // End nightmare run if active
    let was_nightmare = nightmare.active;
    let nightmare_stats = if was_nightmare {
        Some((
            nightmare.wave,
            nightmare.time_survived,
            nightmare.kills,
            nightmare.mini_bosses_defeated,
            nightmare.wave > nightmare.best_wave,
            nightmare.time_survived > nightmare.best_time,
        ))
    } else {
        None
    };
    if was_nightmare {
        nightmare.end();
    }

    // Get high score for comparison
    let high_score =
        save_data.get_high_score(session.player_faction.name(), session.enemy_faction.name());
    let is_new_high = score.score > high_score && score.score > 0;

    // Get mission info - different for endless/nightmare mode
    let mission_name = if was_nightmare {
        "SHIIGERU NIGHTMARE".to_string()
    } else if was_endless {
        format!("Endless Wave {}", endless.wave)
    } else {
        campaign.current_mission_name().to_string()
    };

    // Spawn debris field (background sprites)
    let debris_colors = [
        Color::srgb(0.31, 0.24, 0.20), // Rusty brown
        Color::srgb(0.24, 0.24, 0.25), // Dark gray
        Color::srgb(0.35, 0.27, 0.22), // Warm rust
        Color::srgb(0.20, 0.20, 0.22), // Cold gray
    ];

    for i in 0..25 {
        let x = (fastrand::f32() - 0.5) * SCREEN_WIDTH;
        let y = (fastrand::f32() - 0.5) * SCREEN_HEIGHT;
        let size = 4.0 + fastrand::f32() * 12.0;
        let color = debris_colors[i % debris_colors.len()];

        commands.spawn((
            GameOverRoot,
            DeathDebris {
                velocity: Vec2::new((fastrand::f32() - 0.5) * 8.0, (fastrand::f32() - 0.5) * 5.0),
                spin: (fastrand::f32() - 0.5) * 0.5,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::new(size * 2.0, size)),
                ..default()
            },
            Transform::from_xyz(x, y, 1.0).with_rotation(Quat::from_rotation_z(
                fastrand::f32() * std::f32::consts::TAU,
            )),
        ));
    }

    // Spawn frozen corpse (center of screen)
    commands.spawn((
        GameOverRoot,
        DeathCorpse {
            velocity: Vec2::new((fastrand::f32() - 0.5) * 3.0, (fastrand::f32() - 0.5) * 2.0),
            spin: (fastrand::f32() - 0.5) * 0.2,
        },
        Sprite {
            color: Color::srgb(0.27, 0.25, 0.24), // Frozen body color
            custom_size: Some(Vec2::new(40.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 50.0, 5.0)
            .with_rotation(Quat::from_rotation_z(fastrand::f32() * 0.5)),
    ));

    // Spawn UI overlay
    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 0.85)),
        ))
        .with_children(|parent| {
            // Title - "CLONE LOST"
            parent.spawn((
                Text::new("CLONE LOST"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(COLOR_AMBER),
            ));

            // Mission failed info
            parent.spawn((
                Text::new(format!("Mission Failed: {}", mission_name)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.4, 0.4)),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // New high score banner (if achieved)
            if is_new_high {
                parent.spawn((
                    Text::new("★ NEW HIGH SCORE ★"),
                    TextFont {
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.9, 0.0)),
                ));
            }

            // Final score
            parent.spawn((
                Text::new(format!("FINAL SCORE: {}", format_score(score.score))),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(COLOR_AMBER),
            ));

            // Previous high score (if not beaten)
            if !is_new_high && high_score > 0 {
                parent.spawn((
                    Text::new(format!("High Score: {}", format_score(high_score))),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            }

            // Stats row - different for nightmare mode
            if let Some((wave, time, kills, bosses, new_wave_record, new_time_record)) =
                nightmare_stats
            {
                // Nightmare-specific stats
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|col| {
                        // Wave reached
                        let wave_text = if new_wave_record {
                            format!("★ WAVE {} (NEW RECORD!) ★", wave)
                        } else {
                            format!("Wave Reached: {}", wave)
                        };
                        col.spawn((
                            Text::new(wave_text),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(if new_wave_record {
                                Color::srgb(1.0, 0.9, 0.0)
                            } else {
                                Color::srgb(0.9, 0.5, 0.5)
                            }),
                        ));

                        // Time survived
                        let mins = (time / 60.0) as u32;
                        let secs = (time % 60.0) as u32;
                        let time_text = if new_time_record {
                            format!("★ {:02}:{:02} (NEW RECORD!) ★", mins, secs)
                        } else {
                            format!("Time Survived: {:02}:{:02}", mins, secs)
                        };
                        col.spawn((
                            Text::new(time_text),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(if new_time_record {
                                Color::srgb(1.0, 0.9, 0.0)
                            } else {
                                Color::srgb(0.7, 0.7, 0.7)
                            }),
                        ));

                        // Kills and bosses row
                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(30.0),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new(format!("Kills: {}", kills)),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.59, 0.51, 0.35)),
                            ));

                            if bosses > 0 {
                                row.spawn((
                                    Text::new(format!("Mini-Bosses: {}", bosses)),
                                    TextFont {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.59, 0.51, 0.35)),
                                ));
                            }
                        });
                    });
            } else {
                // Regular campaign stats
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(30.0),
                        ..default()
                    })
                    .with_children(|row| {
                        if score.souls_liberated > 0 {
                            row.spawn((
                                Text::new(format!("Souls: {}", score.souls_liberated)),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.59, 0.51, 0.35)),
                            ));
                        }

                        if score.chain > 1 {
                            row.spawn((
                                Text::new(format!("Chain: {}x", score.chain)),
                                TextFont {
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.59, 0.51, 0.35)),
                            ));
                        }

                        row.spawn((
                            Text::new(format!(
                                "Stage {}-{}",
                                campaign.stage_number(),
                                campaign.mission_in_stage()
                            )),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.59, 0.51, 0.35)),
                        ));
                    });
            }

            // Spacer
            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Button row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(40.0),
                    ..default()
                })
                .with_children(|row| {
                    // RETRY button
                    row.spawn((
                        DeathButton {
                            action: DeathAction::Retry,
                        },
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(COLOR_AMBER),
                        BackgroundColor(Color::srgba(0.83, 0.66, 0.29, 0.1)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("RETRY"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(COLOR_AMBER),
                        ));
                    });

                    // EXIT button
                    row.spawn((
                        DeathButton {
                            action: DeathAction::Exit,
                        },
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(COLOR_AMBER),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("EXIT"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(COLOR_AMBER),
                        ));
                    });
                });

            // Spacer
            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Flavor text
            parent.spawn((
                Text::new("\"You fall... but the Fleet continues.\""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Controller hint
            parent.spawn((
                Text::new("← → Navigate • A/ENTER Select • B/ESC Quit"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
            ));
        });
}

pub(crate) fn update_death_screen_animation(
    time: Res<Time>,
    mut debris_query: Query<(&mut Transform, &DeathDebris), Without<DeathCorpse>>,
    mut corpse_query: Query<(&mut Transform, &DeathCorpse), Without<DeathDebris>>,
    selection: Res<DeathSelection>,
    mut button_query: Query<(&DeathButton, &mut BorderColor, &mut BackgroundColor)>,
) {
    let dt = time.delta_secs();

    // Animate debris
    for (mut transform, debris) in debris_query.iter_mut() {
        transform.translation.x += debris.velocity.x * dt;
        transform.translation.y += debris.velocity.y * dt;
        transform.rotate_z(debris.spin * dt);

        // Wrap around screen
        if transform.translation.x < -SCREEN_WIDTH / 2.0 - 20.0 {
            transform.translation.x = SCREEN_WIDTH / 2.0 + 20.0;
        }
        if transform.translation.x > SCREEN_WIDTH / 2.0 + 20.0 {
            transform.translation.x = -SCREEN_WIDTH / 2.0 - 20.0;
        }
        if transform.translation.y < -SCREEN_HEIGHT / 2.0 - 20.0 {
            transform.translation.y = SCREEN_HEIGHT / 2.0 + 20.0;
        }
        if transform.translation.y > SCREEN_HEIGHT / 2.0 + 20.0 {
            transform.translation.y = -SCREEN_HEIGHT / 2.0 - 20.0;
        }
    }

    // Animate corpse (slower, more constrained)
    for (mut transform, corpse) in corpse_query.iter_mut() {
        transform.translation.x += corpse.velocity.x * dt;
        transform.translation.y += corpse.velocity.y * dt;
        transform.rotate_z(corpse.spin * dt);

        // Keep corpse near center
        if transform.translation.x.abs() > 100.0 {
            transform.translation.x *= 0.99;
        }
        if transform.translation.y.abs() > 80.0 {
            transform.translation.y *= 0.99;
        }
    }

    // Update button highlights
    for (button, mut border, mut bg) in button_query.iter_mut() {
        if button.action == selection.selected {
            border.0 = COLOR_AMBER_BRIGHT;
            bg.0 = Color::srgba(1.0, 0.8, 0.0, 0.15);
        } else {
            border.0 = COLOR_AMBER;
            bg.0 = Color::NONE;
        }
    }
}

pub(crate) fn death_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<DeathSelection>,
    mut score: ResMut<ScoreSystem>,
    mut campaign: ResMut<CampaignState>,
    mut transitions: EventWriter<TransitionEvent>,
) {
    // Navigation
    if keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
        || joystick.dpad_x < 0
    {
        selection.selected = DeathAction::Retry;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
        || joystick.dpad_x > 0
    {
        selection.selected = DeathAction::Exit;
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        match selection.selected {
            DeathAction::Retry => {
                score.reset_game();
                *campaign = CampaignState::default();
                transitions.send(TransitionEvent::to(GameState::ShipSelect));
            }
            DeathAction::Exit => {
                transitions.send(TransitionEvent::to(GameState::MainMenu));
            }
        }
    }

    // Quick exit
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        transitions.send(TransitionEvent::to(GameState::MainMenu));
    }
}

pub(crate) fn despawn_death_screen(mut commands: Commands, query: Query<Entity, With<GameOverRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<DeathSelection>();
}
