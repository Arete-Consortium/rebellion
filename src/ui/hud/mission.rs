//! Wave, mission, and dialogue display systems

use super::common::*;
use crate::core::*;
use crate::entities::{EscortData, Friendly};
use crate::games::caldari_gallente::CGCampaignState;
use crate::games::ActiveModule;
use crate::systems::DialogueSystem;
use bevy::prelude::*;

/// Update wave display (with stage info)
pub fn update_wave_display(
    active_module: Res<ActiveModule>,
    campaign: Res<CampaignState>,
    cg_campaign: Option<Res<CGCampaignState>>,
    mut query: Query<&mut Text, With<WaveText>>,
) {
    for mut text in query.iter_mut() {
        if active_module.is_caldari_gallente() {
            if let Some(cg) = cg_campaign.as_deref() {
                if let Some(mission) = cg.current_mission() {
                    if cg.boss_spawned && !cg.boss_defeated {
                        **text = format!("WAVE {}/{} — BOSS", cg.current_wave, mission.waves + 1);
                    } else {
                        **text = format!("WAVE {}/{}", cg.current_wave, mission.waves + 1);
                    }
                } else {
                    **text = format!("WAVE {}", cg.current_wave);
                }
            } else {
                **text = "WAVE 1".to_string();
            }
            continue;
        }

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
    active_module: Res<ActiveModule>,
    campaign: Res<CampaignState>,
    cg_campaign: Option<Res<CGCampaignState>>,
    score: Res<ScoreSystem>,
    escort_query: Query<&EscortData, With<Friendly>>,
    mut mission_query: Query<
        &mut Text,
        (
            With<MissionNameText>,
            Without<ObjectiveText>,
            Without<SoulsText>,
            Without<EscortText>,
        ),
    >,
    mut objective_query: Query<
        (&mut Text, &mut TextColor),
        (
            With<ObjectiveText>,
            Without<MissionNameText>,
            Without<SoulsText>,
            Without<EscortText>,
        ),
    >,
    mut souls_query: Query<
        &mut Text,
        (
            With<SoulsText>,
            Without<MissionNameText>,
            Without<ObjectiveText>,
            Without<KillCountText>,
            Without<EscortText>,
        ),
    >,
    mut kill_query: Query<
        &mut Text,
        (
            With<KillCountText>,
            Without<MissionNameText>,
            Without<ObjectiveText>,
            Without<SoulsText>,
            Without<EscortText>,
        ),
    >,
    mut escort_query_text: Query<
        (&mut Text, &mut TextColor),
        (
            With<EscortText>,
            Without<MissionNameText>,
            Without<ObjectiveText>,
            Without<SoulsText>,
            Without<KillCountText>,
        ),
    >,
) {
    // Update mission name
    for mut text in mission_query.iter_mut() {
        if active_module.is_caldari_gallente() {
            if let Some(cg) = cg_campaign.as_deref() {
                if let Some(mission) = cg.current_mission() {
                    **text = format!("M{}: {}", cg.mission_number(), mission.name);
                } else {
                    **text = "ARCHIVE 01: CALDARI PRIME".to_string();
                }
            } else {
                **text = "ARCHIVE 01: CALDARI PRIME".to_string();
            }
            continue;
        }

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
            if let Some(mission) = campaign.current_mission() {
                if mission.kill_count_target > 0 {
                    let check = if campaign.enemies_killed >= mission.kill_count_target {
                        " \u{2713}"
                    } else {
                        ""
                    };
                    **text = format!(
                        "KILLS: {}/{}{}",
                        campaign.enemies_killed, mission.kill_count_target, check
                    );
                } else {
                    **text = format!("ENEMIES DEFEATED: {}", campaign.enemies_killed);
                }
            } else {
                **text = String::new();
            }
        } else {
            **text = String::new();
        }
    }

    // Update escort status
    for (mut text, mut color) in escort_query_text.iter_mut() {
        if campaign.in_mission {
            if let Some(mission) = campaign.current_mission() {
                if mission.escort_must_survive {
                    if let Ok(escort) = escort_query.get_single() {
                        let hp_pct = (escort.health_fraction() * 100.0) as u32;
                        if escort.health <= 0.0 {
                            **text = "ESCORT: DESTROYED".to_string();
                            color.0 = Color::srgb(1.0, 0.2, 0.2); // Red
                        } else {
                            **text = format!("ESCORT: {}%", hp_pct);
                            color.0 = Color::srgb(0.3, 0.8, 1.0); // Cyan
                        }
                    } else {
                        **text = "ESCORT: DESTROYED".to_string();
                        color.0 = Color::srgb(1.0, 0.2, 0.2);
                    }
                } else {
                    **text = String::new();
                }
            } else {
                **text = String::new();
            }
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
