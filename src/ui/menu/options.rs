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
    pub(crate) setting: SliderSetting,
}

#[derive(Component)]
pub(crate) struct VolumeLabel {
    pub(crate) setting: SliderSetting,
}

/// Marker for the CONTROLS navigation row in the Options menu.
/// Routes a confirm-press into `GameState::Controls`.
#[derive(Component)]
pub(crate) struct ControlsNavItem;

/// Marker for the RESET TO DEFAULTS navigation row. A confirm-press
/// restores `SoundSettings`, `ScreenShake.multiplier`, and
/// `RumbleSettings.intensity` to their `Default::default()` values
/// and refreshes every slider bar / label in one pass.
#[derive(Component)]
pub(crate) struct ResetNavItem;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SliderSetting {
    Master,
    Music,
    Sfx,
    Shake,
    Rumble,
}

#[derive(Resource)]
pub(crate) struct OptionsMenuState {
    pub(crate) selected: usize,
    pub(crate) cooldown: f32,
    /// Total items in the menu — used as the wrap modulus for nav.
    /// Layout (7 rows total):
    ///   0 = Master Volume   3 = Screen Shake       5 = RESET
    ///   1 = Music  Volume   4 = Controller Rumble  6 = CONTROLS
    ///   2 = SFX    Volume
    /// If more rows are added later, bump this constant.
    pub(crate) total: usize,
}

impl Default for OptionsMenuState {
    fn default() -> Self {
        Self {
            selected: 0,
            cooldown: 0.0,
            total: 7,
        }
    }
}

pub(crate) fn spawn_options_menu(
    mut commands: Commands,
    sound_settings: Res<crate::systems::audio::SoundSettings>,
    screen_shake: Res<crate::systems::effects::screen_effects::ScreenShake>,
    rumble: Res<crate::systems::joystick::RumbleSettings>,
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
                SliderSetting::Master,
                sound_settings.master_volume,
                0,
            );
            spawn_volume_row(
                parent,
                "Music Volume",
                SliderSetting::Music,
                sound_settings.music_volume,
                1,
            );
            spawn_volume_row(
                parent,
                "SFX Volume",
                SliderSetting::Sfx,
                sound_settings.sfx_volume,
                2,
            );

            // Feedback section header — separates haptics from audio.
            parent.spawn((
                Text::new("FEEDBACK"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.5, 0.55)),
                Node {
                    margin: UiRect::top(Val::Px(10.0))
                        .with_bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            spawn_volume_row(
                parent,
                "Screen Shake",
                SliderSetting::Shake,
                screen_shake.multiplier,
                3,
            );
            spawn_volume_row(
                parent,
                "Controller Rumble",
                SliderSetting::Rumble,
                rumble.intensity,
                4,
            );

            // RESET TO DEFAULTS row (index 5) — restores every slider
            // to its resource's `Default::default()` value.
            parent
                .spawn((
                    ResetNavItem,
                    Node {
                        width: Val::Px(400.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        margin: UiRect::top(Val::Px(10.0))
                            .with_bottom(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.15, 0.1, 0.05, 0.9)),
                    BorderColor(Color::srgba(0.4, 0.3, 0.2, 0.5)),
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new("RESET TO DEFAULTS"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.7, 0.5)),
                    ));
                });

            // CONTROLS nav row (index 6) — opens the controller remapping screen.
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
    setting: SliderSetting,
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
    mut screen_shake: ResMut<crate::systems::effects::screen_effects::ScreenShake>,
    mut rumble: ResMut<crate::systems::joystick::RumbleSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sliders: Query<(&VolumeSlider, &mut BorderColor), Without<VolumeLabel>>,
    mut bars: Query<(&VolumeSlider, &mut Node), (Without<VolumeLabel>, Without<BorderColor>)>,
    mut labels: Query<(&VolumeLabel, &mut Text)>,
    mut reset_nav: Query<&mut BorderColor, (With<ResetNavItem>, Without<ControlsNavItem>)>,
    mut controls_nav: Query<&mut BorderColor, (With<ControlsNavItem>, Without<ResetNavItem>)>,
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

        // Adjust slider value (left/right) — only on the 5 slider rows.
        if state.selected < 5 {
            let adjust = if keyboard.pressed(KeyCode::ArrowLeft) || joystick.dpad_x < 0 {
                -0.05
            } else if keyboard.pressed(KeyCode::ArrowRight) || joystick.dpad_x > 0 {
                0.05
            } else {
                0.0
            };

            if adjust != 0.0 {
                let current_setting = match state.selected {
                    0 => SliderSetting::Master,
                    1 => SliderSetting::Music,
                    2 => SliderSetting::Sfx,
                    3 => SliderSetting::Shake,
                    4 => SliderSetting::Rumble,
                    _ => SliderSetting::Master,
                };

                // Update the setting through its resource. The
                // `is_changed()` flag fires on every `ResMut` deref,
                // which `sync_settings_to_save` watches on the next
                // frame and writes the new value to `SaveData.settings`.
                let new_value = match current_setting {
                    SliderSetting::Master => {
                        sound_settings.master_volume =
                            (sound_settings.master_volume + adjust).clamp(0.0, 1.0);
                        sound_settings.master_volume
                    }
                    SliderSetting::Music => {
                        sound_settings.music_volume =
                            (sound_settings.music_volume + adjust).clamp(0.0, 1.0);
                        sound_settings.music_volume
                    }
                    SliderSetting::Sfx => {
                        sound_settings.sfx_volume =
                            (sound_settings.sfx_volume + adjust).clamp(0.0, 1.0);
                        sound_settings.sfx_volume
                    }
                    SliderSetting::Shake => {
                        screen_shake.multiplier =
                            (screen_shake.multiplier + adjust).clamp(0.0, 1.0);
                        screen_shake.multiplier
                    }
                    SliderSetting::Rumble => {
                        rumble.intensity =
                            (rumble.intensity + adjust).clamp(0.0, 1.0);
                        rumble.intensity
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

    // Confirm on the RESET row restores every slider to its resource's
    // canonical default and refreshes all bar/label visuals in one pass.
    if state.cooldown <= 0.0
        && state.selected == 5
        && is_confirm(&keyboard, &joystick)
    {
        let sound_default = crate::systems::audio::SoundSettings::default();
        sound_settings.master_volume = sound_default.master_volume;
        sound_settings.sfx_volume = sound_default.sfx_volume;
        sound_settings.music_volume = sound_default.music_volume;

        let shake_default = crate::systems::effects::screen_effects::ScreenShake::default();
        // Only the persisted multiplier is reset — runtime intensity /
        // duration / timer are gameplay state, not player preferences.
        screen_shake.multiplier = shake_default.multiplier;

        rumble.intensity = crate::systems::joystick::RumbleSettings::default().intensity;

        // Refresh every bar + label in one pass so the player sees the
        // reset take effect immediately.
        for (slider, mut node) in bars.iter_mut() {
            let v = match slider.setting {
                SliderSetting::Master => sound_settings.master_volume,
                SliderSetting::Music => sound_settings.music_volume,
                SliderSetting::Sfx => sound_settings.sfx_volume,
                SliderSetting::Shake => screen_shake.multiplier,
                SliderSetting::Rumble => rumble.intensity,
            };
            node.width = Val::Percent(v * 100.0);
        }
        for (label, mut text) in labels.iter_mut() {
            let v = match label.setting {
                SliderSetting::Master => sound_settings.master_volume,
                SliderSetting::Music => sound_settings.music_volume,
                SliderSetting::Sfx => sound_settings.sfx_volume,
                SliderSetting::Shake => screen_shake.multiplier,
                SliderSetting::Rumble => rumble.intensity,
            };
            **text = format!("{}%", (v * 100.0) as i32);
        }

        state.cooldown = 0.25;
    }

    // Confirm on the CONTROLS row opens the controller remapping screen.
    if state.cooldown <= 0.0
        && state.selected == 6
        && is_confirm(&keyboard, &joystick)
    {
        next_state.set(GameState::Controls);
        state.cooldown = 0.25;
    }

    // Update selection highlighting across the 5 slider rows.
    for (slider, mut border) in sliders.iter_mut() {
        let is_selected = match slider.setting {
            SliderSetting::Master => state.selected == 0,
            SliderSetting::Music => state.selected == 1,
            SliderSetting::Sfx => state.selected == 2,
            SliderSetting::Shake => state.selected == 3,
            SliderSetting::Rumble => state.selected == 4,
        };
        *border = if is_selected {
            BorderColor(Color::srgb(0.4, 0.6, 0.8))
        } else {
            BorderColor(Color::srgba(0.3, 0.3, 0.4, 0.5))
        };
    }

    // Highlight the RESET row when selected.
    for mut border in reset_nav.iter_mut() {
        *border = if state.selected == 5 {
            BorderColor(Color::srgb(0.4, 0.6, 0.8))
        } else {
            BorderColor(Color::srgba(0.4, 0.3, 0.2, 0.5))
        };
    }

    // Highlight the CONTROLS row when selected.
    for mut border in controls_nav.iter_mut() {
        *border = if state.selected == 6 {
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
