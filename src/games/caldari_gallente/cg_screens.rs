//! CG Stage Complete & Victory Screens
//!
//! UI screens shown after mission completion and campaign victory.

use super::campaign::{CGCampaignState, VerticalSliceMode};
use crate::core::{Faction, GameSession, GameState};
use crate::systems::JoystickState;
use bevy::prelude::*;

// ============================================================================
// CG Stage Complete Screen
// ============================================================================

/// Marker for CG stage complete screen
#[derive(Component)]
pub struct CGStageCompleteRoot;

pub fn spawn_cg_stage_complete(
    mut commands: Commands,
    cg_campaign: Res<CGCampaignState>,
    score: Res<crate::core::ScoreSystem>,
    session: Res<GameSession>,
) {
    let mission_name = cg_campaign
        .current_mission()
        .map(|m| m.name)
        .unwrap_or("MISSION");

    // Determine faction color based on player faction
    let faction_color = match session.player_faction {
        Faction::Caldari => Color::srgb(0.2, 0.6, 1.0), // Caldari blue
        Faction::Gallente => Color::srgb(0.3, 0.9, 0.4), // Gallente green
        _ => Color::WHITE,
    };

    // Check if T3 was just unlocked
    let t3_just_unlocked = cg_campaign
        .current_mission()
        .map(|m| m.unlocks_t3)
        .unwrap_or(false);

    commands
        .spawn((
            CGStageCompleteRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.08, 0.95)),
        ))
        .with_children(|parent| {
            // Victory header
            parent.spawn((
                Text::new("MISSION COMPLETE"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
            ));

            parent.spawn((
                Text::new(mission_name),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Stats
            parent.spawn((
                Text::new(format!("Score: {}", score.score)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(format!("Best Chain: {}x", score.chain)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // T3 unlock notification
            if t3_just_unlocked {
                parent.spawn(Node {
                    height: Val::Px(10.0),
                    ..default()
                });

                parent.spawn((
                    Text::new("★ TACTICAL DESTROYERS UNLOCKED ★"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.9, 0.2)),
                ));
            }

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Continue prompt
            parent.spawn((
                Text::new("Press A to continue"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Controller hint
            parent.spawn((
                Text::new("A Continue  •  B Main Menu"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

pub fn cg_stage_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut cg_campaign: ResMut<CGCampaignState>,
    slice_mode: Res<VerticalSliceMode>,
    mut transitions: EventWriter<crate::ui::TransitionEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        // Advance to next mission (returns false if campaign/slice complete)
        if cg_campaign.complete_mission(*slice_mode) {
            // More missions available
            transitions.send(crate::ui::TransitionEvent::to(GameState::Playing));
        } else if slice_mode.is_slice() {
            // Vertical slice complete — show slice end screen
            transitions.send(crate::ui::TransitionEvent::slow(
                GameState::SliceComplete,
            ));
        } else {
            // Full campaign complete!
            transitions.send(crate::ui::TransitionEvent::slow(GameState::Victory));
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        transitions.send(crate::ui::TransitionEvent::to(GameState::MainMenu));
    }
}

pub fn despawn_cg_stage_complete(
    mut commands: Commands,
    query: Query<Entity, With<CGStageCompleteRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ============================================================================
// CG Victory Screen (Campaign Complete)
// ============================================================================

/// Marker for CG victory screen
#[derive(Component)]
pub struct CGVictoryRoot;

/// Marker for CG victory particles
#[derive(Component)]
pub struct CGVictoryParticle {
    velocity: Vec2,
    lifetime: f32,
    max_lifetime: f32,
}

pub fn spawn_cg_victory_screen(
    mut commands: Commands,
    score: Res<crate::core::ScoreSystem>,
    session: Res<GameSession>,
    cg_campaign: Res<CGCampaignState>,
    mut save_data: ResMut<crate::core::SaveData>,
) {
    // Determine faction-specific content
    let (header, subtitle, quote, author, motto, particle_color1, particle_color2) =
        match session.player_faction {
            Faction::Caldari => (
                "CALDARI PRIME SECURED",
                "The State Stands Victorious",
                "\"The Caldari way is the only way.\"",
                "— Caldari Navy Command",
                "FOR THE STATE",
                Color::srgb(0.2, 0.6, 1.0), // Caldari blue
                Color::srgb(0.4, 0.8, 0.9), // Light cyan
            ),
            Faction::Gallente => (
                "CALDARI PRIME LIBERATED",
                "Freedom Prevails",
                "\"Liberty must be defended, at any cost.\"",
                "— Federation High Command",
                "LIBERTÉ POUR TOUS",
                Color::srgb(0.3, 0.9, 0.4), // Gallente green
                Color::srgb(0.5, 0.8, 0.3), // Olive
            ),
            _ => (
                "CAMPAIGN COMPLETE",
                "Victory Achieved",
                "\"Well fought.\"",
                "— Command",
                "VICTORY",
                Color::WHITE,
                Color::srgb(0.8, 0.8, 0.8),
            ),
        };

    // Check for new high score
    let faction_key = format!("cg_{}", session.player_faction.short_name());
    let enemy_key = format!("cg_{}", session.enemy_faction.short_name());
    let previous_high = save_data.get_high_score(&faction_key, &enemy_key);
    let is_new_high_score = score.score > previous_high;

    if is_new_high_score {
        save_data.record_score(&faction_key, &enemy_key, score.score, 5);
    }

    // Spawn celebration particles
    for _ in 0..60 {
        let x = (fastrand::f32() - 0.5) * crate::core::SCREEN_WIDTH;
        let y = -crate::core::SCREEN_HEIGHT / 2.0 - fastrand::f32() * 100.0;
        let vx = (fastrand::f32() - 0.5) * 100.0;
        let vy = 80.0 + fastrand::f32() * 120.0;
        let size = 4.0 + fastrand::f32() * 8.0;
        let lifetime = 3.0 + fastrand::f32() * 4.0;

        let color = if fastrand::bool() {
            particle_color1
        } else {
            particle_color2
        };

        commands.spawn((
            CGVictoryRoot,
            CGVictoryParticle {
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
            CGVictoryRoot,
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
                Text::new(header),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(particle_color1),
            ));

            parent.spawn((
                Text::new(subtitle),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(particle_color2),
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
                    BorderColor(particle_color1),
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.8)),
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
                        Text::new(format!("FINAL SCORE: {}", score.score)),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                    ));

                    if !is_new_high_score && previous_high > 0 {
                        stats.spawn((
                            Text::new(format!("High Score: {}", previous_high)),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                    }

                    stats.spawn((
                        Text::new(format!("Max Multiplier: {:.1}x", score.multiplier)),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));

                    stats.spawn((
                        Text::new(format!(
                            "Missions Completed: {}/5",
                            cg_campaign.mission_index + 1
                        )),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(particle_color2),
                    ));
                });

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Quote
            parent.spawn((
                Text::new(quote),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
            ));

            parent.spawn((
                Text::new(author),
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
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(particle_color1),
                        BackgroundColor(Color::srgba(0.2, 0.6, 1.0, 0.15)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("PLAY AGAIN"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(particle_color1),
                        ));
                    });

                    // MAIN MENU button
                    row.spawn((
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(particle_color1),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("MAIN MENU"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(particle_color1),
                        ));
                    });
                });

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Faction motto
            parent.spawn((
                Text::new(motto),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(particle_color2.with_alpha(0.7)),
            ));
        });
}

pub fn update_cg_victory_particles(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CGVictoryParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime -= dt;
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Fade out
        let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

pub fn cg_victory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut cg_campaign: ResMut<CGCampaignState>,
    mut score: ResMut<crate::core::ScoreSystem>,
    mut transitions: EventWriter<crate::ui::TransitionEvent>,
) {
    // Left/Right to select button (simplified - just accept any input)
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        // Reset and play again
        *cg_campaign = CGCampaignState::default();
        score.reset_game();
        transitions.send(crate::ui::TransitionEvent::to(GameState::FactionSelect));
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        *cg_campaign = CGCampaignState::default();
        transitions.send(crate::ui::TransitionEvent::to(GameState::MainMenu));
    }
}

pub fn despawn_cg_victory(mut commands: Commands, query: Query<Entity, With<CGVictoryRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ============================================================================
// Vertical Slice Complete Screen
// ============================================================================

/// Marker for vertical slice complete screen
#[derive(Component)]
pub struct CGSliceCompleteRoot;

/// Spawn "Vertical Slice Complete" screen shown after Mission 3 in slice mode
pub fn spawn_cg_slice_complete(
    mut commands: Commands,
    score: Res<crate::core::ScoreSystem>,
    session: Res<GameSession>,
    cg_campaign: Res<CGCampaignState>,
) {
    let faction_color = match session.player_faction {
        Faction::Caldari => Color::srgb(0.2, 0.6, 1.0),
        Faction::Gallente => Color::srgb(0.3, 0.9, 0.4),
        _ => Color::WHITE,
    };

    let mission_name = cg_campaign
        .current_mission()
        .map(|m| m.name)
        .unwrap_or("MISSION 3");

    commands
        .spawn((
            CGSliceCompleteRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.08, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("VERTICAL SLICE COMPLETE"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
            ));

            parent.spawn((
                Text::new(format!("{} — Victory", mission_name)),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            parent.spawn((
                Text::new(format!("Score: {}", score.score)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(format!("Best Chain: {}x", score.chain)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            parent.spawn((
                Text::new("Thank you for playing the Rebellion demo."),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            parent.spawn((
                Text::new("The full campaign is coming soon."),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            parent.spawn(Node {
                height: Val::Px(40.0),
                ..default()
            });

            parent.spawn((
                Text::new("Press SPACE or ENTER for Main Menu"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

/// Handle input on slice complete screen
pub fn cg_slice_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut cg_campaign: ResMut<CGCampaignState>,
    mut transitions: EventWriter<crate::ui::TransitionEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
        || keyboard.just_pressed(KeyCode::Escape)
        || joystick.back()
    {
        *cg_campaign = CGCampaignState::default();
        transitions.send(crate::ui::TransitionEvent::to(GameState::MainMenu));
    }
}

/// Despawn slice complete screen
pub fn despawn_cg_slice_complete(
    mut commands: Commands,
    query: Query<Entity, With<CGSliceCompleteRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
