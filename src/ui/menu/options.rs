//! Options Menu

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::systems::JoystickState;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct OptionsMenuRoot;

#[derive(Component)]
pub(crate) struct VolumeSlider {
    pub(crate) setting: VolumeSetting,
}

#[derive(Component)]
pub(crate) struct VolumeLabel {
    pub(crate) setting: VolumeSetting,
}

/// Marker for the CONTROLS navigation row in the Options menu.
/// Routes a confirm-press into `GameState::Controls`.
#[derive(Component)]
pub(crate) struct ControlsNavItem;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum VolumeSetting {
    Master,
    Music,
    Sfx,
}

#[derive(Resource)]
pub(crate) struct OptionsMenuState {
    pub(crate) selected: usize,
    pub(crate) cooldown: f32,
    /// Total items in the menu — used as the wrap modulus for nav.
    /// Includes the 3 audio sliders (0..=2) plus the CONTROLS nav row
    /// at index 3. Default is 4. If more rows are added later, bump
    /// this constant.
    pub(crate) total: usize,
}

impl Default for OptionsMenuState {
    fn default() -> Self {
        Self {
            selected: 0,
            cooldown: 0.0,
            total: 4,
        }
    }
}

pub(crate) fn spawn_options_menu(
    mut commands: Commands,
    sound_settings: Res<crate::systems::audio::SoundSettings>,
) {
    commands.init_resource::<OptionsMenuState>();

    // Root container
    commands
        .spawn((
            OptionsMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.95)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("OPTIONS"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Audio section header
            parent.spawn((
                Text::new("AUDIO"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Volume sliders
            spawn_volume_row(
                parent,
                "Master Volume",
                VolumeSetting::Master,
                sound_settings.master_volume,
                0,
            );
            spawn_volume_row(
                parent,
                "Music Volume",
                VolumeSetting::Music,
                sound_settings.music_volume,
                1,
            );
            spawn_volume_row(
                parent,
                "SFX Volume",
                VolumeSetting::Sfx,
                sound_settings.sfx_volume,
                2,
            );

            // CONTROLS nav row (index 3) — opens the controller remapping screen.
            parent
                .spawn((
                    ControlsNavItem,
                    Node {
                        width: Val::Px(400.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::all(Val::Px(10.0)),
                        margin: UiRect::bottom(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
                    BorderColor(Color::srgba(0.3, 0.3, 0.4, 0.5)),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new("CONTROLS"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));
                    row.spawn((
                        Text::new(">"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                });

            // Back instruction
            parent.spawn((
                Text::new("D-PAD Select  •  LS Adjust  •  A Confirm  •  B Back"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_volume_row(
    parent: &mut ChildBuilder,
    label: &str,
    setting: VolumeSetting,
    value: f32,
    index: usize,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(400.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
            BorderColor(if index == 0 {
                Color::srgb(0.4, 0.6, 0.8)
            } else {
                Color::srgba(0.3, 0.3, 0.4, 0.5)
            }),
            VolumeSlider { setting },
        ))
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Value + bar container
            row.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },))
                .with_children(|value_row| {
                    // Visual bar background
                    value_row
                        .spawn((
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(12.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                        ))
                        .with_children(|bar_bg| {
                            // Filled portion
                            bar_bg.spawn((
                                VolumeSlider { setting },
                                Node {
                                    width: Val::Percent(value * 100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.6, 0.9)),
                            ));
                        });

                    // Percentage text
                    value_row.spawn((
                        VolumeLabel { setting },
                        Text::new(format!("{}%", (value * 100.0) as i32)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });
        });
}

pub(crate) fn options_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    time: Res<Time>,
    mut state: ResMut<OptionsMenuState>,
    mut sound_settings: ResMut<crate::systems::audio::SoundSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sliders: Query<(&VolumeSlider, &mut BorderColor), Without<VolumeLabel>>,
    mut bars: Query<(&VolumeSlider, &mut Node), (Without<VolumeLabel>, Without<BorderColor>)>,
    mut labels: Query<(&VolumeLabel, &mut Text)>,
    mut controls_nav: Query<&mut BorderColor, With<ControlsNavItem>>,
) {
    let dt = time.delta_secs();
    state.cooldown = (state.cooldown - dt).max(0.0);

    // Navigation (up/down)
    if state.cooldown <= 0.0 {
        let nav = get_nav_input(&keyboard, &joystick);
        if nav != 0 {
            state.selected = (state.selected as i32 + nav).rem_euclid(state.total as i32) as usize;
            state.cooldown = 0.15;
        }

        // Adjust volume (left/right) — only on slider rows.
        if state.selected < 3 {
            let adjust = if keyboard.pressed(KeyCode::ArrowLeft) || joystick.dpad_x < 0 {
                -0.05
            } else if keyboard.pressed(KeyCode::ArrowRight) || joystick.dpad_x > 0 {
                0.05
            } else {
                0.0
            };

            if adjust != 0.0 {
                let current_setting = match state.selected {
                    0 => VolumeSetting::Master,
                    1 => VolumeSetting::Music,
                    2 => VolumeSetting::Sfx,
                    _ => VolumeSetting::Master,
                };

                // Update the setting
                let new_value = match current_setting {
                    VolumeSetting::Master => {
                        sound_settings.master_volume =
                            (sound_settings.master_volume + adjust).clamp(0.0, 1.0);
                        sound_settings.master_volume
                    }
                    VolumeSetting::Music => {
                        sound_settings.music_volume =
                            (sound_settings.music_volume + adjust).clamp(0.0, 1.0);
                        sound_settings.music_volume
                    }
                    VolumeSetting::Sfx => {
                        sound_settings.sfx_volume =
                            (sound_settings.sfx_volume + adjust).clamp(0.0, 1.0);
                        sound_settings.sfx_volume
                    }
                };

                // Update bar width
                for (slider, mut node) in bars.iter_mut() {
                    if slider.setting == current_setting {
                        node.width = Val::Percent(new_value * 100.0);
                    }
                }

                // Update label
                for (label, mut text) in labels.iter_mut() {
                    if label.setting == current_setting {
                        **text = format!("{}%", (new_value * 100.0) as i32);
                    }
                }

                state.cooldown = 0.08;
            }
        }
    }

    // Confirm on the CONTROLS row opens the controller remapping screen.
    if state.cooldown <= 0.0
        && state.selected == 3
        && is_confirm(&keyboard, &joystick)
    {
        next_state.set(GameState::Controls);
        state.cooldown = 0.25;
    }

    // Update selection highlighting
    for (slider, mut border) in sliders.iter_mut() {
        let is_selected = match slider.setting {
            VolumeSetting::Master => state.selected == 0,
            VolumeSetting::Music => state.selected == 1,
            VolumeSetting::Sfx => state.selected == 2,
        };
        *border = if is_selected {
            BorderColor(Color::srgb(0.4, 0.6, 0.8))
        } else {
            BorderColor(Color::srgba(0.3, 0.3, 0.4, 0.5))
        };
    }

    // Highlight the CONTROLS row when selected.
    for mut border in controls_nav.iter_mut() {
        *border = if state.selected == 3 {
            BorderColor(Color::srgb(0.4, 0.6, 0.8))
        } else {
            BorderColor(Color::srgba(0.3, 0.3, 0.4, 0.5))
        };
    }

    // Back to main menu
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
    }
}
