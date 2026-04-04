//! Victory Screen (Campaign Complete)

#![allow(dead_code)]

use bevy::prelude::*;
use crate::core::*;
use crate::systems::JoystickState;
use crate::ui::TransitionEvent;
use super::common::*;

#[derive(Component)]
pub(crate) struct VictoryRoot;

/// Marker for victory celebration particles
#[derive(Component)]
pub(crate) struct VictoryParticle {
    pub(crate) velocity: Vec2,
    pub(crate) lifetime: f32,
    pub(crate) max_lifetime: f32,
}

/// Victory screen button
#[derive(Component)]
pub(crate) struct VictoryButton {
    pub(crate) action: VictoryAction,
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum VictoryAction {
    PlayAgain,
    MainMenu,
}

/// Victory screen selection state
#[derive(Resource)]
pub(crate) struct VictorySelection {
    pub(crate) selected: VictoryAction,
}

impl Default for VictorySelection {
    fn default() -> Self {
        Self {
            selected: VictoryAction::PlayAgain,
        }
    }
}

pub(crate) fn spawn_victory_screen(
    mut commands: Commands,
    score: Res<ScoreSystem>,
    session: Res<GameSession>,
    campaign: Res<CampaignState>,
    mut save_data: ResMut<SaveData>,
) {
    // Initialize selection
    commands.insert_resource(VictorySelection::default());

    // Check for new high score
    let previous_high =
        save_data.get_high_score(session.player_faction.name(), session.enemy_faction.name());
    let is_new_high_score = score.score > previous_high;

    // Record the score if it's a new high
    if is_new_high_score {
        save_data.record_score(
            session.player_faction.name(),
            session.enemy_faction.name(),
            score.score,
            campaign.stage_number(),
        );
    }

    // Spawn celebration particles
    for _ in 0..60 {
        let x = (fastrand::f32() - 0.5) * SCREEN_WIDTH;
        let y = -SCREEN_HEIGHT / 2.0 - fastrand::f32() * 100.0;
        let vx = (fastrand::f32() - 0.5) * 100.0;
        let vy = 80.0 + fastrand::f32() * 120.0;
        let size = 4.0 + fastrand::f32() * 8.0;
        let lifetime = 3.0 + fastrand::f32() * 4.0;

        // Gold/amber particles
        let color = if fastrand::bool() {
            Color::srgb(1.0, 0.85, 0.2) // Gold
        } else {
            Color::srgb(0.85, 0.4, 0.15) // Minmatar rust
        };

        commands.spawn((
            VictoryRoot,
            VictoryParticle {
                velocity: Vec2::new(vx, vy),
                lifetime,
                max_lifetime: lifetime,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(x, y, 50.0),
        ));
    }

    // Main UI container
    commands
        .spawn((
            VictoryRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.05, 0.9)),
        ))
        .with_children(|parent| {
            // Victory header
            parent.spawn((
                Text::new("LIBERATION COMPLETE"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)), // Gold
            ));

            parent.spawn((
                Text::new("The Amarr Empire Has Fallen"),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Campaign stats box
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.8, 0.6, 0.2)),
                    BackgroundColor(Color::srgba(0.1, 0.08, 0.02, 0.8)),
                ))
                .with_children(|stats| {
                    // New high score banner
                    if is_new_high_score {
                        stats.spawn((
                            Text::new("★ NEW HIGH SCORE ★"),
                            TextFont {
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.9, 0.0)),
                        ));
                    }

                    stats.spawn((
                        Text::new(format!("FINAL SCORE: {}", format_score(score.score))),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                    ));

                    // Show previous high if not beaten
                    if !is_new_high_score && previous_high > 0 {
                        stats.spawn((
                            Text::new(format!("High Score: {}", format_score(previous_high))),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                    }

                    stats.spawn((
                        Text::new(format!("Souls Liberated: {}", score.souls_liberated)),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.85, 1.0)),
                    ));

                    stats.spawn((
                        Text::new(format!("Kill Multiplier: {:.1}x", score.multiplier)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.6, 0.3)),
                    ));
                });

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Elder's final words
            parent.spawn((
                Text::new("\"Our ancestors smile upon us this day.\""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
            ));

            parent.spawn((
                Text::new("— Elder Drupar Maak"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.6)),
            ));

            parent.spawn(Node {
                height: Val::Px(25.0),
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
                    // PLAY AGAIN button
                    row.spawn((
                        VictoryButton {
                            action: VictoryAction::PlayAgain,
                        },
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(1.0, 0.85, 0.2)),
                        BackgroundColor(Color::srgba(1.0, 0.85, 0.2, 0.15)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("PLAY AGAIN"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.85, 0.2)),
                        ));
                    });

                    // MAIN MENU button
                    row.spawn((
                        VictoryButton {
                            action: VictoryAction::MainMenu,
                        },
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(1.0, 0.85, 0.2)),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("MAIN MENU"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.85, 0.2)),
                        ));
                    });
                });

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Minmatar motto
            parent.spawn((
                Text::new("IN RUST WE TRUST"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            // Controller hint
            parent.spawn((
                Text::new("← → Navigate • A/ENTER Select"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
            ));
        });
}

/// Update victory celebration particles
pub(crate) fn update_victory_particles(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut VictoryParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut particle, mut sprite) in query.iter_mut() {
        // Move particle upward
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Add slight wave motion
        transform.translation.x += (particle.lifetime * 3.0).sin() * 20.0 * dt;

        // Slow down over time
        particle.velocity.y *= 0.995;
        particle.lifetime -= dt;

        // Fade out
        let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(alpha);

        // Reset if off screen or dead
        if particle.lifetime <= 0.0 || transform.translation.y > SCREEN_HEIGHT / 2.0 + 50.0 {
            transform.translation.x = (fastrand::f32() - 0.5) * SCREEN_WIDTH;
            transform.translation.y = -SCREEN_HEIGHT / 2.0 - fastrand::f32() * 50.0;
            particle.lifetime = particle.max_lifetime;
            particle.velocity.y = 80.0 + fastrand::f32() * 120.0;
        }
    }
}

pub(crate) fn victory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<VictorySelection>,
    mut score: ResMut<ScoreSystem>,
    mut campaign: ResMut<CampaignState>,
    mut transitions: EventWriter<TransitionEvent>,
) {
    // Navigation (left/right for button selection)
    if keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
        || joystick.dpad_x < 0
    {
        selection.selected = VictoryAction::PlayAgain;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
        || joystick.dpad_x > 0
    {
        selection.selected = VictoryAction::MainMenu;
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        match selection.selected {
            VictoryAction::PlayAgain => {
                score.reset_game();
                *campaign = CampaignState::default();
                transitions.send(TransitionEvent::to(GameState::ShipSelect));
            }
            VictoryAction::MainMenu => {
                score.reset_game();
                *campaign = CampaignState::default();
                transitions.send(TransitionEvent::slow(GameState::MainMenu));
            }
        }
    }

    // Quick exit to menu
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        score.reset_game();
        *campaign = CampaignState::default();
        transitions.send(TransitionEvent::slow(GameState::MainMenu));
    }
}

pub(crate) fn update_victory_buttons(
    selection: Res<VictorySelection>,
    mut button_query: Query<(&VictoryButton, &mut BorderColor, &mut BackgroundColor)>,
) {
    let gold = Color::srgb(1.0, 0.85, 0.2);
    let gold_bright = Color::srgb(1.0, 0.95, 0.4);

    for (button, mut border, mut bg) in button_query.iter_mut() {
        if button.action == selection.selected {
            border.0 = gold_bright;
            bg.0 = Color::srgba(1.0, 0.85, 0.2, 0.2);
        } else {
            border.0 = gold;
            bg.0 = Color::NONE;
        }
    }
}

pub(crate) fn despawn_victory_screen(mut commands: Commands, query: Query<Entity, With<VictoryRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<VictorySelection>();
}
