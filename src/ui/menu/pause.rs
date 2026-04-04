//! Pause Menu

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::systems::JoystickState;
use crate::ui::TransitionEvent;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct PauseMenuRoot;

/// Pause menu selection state
#[derive(Resource, Default)]
pub(crate) struct PauseSelection {
    pub(crate) index: usize,
}

/// Pause menu items
const PAUSE_ITEM_COUNT: usize = 8;
const PAUSE_IDX_RESUME: usize = 0;
const PAUSE_IDX_MASTER: usize = 1;
const PAUSE_IDX_MUSIC: usize = 2;
const PAUSE_IDX_SFX: usize = 3;
const PAUSE_IDX_SHAKE: usize = 4;
const PAUSE_IDX_RUMBLE: usize = 5;
const PAUSE_IDX_RESTART: usize = 6;
const PAUSE_IDX_QUIT: usize = 7;

/// Slider type for identifying which setting to adjust
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum SliderType {
    MasterVolume,
    MusicVolume,
    SfxVolume,
    ScreenShake,
    Rumble,
}

/// Marker for slider bar fill
#[derive(Component)]
pub(crate) struct SliderFill {
    pub(crate) slider_type: SliderType,
}

/// Marker for slider value text
#[derive(Component)]
pub(crate) struct SliderValueText {
    pub(crate) slider_type: SliderType,
}

#[derive(Component)]
pub(crate) struct PauseMenuItem(pub(crate) usize);

#[derive(Component)]
pub(crate) struct PauseMenuItemText(pub(crate) usize);

pub(crate) fn spawn_pause_menu(
    mut commands: Commands,
    campaign: Res<CampaignState>,
    score: Res<ScoreSystem>,
    session: Res<GameSession>,
    sound_settings: Res<crate::systems::SoundSettings>,
    screen_shake: Res<crate::systems::ScreenShake>,
    rumble_settings: Res<crate::systems::RumbleSettings>,
) {
    commands.insert_resource(PauseSelection::default());

    let mission_name = campaign
        .current_mission()
        .map(|m| m.name)
        .unwrap_or("MISSION");

    let faction_color = session.player_faction.primary_color();

    commands
        .spawn((
            PauseMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.05, 0.85)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            // Mission info
            parent.spawn((
                Text::new(mission_name),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            // Current stats
            parent.spawn((
                Text::new(format!(
                    "Score: {} • Souls: {}",
                    score.score, campaign.mission_souls
                )),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.6, 0.8)),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(12.0),
                ..default()
            });

            // Resume button
            spawn_pause_menu_item(parent, PAUSE_IDX_RESUME, "RESUME");

            // Audio sliders section
            parent.spawn(Node {
                height: Val::Px(4.0),
                ..default()
            });

            // Master volume slider
            spawn_settings_slider(
                parent,
                PAUSE_IDX_MASTER,
                "MASTER",
                sound_settings.master_volume,
                SliderType::MasterVolume,
            );

            // Music volume slider
            spawn_settings_slider(
                parent,
                PAUSE_IDX_MUSIC,
                "MUSIC",
                sound_settings.music_volume,
                SliderType::MusicVolume,
            );

            // SFX volume slider
            spawn_settings_slider(
                parent,
                PAUSE_IDX_SFX,
                "SFX",
                sound_settings.sfx_volume,
                SliderType::SfxVolume,
            );

            // Screen shake slider
            spawn_settings_slider(
                parent,
                PAUSE_IDX_SHAKE,
                "SHAKE",
                screen_shake.multiplier,
                SliderType::ScreenShake,
            );

            // Rumble slider
            spawn_settings_slider(
                parent,
                PAUSE_IDX_RUMBLE,
                "RUMBLE",
                rumble_settings.intensity,
                SliderType::Rumble,
            );

            parent.spawn(Node {
                height: Val::Px(4.0),
                ..default()
            });

            // Restart button
            spawn_pause_menu_item(parent, PAUSE_IDX_RESTART, "RESTART MISSION");

            // Quit button
            spawn_pause_menu_item(parent, PAUSE_IDX_QUIT, "QUIT TO MENU");

            // Spacer
            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Controls hint
            parent.spawn((
                Text::new("↑↓ Navigate • ←→ Adjust • A/ENTER Select"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.55, 0.55, 0.55)),
            ));
        });
}

/// Spawn a simple pause menu button item
fn spawn_pause_menu_item(parent: &mut ChildBuilder, index: usize, label: &str) {
    parent
        .spawn((
            PauseMenuItem(index),
            Node {
                padding: UiRect::axes(Val::Px(25.0), Val::Px(8.0)),
                min_width: Val::Px(260.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        ))
        .with_children(|btn| {
            btn.spawn((
                PauseMenuItemText(index),
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

/// Spawn a settings slider row
fn spawn_settings_slider(
    parent: &mut ChildBuilder,
    index: usize,
    label: &str,
    value: f32,
    slider_type: SliderType,
) {
    parent
        .spawn((
            PauseMenuItem(index),
            Node {
                padding: UiRect::axes(Val::Px(15.0), Val::Px(6.0)),
                min_width: Val::Px(260.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        ))
        .with_children(|row| {
            // Label
            row.spawn((
                PauseMenuItemText(index),
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Slider container
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|slider_row| {
                // Slider background bar
                slider_row
                    .spawn(Node {
                        width: Val::Px(100.0),
                        height: Val::Px(10.0),
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgb(0.15, 0.15, 0.15)))
                    .with_children(|bar| {
                        // Slider fill
                        bar.spawn((
                            SliderFill { slider_type },
                            Node {
                                width: Val::Percent(value * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.6, 0.8)),
                        ));
                    });

                // Value text
                slider_row.spawn((
                    SliderValueText { slider_type },
                    Text::new(format!("{}%", (value * 100.0) as i32)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });
        });
}

pub(crate) fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<PauseSelection>,
    mut next_state: ResMut<NextState<GameState>>,
    mut transitions: EventWriter<TransitionEvent>,
    mut sound_settings: ResMut<crate::systems::SoundSettings>,
    mut screen_shake: ResMut<crate::systems::ScreenShake>,
    mut rumble_settings: ResMut<crate::systems::RumbleSettings>,
    mut item_query: Query<(&PauseMenuItem, &mut BackgroundColor)>,
    mut text_query: Query<(&PauseMenuItemText, &mut TextColor)>,
    mut slider_fill_query: Query<(&SliderFill, &mut Node)>,
    mut slider_text_query: Query<(&SliderValueText, &mut Text)>,
    time: Res<Time>,
    mut cooldown: Local<f32>,
) {
    *cooldown -= time.delta_secs();

    // Navigation (up/down)
    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && *cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(PAUSE_ITEM_COUNT as i32) as usize;
        *cooldown = MENU_NAV_COOLDOWN;
    }

    // Horizontal input for sliders (left/right)
    let h_input = get_horizontal_input(&keyboard, &joystick);
    if h_input != 0 && *cooldown <= 0.0 {
        let delta = h_input as f32 * 0.05; // 5% per press

        match selection.index {
            PAUSE_IDX_MASTER => {
                sound_settings.master_volume =
                    (sound_settings.master_volume + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            PAUSE_IDX_MUSIC => {
                sound_settings.music_volume = (sound_settings.music_volume + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            PAUSE_IDX_SFX => {
                sound_settings.sfx_volume = (sound_settings.sfx_volume + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            PAUSE_IDX_SHAKE => {
                screen_shake.multiplier = (screen_shake.multiplier + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            PAUSE_IDX_RUMBLE => {
                rumble_settings.intensity = (rumble_settings.intensity + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            _ => {}
        }
    }

    // Update slider visuals
    for (fill, mut node) in slider_fill_query.iter_mut() {
        let value = match fill.slider_type {
            SliderType::MasterVolume => sound_settings.master_volume,
            SliderType::MusicVolume => sound_settings.music_volume,
            SliderType::SfxVolume => sound_settings.sfx_volume,
            SliderType::ScreenShake => screen_shake.multiplier,
            SliderType::Rumble => rumble_settings.intensity,
        };
        node.width = Val::Percent(value * 100.0);
    }
    for (text_marker, mut text) in slider_text_query.iter_mut() {
        let value = match text_marker.slider_type {
            SliderType::MasterVolume => sound_settings.master_volume,
            SliderType::MusicVolume => sound_settings.music_volume,
            SliderType::SfxVolume => sound_settings.sfx_volume,
            SliderType::ScreenShake => screen_shake.multiplier,
            SliderType::Rumble => rumble_settings.intensity,
        };
        **text = format!("{}%", (value * 100.0) as i32);
    }

    // Update visual selection
    let session_faction_color = Color::srgb(0.8, 0.5, 0.2); // Default orange
    for (item, mut bg) in item_query.iter_mut() {
        if item.0 == selection.index {
            bg.0 = Color::srgba(0.3, 0.25, 0.15, 0.9);
        } else {
            bg.0 = Color::srgba(0.1, 0.1, 0.1, 0.8);
        }
    }
    for (item, mut color) in text_query.iter_mut() {
        if item.0 == selection.index {
            color.0 = session_faction_color;
        } else {
            color.0 = Color::srgb(0.6, 0.6, 0.6);
        }
    }

    // Selection (confirm button)
    if is_confirm(&keyboard, &joystick) {
        match selection.index {
            PAUSE_IDX_RESUME => {
                next_state.set(GameState::Playing);
            }
            PAUSE_IDX_RESTART => {
                transitions.send(TransitionEvent::quick(GameState::Playing));
            }
            PAUSE_IDX_QUIT => {
                transitions.send(TransitionEvent::to(GameState::MainMenu));
            }
            PAUSE_IDX_MASTER | PAUSE_IDX_MUSIC | PAUSE_IDX_SFX | PAUSE_IDX_SHAKE
            | PAUSE_IDX_RUMBLE => {
                // Pressing confirm on sliders does nothing (use left/right)
            }
            _ => {}
        }
    }

    // Quick resume with ESC or Start
    if keyboard.just_pressed(KeyCode::Escape) || joystick.start() {
        next_state.set(GameState::Playing);
    }
}
