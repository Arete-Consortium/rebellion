//! Boss Intro Screen

#![allow(dead_code)]

use bevy::prelude::*;
use crate::core::*;
use crate::entities::boss::get_boss_for_stage;

#[derive(Component)]
pub(crate) struct BossIntroRoot;

/// Boss intro text that pulses
#[derive(Component)]
pub(crate) struct BossIntroWarning {
    pub(crate) timer: f32,
}

/// Boss intro name that fades in
#[derive(Component)]
pub(crate) struct BossIntroName {
    pub(crate) timer: f32,
}

/// Boss intro dialogue that types in
#[derive(Component)]
pub(crate) struct BossIntroDialogue {
    pub(crate) full_text: String,
    pub(crate) timer: f32,
}

pub(crate) fn spawn_boss_intro(mut commands: Commands, campaign: Res<CampaignState>) {
    // Get boss data for dialogue and phase info
    let stage = (campaign.mission_index + 1) as u32;
    let boss_data = get_boss_for_stage(stage);

    let (boss_name, boss_title, dialogue, phases) = if let Some(data) = &boss_data {
        (
            data.name.as_str(),
            data.title.as_str(),
            data.dialogue_intro.clone(),
            data.total_phases,
        )
    } else if let Some(mission) = campaign.current_mission() {
        (
            mission.boss.name(),
            mission.name,
            "Prepare for battle...".to_string(),
            2,
        )
    } else {
        ("UNKNOWN", "???", "Prepare for battle...".to_string(), 1)
    };

    // Phase difficulty indicator
    let phase_text = match phases {
        1 => "Single Phase",
        2 => "Two Phases",
        3 => "Three Phases • Challenging",
        4 => "Four Phases • Dangerous",
        5 => "Five Phases • EXTREME",
        _ => "Multi-Phase",
    };

    commands
        .spawn((
            BossIntroRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        ))
        .with_children(|parent| {
            // Warning text (pulses)
            parent.spawn((
                Text::new("⚠ WARNING ⚠"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                BossIntroWarning { timer: 0.0 },
            ));

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Boss name (fades in)
            parent.spawn((
                Text::new(boss_name),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.85, 0.2, 0.0)), // Start transparent
                BossIntroName { timer: 0.0 },
            ));

            // Boss title
            parent.spawn((
                Text::new(boss_title),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.7, 0.3)),
            ));

            // Phase indicator
            parent.spawn((
                Text::new(phase_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if phases >= 4 {
                    Color::srgb(1.0, 0.4, 0.4) // Red for dangerous
                } else if phases >= 3 {
                    Color::srgb(1.0, 0.7, 0.3) // Orange for challenging
                } else {
                    Color::srgb(0.6, 0.6, 0.6) // Gray for normal
                }),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Boss dialogue (types in)
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                BossIntroDialogue {
                    full_text: format!("\"{}\"", dialogue),
                    timer: 0.0,
                },
            ));
        });
}

pub(crate) fn boss_intro_update(
    time: Res<Time>,
    mut warning_query: Query<(&mut TextColor, &mut BossIntroWarning)>,
    mut name_query: Query<(&mut TextColor, &mut BossIntroName), Without<BossIntroWarning>>,
    mut dialogue_query: Query<(&mut Text, &mut BossIntroDialogue)>,
) {
    let dt = time.delta_secs();

    // Pulse warning text
    for (mut color, mut warning) in warning_query.iter_mut() {
        warning.timer += dt * 4.0;
        let pulse = (warning.timer.sin() * 0.3 + 0.7).clamp(0.4, 1.0);
        *color = TextColor(Color::srgb(1.0, 0.2 * pulse, 0.2 * pulse));
    }

    // Fade in boss name
    for (mut color, mut name) in name_query.iter_mut() {
        name.timer += dt * 2.0;
        let alpha = (name.timer - 0.3).clamp(0.0, 1.0); // Delay 0.3s then fade in
        *color = TextColor(Color::srgba(1.0, 0.85, 0.2, alpha));
    }

    // Type in dialogue
    for (mut text, mut dialogue) in dialogue_query.iter_mut() {
        dialogue.timer += dt;
        let chars_to_show = ((dialogue.timer - 0.5) * 30.0) as usize; // 30 chars/sec, 0.5s delay
        let chars_to_show = chars_to_show.min(dialogue.full_text.len());
        if chars_to_show > 0 {
            **text = dialogue.full_text[..chars_to_show].to_string();
        }
    }
}
