//! Score and combo display systems

use super::common::*;
use crate::core::*;
use crate::systems::ComboHeatSystem;
use bevy::prelude::*;

pub fn update_score_display(score: Res<ScoreSystem>, mut query: Query<&mut Text, With<ScoreText>>) {
    for mut text in query.iter_mut() {
        **text = format!("SCORE: {}", score.score);
    }
}

pub fn update_combo_display(
    score: Res<ScoreSystem>,
    mut combo_query: Query<
        (&mut Text, &mut TextColor, &mut TextFont),
        (With<ComboText>, Without<GradeText>),
    >,
    mut grade_query: Query<(&mut Text, &mut TextColor), (With<GradeText>, Without<ComboText>)>,
) {
    for (mut text, mut color, mut font) in combo_query.iter_mut() {
        **text = format!("x{:.1}", score.multiplier);

        // Scale font size with multiplier (base 20, max 40)
        let base_size = 20.0_f32;
        let scale_factor = 2.5_f32;
        font.font_size = (base_size + score.multiplier * scale_factor).min(40.0);

        // Color based on multiplier
        color.0 = if score.multiplier >= 10.0 {
            Color::srgb(1.0, 0.3, 0.3)
        } else if score.multiplier >= 5.0 {
            Color::srgb(1.0, 0.6, 0.2)
        } else if score.multiplier >= 2.0 {
            Color::srgb(1.0, 0.9, 0.3)
        } else {
            Color::WHITE
        };
    }

    for (mut text, mut text_color) in grade_query.iter_mut() {
        let grade = score.get_grade();
        **text = grade.as_str().to_string();
        text_color.0 = grade.color();
    }
}

/// Update combo kills display
pub fn update_combo_kills(
    heat_system: Res<ComboHeatSystem>,
    mut query: Query<&mut Text, With<ComboKillsText>>,
) {
    for mut text in query.iter_mut() {
        if let Some(tier_name) = heat_system.combo_tier_name() {
            **text = format!("{} x{}", tier_name, heat_system.combo_count);
        } else if heat_system.combo_count > 0 {
            **text = format!("{}x", heat_system.combo_count);
        } else {
            **text = String::new();
        }
    }
}

/// Update combo timer bar (shows time remaining to keep combo)
pub fn update_combo_timer_bar(
    heat_system: Res<ComboHeatSystem>,
    mut container_query: Query<&mut Node, With<ComboTimerContainer>>,
    mut fill_query: Query<
        (&mut Node, &mut BackgroundColor),
        (With<ComboTimerBar>, Without<ComboTimerContainer>),
    >,
) {
    let has_combo = heat_system.combo_count > 0;
    let timer_percent = heat_system.combo_timer_percent();

    // Show/hide container
    for mut node in container_query.iter_mut() {
        node.display = if has_combo {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Update fill width and color
    for (mut node, mut bg) in fill_query.iter_mut() {
        node.width = Val::Percent(timer_percent * 100.0);

        // Color changes as timer runs low
        bg.0 = if timer_percent < 0.3 {
            Color::srgb(1.0, 0.3, 0.2) // Red when low
        } else if timer_percent < 0.5 {
            Color::srgb(1.0, 0.6, 0.2) // Orange when getting low
        } else {
            Color::srgb(1.0, 0.8, 0.2) // Gold when healthy
        };
    }
}
