//! Wave, mission, and dialogue display systems

use super::common::*;
use crate::core::*;
use crate::systems::DialogueSystem;
use bevy::prelude::*;

/// Update wave display (with stage info)
pub fn update_wave_display(
    campaign: Res<CampaignState>,
    mut query: Query<&mut Text, With<WaveText>>,
) {
    for mut text in query.iter_mut() {
        if let Some(mission) = campaign.current_mission() {
            if mission.timed_survival_seconds > 0.0 {
                let remaining = (mission.timed_survival_seconds - campaign.mission_timer).max(0.0);
                **text = format!("SURVIVE: {:.1}s", remaining);
            } else if campaign.is_boss_wave() {
                **text = format!(
                    "WAVE {}/{} - BOSS",
                    campaign.current_wave,
                    mission.enemy_waves + 1
                );
            } else {
                **text = format!("WAVE {}/{}", campaign.current_wave, mission.enemy_waves + 1);
            }
        } else {
            **text = format!("WAVE {}", campaign.current_wave);
        }
    }
}

/// Update mission info display
pub fn update_mission_display(
    campaign: Res<CampaignState>,
    score: Res<ScoreSystem>,
    mut mission_query: Query<
        &mut Text,
        (
            With<MissionNameText>,
            Without<ObjectiveText>,
            Without<SoulsText>,
        ),
    >,
    mut objective_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<ObjectiveText>,
            Without<MissionNameText>,
            Without<SoulsText>,
        ),
    >,
    mut souls_query: Query<
        &mut Text,
        (
            With<SoulsText>,
            Without<MissionNameText>,
            Without<ObjectiveText>,
            Without<KillCountText>,
        ),
    >,
    mut kill_query: Query<
        &mut Text,
        (
            With<KillCountText>,
            Without<MissionNameText>,
            Without<ObjectiveText>,
            Without<SoulsText>,
        ),
    >,
) {
    // Update mission name
    for mut text in mission_query.iter_mut() {
        if let Some(mission) = campaign.current_mission() {
            **text = format!(
                "M{}: {} - {}",
                campaign.mission_number(),
                mission.name,
                campaign.act.name()
            );
        } else {
            **text = String::new();
        }
    }

    // Update objective
    for (mut text, mut color) in objective_query.iter_mut() {
        if let Some(mission) = campaign.current_mission() {
            if campaign.primary_complete {
                **text = format!("\u{2713} {}", mission.primary_objective);
                color.0 = Color::srgb(0.3, 1.0, 0.3); // Bright green when complete
            } else {
                **text = format!("\u{25ef} {}", mission.primary_objective);
                color.0 = Color::srgb(0.5, 0.8, 0.5); // Dim green when incomplete
            }
        } else {
            **text = String::new();
        }
    }

    // Update souls liberated
    for mut text in souls_query.iter_mut() {
        if campaign.in_mission {
            let bonus = if let Some(mission) = campaign.current_mission() {
                if campaign.mission_souls >= mission.souls_to_liberate {
                    " \u{2713}"
                } else {
                    ""
                }
            } else {
                ""
            };
            **text = format!("SOULS LIBERATED: {}{}", score.souls_liberated, bonus);
        } else {
            **text = String::new();
        }
    }

    // Update enemies killed
    for mut text in kill_query.iter_mut() {
        if campaign.in_mission {
            **text = format!("ENEMIES DEFEATED: {}", campaign.enemies_killed);
        } else {
            **text = String::new();
        }
    }
}

/// Update dialogue display based on DialogueSystem state
pub fn update_dialogue_display(
    dialogue_system: Res<DialogueSystem>,
    mut container_query: Query<&mut Node, With<DialogueContainer>>,
    mut speaker_query: Query<&mut Text, (With<DialogueSpeakerText>, Without<DialogueContentText>)>,
    mut content_query: Query<&mut Text, (With<DialogueContentText>, Without<DialogueSpeakerText>)>,
) {
    let is_active = dialogue_system.is_active();

    // Show/hide dialogue container
    for mut node in container_query.iter_mut() {
        node.display = if is_active {
            Display::Flex
        } else {
            Display::None
        };
    }

    if let Some(text) = &dialogue_system.active_text {
        // Update speaker name
        for mut speaker in speaker_query.iter_mut() {
            **speaker = dialogue_system.speaker.clone();
        }

        // Update dialogue content
        for mut content in content_query.iter_mut() {
            **content = text.clone();
        }
    }
}
