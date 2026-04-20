//! HUD spawning and despawning
//!
//! Layout and construction of the HUD UI tree.

use super::common::*;
use bevy::prelude::*;

pub fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|parent| {
            // === TOP BAR ===
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(80.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|top| {
                    // Left: Score, mission, and wave
                    top.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    })
                    .with_children(|left| {
                        left.spawn((
                            ScoreText,
                            Text::new("SCORE: 0"),
                            TextFont {
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        left.spawn((
                            MissionNameText,
                            Text::new(""),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.8, 0.6, 0.3)), // Rust/amber
                        ));
                        left.spawn((
                            WaveText,
                            Text::new("WAVE 1"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                        left.spawn((
                            ObjectiveText,
                            Text::new(""),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.8, 0.5)), // Green for objectives
                        ));
                        left.spawn((
                            SoulsText,
                            Text::new(""),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.4, 0.7, 1.0)), // Blue for souls
                        ));
                    });

                    // Center: Combo kills and tier with timer bar
                    top.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|center| {
                        center.spawn((
                            ComboKillsText,
                            Text::new(""),
                            TextFont {
                                font_size: 36.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.8, 0.2)),
                        ));
                        // Combo timer bar (hidden when no combo)
                        center
                            .spawn((
                                ComboTimerContainer,
                                Node {
                                    width: Val::Px(120.0),
                                    height: Val::Px(6.0),
                                    display: Display::None, // Hidden initially
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                                BorderRadius::all(Val::Px(2.0)),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    ComboTimerBar,
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(1.0, 0.8, 0.2)),
                                    BorderRadius::all(Val::Px(2.0)),
                                ));
                            });
                    });

                    // Right: Multiplier and Grade
                    top.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::End,
                        ..default()
                    })
                    .with_children(|right| {
                        right.spawn((
                            ComboText,
                            Text::new("x1.0"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.9, 0.3)),
                        ));
                        right.spawn((
                            GradeText,
                            Text::new("D"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.5)),
                        ));
                    });
                });

            // === BOSS HEALTH BAR (hidden by default) ===
            parent
                .spawn((
                    BossHealthContainer,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(10.0)),
                        display: Display::None, // Hidden until boss spawns
                        ..default()
                    },
                ))
                .with_children(|boss_ui| {
                    // Boss name
                    boss_ui.spawn((
                        BossNameText,
                        Text::new("BOSS NAME"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.3, 0.3)),
                    ));
                    // Health bar background
                    boss_ui
                        .spawn((
                            Node {
                                width: Val::Percent(60.0),
                                height: Val::Px(16.0),
                                margin: UiRect::top(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.0, 0.0, 0.8)),
                        ))
                        .with_children(|bar| {
                            bar.spawn((
                                BossHealthFill,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.9, 0.2, 0.2)),
                            ));
                        });
                });

            // === POWERUP STATUS BAR (right side, vertical stack) ===
            parent
                .spawn((
                    PowerupIndicator,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(100.0),
                        right: Val::Px(10.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        align_items: AlignItems::FlexEnd,
                        ..default()
                    },
                ))
                .with_children(|indicators| {
                    // Overdrive status box (cyan)
                    spawn_powerup_status_box(
                        indicators,
                        PowerupType::Overdrive,
                        "OVERDRIVE",
                        Color::srgb(0.3, 0.9, 1.0),
                        5.0, // max duration
                    );
                    // Damage boost status box (red/orange)
                    spawn_powerup_status_box(
                        indicators,
                        PowerupType::DamageBoost,
                        "DAMAGE x2",
                        Color::srgb(1.0, 0.4, 0.2),
                        10.0, // max duration
                    );
                    // Invulnerability status box (gold/white)
                    spawn_powerup_status_box(
                        indicators,
                        PowerupType::Invulnerability,
                        "INVULN",
                        Color::srgb(1.0, 0.9, 0.4),
                        3.0, // max duration
                    );
                });

            // === BOTTOM BAR: Meters only (health is shown in capacitor wheel) ===
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(80.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|bottom| {
                    // Left side: Status meters (Heat, Salt Miner)
                    bottom
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(3.0),
                            align_items: AlignItems::FlexStart,
                            ..default()
                        })
                        .with_children(|left| {
                            // Heat meter (orange/red)
                            spawn_health_bar(left, HeatBar, Color::srgb(1.0, 0.5, 0.0), "HEAT");
                            // Salt Miner meter (purple)
                            spawn_health_bar(
                                left,
                                SaltMinerBar,
                                Color::srgb(0.8, 0.2, 0.8),
                                "SALT MINER",
                            );
                            // Ship ability indicator (blue/cyan)
                            spawn_ability_indicator(left);
                            // Ammo type indicator (for autocannons)
                            spawn_ammo_indicator(left);
                        });

                    // Center: Spacer to push wingman gauge right
                    bottom
                        .spawn((
                            WingmanGauge,
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::FlexEnd,
                                ..default()
                            },
                        ))
                        .with_children(|right| {
                            // Label
                            right.spawn((
                                Text::new("WINGMAN"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.6, 0.3)),
                            ));

                            // Progress bar container
                            right
                                .spawn((
                                    Node {
                                        width: Val::Px(100.0),
                                        height: Val::Px(10.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.15, 0.1, 0.05, 0.9)),
                                    BorderColor(Color::srgb(0.5, 0.35, 0.2)),
                                    BorderRadius::all(Val::Px(2.0)),
                                ))
                                .with_children(|bar| {
                                    bar.spawn((
                                        WingmanGaugeFill,
                                        Node {
                                            width: Val::Percent(0.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.8, 0.5, 0.2)),
                                        BorderRadius::all(Val::Px(2.0)),
                                    ));
                                });

                            // Kill count
                            right.spawn((
                                WingmanCountText,
                                Text::new("0/15"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.6, 0.5, 0.35)),
                            ));

                            // Active wingman icons placeholder
                            right.spawn((
                                Text::new(""),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.7, 0.4)),
                            ));
                        });

                    // Drone status indicator (between wingman gauge and right edge)
                    bottom
                        .spawn((
                            DroneStatusContainer,
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::FlexEnd,
                                margin: UiRect::left(Val::Px(12.0)),
                                display: Display::None,
                                ..default()
                            },
                        ))
                        .with_children(|drone_panel| {
                            drone_panel.spawn((
                                Text::new("DRONES"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.3, 0.7, 0.9)),
                            ));
                            drone_panel.spawn((
                                DroneStatusText,
                                Text::new("0 active"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.4, 0.6, 0.8)),
                            ));
                        });
                });
        });

    // === DIALOGUE BOX (separate from HUD root for positioning) ===
    commands
        .spawn((
            DialogueContainer,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(120.0),
                left: Val::Percent(15.0),
                width: Val::Percent(70.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(15.0)),
                column_gap: Val::Px(15.0),
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|dialogue| {
            // Elder portrait placeholder (rust-colored square)
            dialogue.spawn((
                Node {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.6, 0.35, 0.2)), // Rust/bronze color for Minmatar
                BorderRadius::all(Val::Px(4.0)),
            ));

            // Text container
            dialogue
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(5.0),
                    ..default()
                })
                .with_children(|text_area| {
                    // Speaker name
                    text_area.spawn((
                        DialogueSpeakerText,
                        Text::new("Tribal Elder"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.6, 0.4)), // Rust/amber color
                    ));

                    // Dialogue text
                    text_area.spawn((
                        DialogueContentText,
                        Text::new(""),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.85)),
                    ));
                });
        });

    // === ACHIEVEMENT POPUP (hidden by default) ===
    commands
        .spawn((
            AchievementPopup,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(100.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-150.0)), // Center the 300px wide popup
                width: Val::Px(300.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                row_gap: Val::Px(4.0),
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.1, 0.0, 0.95)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|popup| {
            // "ACHIEVEMENT UNLOCKED" header
            popup.spawn((
                Text::new("ACHIEVEMENT UNLOCKED"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.6, 0.2)),
            ));
            // Achievement name
            popup.spawn((
                AchievementPopupName,
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.3)), // Gold
            ));
            // Achievement description
            popup.spawn((
                AchievementPopupDesc,
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });

    // === BUFF EXPIRATION SCREEN EDGE WARNINGS ===
    spawn_screen_edge_warnings(&mut commands);

    info!("HUD spawned");
}

/// Spawn screen edge warning overlays (hidden by default)
fn spawn_screen_edge_warnings(commands: &mut Commands) {
    // Edge dimensions
    const EDGE_THICKNESS: f32 = 8.0;

    // Top edge
    commands.spawn((
        BuffExpirationWarning {
            edge: ScreenEdge::Top,
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Px(EDGE_THICKNESS),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));

    // Bottom edge
    commands.spawn((
        BuffExpirationWarning {
            edge: ScreenEdge::Bottom,
        },
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Px(EDGE_THICKNESS),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));

    // Left edge
    commands.spawn((
        BuffExpirationWarning {
            edge: ScreenEdge::Left,
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Px(EDGE_THICKNESS),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));

    // Right edge
    commands.spawn((
        BuffExpirationWarning {
            edge: ScreenEdge::Right,
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            right: Val::Px(0.0),
            width: Val::Px(EDGE_THICKNESS),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::NONE),
    ));
}

pub fn spawn_health_bar<M: Component>(
    parent: &mut ChildBuilder,
    marker: M,
    color: Color,
    label: &str,
) {
    parent
        .spawn(Node {
            width: Val::Px(200.0),
            height: Val::Px(12.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(5.0),
            ..default()
        })
        .with_children(|parent| {
            // Label
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(color),
            ));

            // Bar background
            parent
                .spawn((
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                ))
                .with_children(|parent| {
                    // Bar fill
                    parent.spawn((
                        marker,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(color),
                    ));
                });
        });
}

/// Spawn a powerup status box with icon, label, timer bar, and countdown
pub fn spawn_powerup_status_box(
    parent: &mut ChildBuilder,
    powerup_type: PowerupType,
    label: &str,
    color: Color,
    _max_duration: f32,
) {
    // Get the appropriate marker component based on type
    let (marker_overdrive, marker_damage, marker_invuln) = match powerup_type {
        PowerupType::Overdrive => (Some(OverdriveIndicator), None, None),
        PowerupType::DamageBoost => (None, Some(DamageBoostIndicator), None),
        PowerupType::Invulnerability => (None, None, Some(InvulnIndicator)),
    };

    // Main container - hidden by default
    let mut container = parent.spawn((
        PowerupStatusBox { powerup_type },
        Node {
            width: Val::Px(140.0),
            height: Val::Px(36.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(4.0)),
            column_gap: Val::Px(6.0),
            display: Display::None, // Hidden until powerup is active
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
        BorderRadius::all(Val::Px(4.0)),
    ));

    // Add type-specific marker
    if marker_overdrive.is_some() {
        container.insert(OverdriveIndicator);
    }
    if marker_damage.is_some() {
        container.insert(DamageBoostIndicator);
    }
    if marker_invuln.is_some() {
        container.insert(InvulnIndicator);
    }

    container.with_children(|box_parent| {
        // Left: Icon placeholder (colored square)
        box_parent.spawn((
            Node {
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(color),
            BorderRadius::all(Val::Px(3.0)),
        ));

        // Right: Label and timer bar
        box_parent
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                row_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|right| {
                // Label text
                right.spawn((
                    Text::new(label),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(color),
                ));

                // Timer bar background
                right
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(6.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                        BorderRadius::all(Val::Px(2.0)),
                    ))
                    .with_children(|bar| {
                        // Timer bar fill
                        bar.spawn((
                            PowerupTimerBar { powerup_type },
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(color),
                            BorderRadius::all(Val::Px(2.0)),
                        ));
                    });
            });

        // Countdown text (shown when < 2 seconds remaining)
        box_parent.spawn((
            PowerupCountdown { powerup_type },
            Text::new(""),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.3, 0.3)),
            Node {
                display: Display::None, // Hidden until countdown starts
                position_type: PositionType::Absolute,
                right: Val::Px(4.0),
                ..default()
            },
        ));
    });
}

/// Spawn the ability indicator UI
pub fn spawn_ability_indicator(parent: &mut ChildBuilder) {
    // Container with label, key hint, and cooldown bar
    parent
        .spawn((
            AbilityIndicatorContainer,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(2.0),
                align_items: AlignItems::FlexStart,
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            },
        ))
        .with_children(|container| {
            // Top row: ability name + key hint
            container
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // Ability name
                    row.spawn((
                        AbilityIndicatorText,
                        Text::new("ABILITY"),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.8, 1.0)), // Cyan
                    ));

                    // Key hint
                    row.spawn((
                        AbilityKeyHint,
                        Text::new("[RT]"),
                        TextFont {
                            font_size: 9.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                });

            // Cooldown bar container
            container
                .spawn((
                    Node {
                        width: Val::Px(100.0),
                        height: Val::Px(8.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.15, 0.2, 0.9)),
                    BorderColor(Color::srgb(0.2, 0.4, 0.6)),
                    BorderRadius::all(Val::Px(2.0)),
                ))
                .with_children(|bar| {
                    // Fill bar
                    bar.spawn((
                        AbilityIndicatorFill,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.8, 1.0)), // Cyan
                        BorderRadius::all(Val::Px(1.0)),
                    ));
                });
        });
}

/// Spawn the ammo type indicator UI
pub fn spawn_ammo_indicator(parent: &mut ChildBuilder) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        })
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new("AMMO"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
            // Ammo type name (colored by ammo type)
            row.spawn((
                AmmoTypeText,
                Text::new("SABOT"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            // Key hint
            row.spawn((
                Text::new("[D-PAD]"),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

pub fn despawn_hud(
    mut commands: Commands,
    hud_query: Query<Entity, With<HudRoot>>,
    dialogue_query: Query<Entity, With<DialogueContainer>>,
    warning_query: Query<Entity, With<BuffExpirationWarning>>,
    popup_query: Query<Entity, With<AchievementPopup>>,
) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in dialogue_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in warning_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in popup_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
