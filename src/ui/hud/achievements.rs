//! Achievement popup display system

use super::common::*;
use crate::core::*;
use bevy::prelude::*;

/// Update achievement popup display
pub fn update_achievement_popup(
    mut popup_state: ResMut<AchievementPopupState>,
    time: Res<Time>,
    mut popup_query: Query<&mut Node, With<AchievementPopup>>,
    mut name_query: Query<
        (&mut Text, &mut TextColor),
        (With<AchievementPopupName>, Without<AchievementPopupDesc>),
    >,
    mut desc_query: Query<&mut Text, (With<AchievementPopupDesc>, Without<AchievementPopupName>)>,
) {
    let dt = time.delta_secs();

    // Update timer if showing an achievement
    if popup_state.current.is_some() {
        popup_state.timer -= dt;
        if popup_state.timer <= 0.0 {
            // Hide current popup
            popup_state.current = None;
            if let Ok(mut node) = popup_query.get_single_mut() {
                node.display = Display::None;
            }
        }
    }

    // Show next queued achievement if not currently showing one
    if popup_state.current.is_none() && !popup_state.queue.is_empty() {
        let achievement = popup_state.queue.remove(0);
        popup_state.current = Some(achievement);
        popup_state.timer = AchievementPopupState::DISPLAY_TIME;

        // Update popup content
        if let Ok((mut name_text, mut name_color)) = name_query.get_single_mut() {
            **name_text = achievement.name().to_string();
            name_color.0 = achievement.color();
        }
        if let Ok(mut desc_text) = desc_query.get_single_mut() {
            **desc_text = achievement.description().to_string();
        }

        // Show popup
        if let Ok(mut node) = popup_query.get_single_mut() {
            node.display = Display::Flex;
        }
    }
}
