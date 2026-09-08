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
    ef_campaign: Option<Res<crate::games::elder_fleet::ElderFleetCampaignState>>,
    mut query: Query<&mut Text, With<WaveText>>,
) {
    for mut text in query.iter_mut() {
        if active_module.is_elder_fleet() {
            if let Some(ef) = ef_campaign.as_deref() {
                **text = if ef.boss_spawned {
                    "BOSS ENGAGEMENT".into()
                } else {
                    format!("WAVE {}/{}", ef.current_wave.max(1), ef.waves_in_mission)
                };
            }
            continue;
        }
        if active_module.is_caldari_gallente() {
            if let Some(cg) = cg_campaign.as_deref() {
                if let Some(mission) = cg.current_mission() {
                    if cg.boss_spawned && !cg.boss_defeated {
                        **text = "BOSS ENGAGEMENT".to_string();
                    } else {
                        **text = format!(
                            "WAVE {}/{}",
                            cg.current_wave.saturating_sub(1).clamp(1, mission.waves),
                            mission.waves
                        );
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
    ef_campaign: Option<Res<crate::games::elder_fleet::ElderFleetCampaignState>>,
    score: Res<ScoreSystem>,
    transport: Option<Res<crate::games::elder_fleet::transport::TransportObjective>>,
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
    if active_module.is_elder_fleet() {
        let index = ef_campaign
            .as_deref()
            .map(|ef| ef.current_mission)
            .unwrap_or(0);
        for mut text in &mut mission_query {
            **text = crate::games::elder_fleet::mission_info(&active_module, index)
                .map(|m| format!("M{}: {}", index + 1, m.name))
                .unwrap_or_default();
        }
        for (mut text, mut color) in &mut objective_query {
            **text = transport
                .as_deref()
                .map(|t| t.hud_instruction())
                .unwrap_or("Clear the waves. Defeat the enemy commander.")
                .into();
            color.0 = Color::srgb(0.65, 0.85, 0.95);
        }
        for mut text in &mut souls_query {
            **text = String::new();
        }
        for mut text in &mut kill_query {
            **text = String::new();
        }
        for (mut text, mut color) in &mut escort_query_text {
            **text = transport
                .as_deref()
                .map(|t| t.hud_status())
                .unwrap_or_default();
            color.0 = if transport
                .as_deref()
                .is_some_and(|t| t.health_fraction < 0.3 || t.remaining_seconds() < 15.0)
            {
                Color::srgb(1.0, 0.65, 0.3)
            } else {
                Color::srgb(0.55, 0.95, 1.0)
            };
        }
        return;
    }
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

    if active_module.is_caldari_gallente() {
        for (mut text, mut color) in &mut objective_query {
            **text = cg_campaign
                .as_deref()
                .and_then(|cg| cg.current_mission())
                .map(|mission| mission.primary_objective.to_string())
                .unwrap_or_default();
            color.0 = Color::srgb(0.65, 0.85, 0.95);
        }
        for mut text in &mut souls_query {
            **text = String::new();
        }
        for mut text in &mut kill_query {
            **text = String::new();
        }
        for (mut text, _) in &mut escort_query_text {
            **text = String::new();
        }
        return;
    }
    // Update objective
    for (mut text, mut color) in objective_query.iter_mut() {
        if let Some(mission) = campaign.current_mission() {
            if campaign.primary_complete {
                **text = format!("OK {}", mission.primary_objective);
                color.0 = Color::srgb(0.3, 1.0, 0.3); // Bright green when complete
            } else {
                **text = format!("> {}", mission.primary_objective);
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
                    " OK"
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
                        " OK"
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
