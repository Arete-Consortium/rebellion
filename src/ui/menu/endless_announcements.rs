//! Endless Mode Wave Announcements

#![allow(dead_code)]

use bevy::prelude::*;

/// Marker for endless wave announcement
#[derive(Component)]
pub(crate) struct EndlessWaveAnnouncement {
    pub(crate) timer: f32,
    pub(crate) max_time: f32,
}

/// Marker for endless mini-boss intro
#[derive(Component)]
pub(crate) struct EndlessMiniBossIntro {
    pub(crate) timer: f32,
    pub(crate) max_time: f32,
}

/// Pulse text for announcements
#[derive(Component)]
pub(crate) struct EndlessAnnouncementPulse {
    pub(crate) timer: f32,
}

/// Listen for wave events and spawn announcements (endless mode only)
pub(crate) fn spawn_endless_wave_announcements(
    mut commands: Commands,
    endless: Res<crate::core::EndlessMode>,
    mut wave_events: EventReader<crate::core::events::SpawnWaveEvent>,
) {
    // Only in endless mode
    if !endless.active {
        return;
    }

    for event in wave_events.read() {
        // Show announcement on wave 1, every 5th wave, and every 10th wave
        let wave = event.wave_number;
        if wave == 1 || wave % 5 == 0 {
            commands
                .spawn((
                    EndlessWaveAnnouncement {
                        timer: 0.0,
                        max_time: 1.5,
                    },
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(format!("WAVE {}", wave)),
                        TextFont {
                            font_size: 72.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.2)), // Gold for Elder Fleet
                        EndlessAnnouncementPulse { timer: 0.0 },
                    ));

                    // Escalation warning
                    if wave >= 30 {
                        parent.spawn((
                            Text::new("EXTREME ESCALATION"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.3, 0.3)),
                        ));
                    } else if wave >= 20 {
                        parent.spawn((
                            Text::new("HIGH ESCALATION"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.5, 0.2)),
                        ));
                    } else if wave >= 10 {
                        parent.spawn((
                            Text::new("ESCALATING"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.7, 0.3)),
                        ));
                    }
                });
        }
    }
}

/// Listen for boss spawn events in endless mode
pub(crate) fn spawn_endless_miniboss_announcement(
    mut commands: Commands,
    endless: Res<crate::core::EndlessMode>,
    mut boss_events: EventReader<crate::core::BossSpawnEvent>,
) {
    // Only in endless mode for mini-bosses
    if !endless.active {
        return;
    }

    for _event in boss_events.read() {
        commands
            .spawn((
                EndlessMiniBossIntro {
                    timer: 0.0,
                    max_time: 2.0,
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("⚠ MINI-BOSS ⚠"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.3, 0.3)),
                    EndlessAnnouncementPulse { timer: 0.0 },
                ));

                parent.spawn(Node {
                    height: Val::Px(10.0),
                    ..default()
                });

                parent.spawn((
                    Text::new(format!("Wave {} Champion", endless.wave)),
                    TextFont {
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                ));

                parent.spawn(Node {
                    height: Val::Px(10.0),
                    ..default()
                });

                parent.spawn((
                    Text::new(format!("Escalation: {:.1}x", endless.escalation)),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.6, 0.2)),
                ));
            });
    }
}

/// Update endless wave announcements
pub(crate) fn update_endless_announcements(
    time: Res<Time>,
    mut commands: Commands,
    mut wave_query: Query<(Entity, &mut EndlessWaveAnnouncement, &mut BackgroundColor)>,
    mut boss_query: Query<
        (Entity, &mut EndlessMiniBossIntro, &mut BackgroundColor),
        Without<EndlessWaveAnnouncement>,
    >,
    mut pulse_query: Query<(&mut TextColor, &mut EndlessAnnouncementPulse)>,
) {
    let dt = time.delta_secs();

    // Update wave announcements
    for (entity, mut announcement, mut bg) in wave_query.iter_mut() {
        announcement.timer += dt;

        // Fade in/out
        let progress = announcement.timer / announcement.max_time;
        let alpha = if progress < 0.15 {
            progress / 0.15 * 0.25
        } else if progress > 0.7 {
            (1.0 - progress) / 0.3 * 0.25
        } else {
            0.25
        };
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));

        if announcement.timer >= announcement.max_time {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Update mini-boss intros
    for (entity, mut intro, mut bg) in boss_query.iter_mut() {
        intro.timer += dt;

        // Fade out near end
        let progress = intro.timer / intro.max_time;
        let alpha = if progress > 0.7 {
            (1.0 - progress) / 0.3 * 0.6
        } else {
            0.6
        };
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));

        if intro.timer >= intro.max_time {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Pulse text
    for (mut color, mut pulse) in pulse_query.iter_mut() {
        pulse.timer += dt * 5.0;
        let intensity = (pulse.timer.sin() * 0.2 + 0.8).clamp(0.6, 1.0);
        // Preserve the base color but vary intensity
        let base = color.0;
        *color = TextColor(Color::srgb(
            base.to_srgba().red * intensity,
            base.to_srgba().green * intensity,
            base.to_srgba().blue * intensity,
        ));
    }
}
