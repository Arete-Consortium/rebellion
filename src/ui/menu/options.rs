//! Options Menu

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::systems::JoystickState;
use bevy::audio::PlaybackMode;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct OptionsMenuRoot;

#[derive(Component)]
pub(crate) struct VolumeSlider {
    pub(crate) setting: SliderSetting,
}

/// Distinguishes the filled bar from the slider row. Both have a `Node`,
/// which automatically includes `BorderColor` in Bevy 0.15.
#[derive(Component)]
pub(crate) struct VolumeSliderFill;

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
                    margin: UiRect::top(Val::Px(10.0)).with_bottom(Val::Px(20.0)),
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
                        margin: UiRect::top(Val::Px(10.0)).with_bottom(Val::Px(10.0)),
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
                                VolumeSliderFill,
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
    mut commands: Commands,
    (keyboard, bindings): (Res<ButtonInput<KeyCode>>, Res<KeyBindings>),
    joystick: Res<JoystickState>,
    time: Res<Time>,
    mut state: ResMut<OptionsMenuState>,
    mut sound_settings: ResMut<crate::systems::audio::SoundSettings>,
    mut screen_shake: ResMut<crate::systems::effects::screen_effects::ScreenShake>,
    mut rumble: ResMut<crate::systems::joystick::RumbleSettings>,
    sounds: Res<crate::systems::audio::SoundAssets>,
    mut rumble_writer: EventWriter<crate::systems::joystick::RumbleRequest>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sliders: Query<
        (&VolumeSlider, &mut BorderColor),
        (
            Without<VolumeLabel>,
            Without<VolumeSliderFill>,
            Without<ResetNavItem>,
            Without<ControlsNavItem>,
        ),
    >,
    mut bars: Query<(&VolumeSlider, &mut Node), (With<VolumeSliderFill>, Without<VolumeLabel>)>,
    mut labels: Query<(&VolumeLabel, &mut Text)>,
    mut reset_nav: Query<&mut BorderColor, (With<ResetNavItem>, Without<ControlsNavItem>)>,
    mut controls_nav: Query<&mut BorderColor, (With<ControlsNavItem>, Without<ResetNavItem>)>,
) {
    let dt = time.delta_secs();
    state.cooldown = (state.cooldown - dt).max(0.0);

    // Navigation (up/down)
    if state.cooldown <= 0.0 {
        // This menu reserves horizontal input for sliders. The shared menu
        // helper also treats left/right as row navigation, so keep this
        // screen's navigation vertical and give it priority over adjustment.
        let nav = get_vertical_input(&keyboard, &joystick, &bindings);
        if nav != 0 {
            state.selected = (state.selected as i32 + nav).rem_euclid(state.total as i32) as usize;
            state.cooldown = 0.15;
        } else if state.selected < 5 {
            // Adjust the selected slider with keys, D-pad or left-stick X.
            let adjust = get_horizontal_input(&keyboard, &joystick, &bindings) as f32 * 0.05;

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
                        rumble.intensity = (rumble.intensity + adjust).clamp(0.0, 1.0);
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

                // Live previews so the player hears/feels the new
                // setting without quitting. Both gated by the 0.08s
                // cooldown so playback never piles up.
                //
                // - SFX: spawn a one-shot menu_select at the new
                //   sfx_volume * master_volume. Master slider
                //   intentionally skipped — previewing the master
                //   would mid-playback alter its own volume (a
                //   feedback loop). The music system updates the
                //   existing menu track live as its sliders change.
                // - Rumble: send a Custom RumbleRequest so the
                //   gamepad pulses at the new intensity.
                if current_setting == SliderSetting::Sfx {
                    if let Some(source) = sounds.menu_select.clone() {
                        commands.spawn((
                            AudioPlayer(source),
                            PlaybackSettings {
                                mode: PlaybackMode::Despawn,
                                volume: bevy::audio::Volume::new(
                                    sound_settings.sfx_volume * sound_settings.master_volume * 0.7,
                                ),
                                ..default()
                            },
                        ));
                    }
                }
                if current_setting == SliderSetting::Rumble {
                    rumble_writer.send(crate::systems::joystick::RumbleRequest::new(
                        crate::systems::joystick::RumbleType::Custom {
                            strong: 0.6,
                            weak: 0.4,
                            duration_ms: 120,
                        },
                    ));
                }

                state.cooldown = 0.08;
            }
        }
    }

    // Confirm on the RESET row restores every slider to its resource's
    // canonical default and refreshes all bar/label visuals in one pass.
    if state.cooldown <= 0.0 && state.selected == 5 && is_confirm(&keyboard, &joystick, &bindings) {
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
    if state.cooldown <= 0.0 && state.selected == 6 && is_confirm(&keyboard, &joystick, &bindings) {
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
    if is_cancel(&keyboard, &joystick, &bindings) {
        next_state.set(GameState::MainMenu);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_builder::build_headless_app;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn native_menu_update_systems_initialize_before_options_is_active() {
        let mut app = build_headless_app();
        app.add_plugins(crate::ui::menu::MenuPlugin);
        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Loading
        );

        // Bevy initializes every system's queries even when its state run
        // condition is false. Initialize the actual native menu schedule to
        // catch startup query conflicts without requiring a renderer/window.
        app.world_mut().schedule_scope(Update, |world, schedule| {
            schedule
                .initialize(world)
                .expect("native menu systems initialize");
        });
    }

    #[test]
    fn reset_refreshes_native_options_bars_labels_and_row_highlights() {
        let mut app = build_headless_app();
        app.init_resource::<crate::systems::audio::SoundAssets>();
        let world = app.world_mut();
        {
            let mut sound = world.resource_mut::<crate::systems::audio::SoundSettings>();
            sound.master_volume = 0.1;
            sound.music_volume = 0.1;
            sound.sfx_volume = 0.1;
        }
        world
            .resource_mut::<crate::systems::ScreenShake>()
            .multiplier = 0.1;
        world
            .resource_mut::<crate::systems::RumbleSettings>()
            .intensity = 0.1;
        world
            .run_system_once(spawn_options_menu)
            .expect("spawn native options widgets");
        world.resource_mut::<OptionsMenuState>().selected = 5;
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        world
            .run_system_once(options_menu_input)
            .expect("reset through native options input");

        let sound = crate::systems::audio::SoundSettings::default();
        let defaults = [
            (SliderSetting::Master, sound.master_volume),
            (SliderSetting::Music, sound.music_volume),
            (SliderSetting::Sfx, sound.sfx_volume),
            (
                SliderSetting::Shake,
                crate::systems::ScreenShake::default().multiplier,
            ),
            (
                SliderSetting::Rumble,
                crate::systems::RumbleSettings::default().intensity,
            ),
        ];
        let mut bars = world.query_filtered::<(&VolumeSlider, &Node), With<VolumeSliderFill>>();
        assert_eq!(bars.iter(world).count(), 5);
        for (setting, value) in defaults {
            let (_, node) = bars
                .iter(world)
                .find(|(slider, _)| slider.setting == setting)
                .expect("filled bar exists for each setting");
            assert_eq!(node.width, Val::Percent(value * 100.0));
            let mut labels = world.query::<(&VolumeLabel, &Text)>();
            let (_, text) = labels
                .iter(world)
                .find(|(label, _)| label.setting == setting)
                .expect("percentage label exists for each setting");
            assert_eq!(text.0, format!("{}%", (value * 100.0) as i32));
        }

        let highlight = Color::srgb(0.4, 0.6, 0.8);
        let mut reset_rows = world.query_filtered::<&BorderColor, With<ResetNavItem>>();
        assert_eq!(reset_rows.single(world).0, highlight);
        let mut controls_rows = world.query_filtered::<&BorderColor, With<ControlsNavItem>>();
        assert_ne!(controls_rows.single(world).0, highlight);
        let mut slider_rows =
            world.query_filtered::<&BorderColor, (With<VolumeSlider>, Without<VolumeSliderFill>)>();
        assert_eq!(slider_rows.iter(world).count(), 5);
        assert!(slider_rows.iter(world).all(|border| border.0 != highlight));
    }

    #[test]
    fn horizontal_input_adjusts_only_the_selected_slider() {
        for (key, dpad_x, stick_x, change) in [
            (Some(KeyCode::ArrowRight), 0, 0.0, 0.05),
            (Some(KeyCode::ArrowLeft), 0, 0.0, -0.05),
            (None, 1, 0.0, 0.05),
            (None, -1, 0.0, -0.05),
            (None, 0, 0.8, 0.05),
            (None, 0, -0.8, -0.05),
        ] {
            let mut app = build_headless_app();
            app.init_resource::<crate::systems::audio::SoundAssets>();
            let world = app.world_mut();
            world
                .run_system_once(spawn_options_menu)
                .expect("spawn options");
            if let Some(key) = key {
                world.resource_mut::<ButtonInput<KeyCode>>().press(key);
            }
            {
                let mut joystick = world.resource_mut::<JoystickState>();
                joystick.dpad_x = dpad_x;
                joystick.left_x = stick_x;
            }

            world
                .run_system_once(options_menu_input)
                .expect("adjust slider");
            assert_eq!(world.resource::<OptionsMenuState>().selected, 0);
            let defaults = crate::systems::audio::SoundSettings::default();
            let sound = world.resource::<crate::systems::audio::SoundSettings>();
            let expected = defaults.master_volume + change;
            assert!((sound.master_volume - expected).abs() < 0.0001);
            assert_eq!(sound.music_volume, defaults.music_volume);
            assert_eq!(sound.sfx_volume, defaults.sfx_volume);
            let mut bars = world.query_filtered::<(&VolumeSlider, &Node), With<VolumeSliderFill>>();
            let (_, master_bar) = bars
                .iter(world)
                .find(|(slider, _)| slider.setting == SliderSetting::Master)
                .expect("master slider fill");
            assert_eq!(master_bar.width, Val::Percent(expected * 100.0));
        }
    }

    #[test]
    fn vertical_navigation_does_not_adjust_the_destination_slider() {
        let mut app = build_headless_app();
        app.init_resource::<crate::systems::audio::SoundAssets>();
        let world = app.world_mut();
        world
            .run_system_once(spawn_options_menu)
            .expect("spawn options");
        {
            let mut keyboard = world.resource_mut::<ButtonInput<KeyCode>>();
            keyboard.press(KeyCode::ArrowDown);
            keyboard.press(KeyCode::ArrowRight);
        }
        world
            .run_system_once(options_menu_input)
            .expect("navigate options");
        assert_eq!(world.resource::<OptionsMenuState>().selected, 1);
        let defaults = crate::systems::audio::SoundSettings::default();
        let sound = world.resource::<crate::systems::audio::SoundSettings>();
        assert_eq!(sound.master_volume, defaults.master_volume);
        assert_eq!(sound.music_volume, defaults.music_volume);
    }
}
