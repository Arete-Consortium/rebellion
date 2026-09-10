//! Stage Complete Screen

#![allow(dead_code)]

use crate::core::KeyBindings;
use crate::core::*;
use crate::systems::JoystickState;
use crate::ui::menu::common::*;
use crate::ui::TransitionEvent;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct StageCompleteRoot;

pub(crate) fn spawn_stage_complete(
    mut commands: Commands,
    campaign: Res<CampaignState>,
    score: Res<ScoreSystem>,
    session: Res<GameSession>,
    active: Res<crate::games::ActiveModule>,
    ef: Res<crate::games::elder_fleet::ElderFleetCampaignState>,
    bindings: Res<KeyBindings>,
    transport: Option<Res<crate::games::elder_fleet::transport::TransportObjective>>,
) {
    let mission_name = if active.is_elder_fleet() {
        crate::games::elder_fleet::mission_info(&active, ef.current_mission.saturating_sub(1))
            .map(|m| m.name)
            .unwrap_or("MISSION")
    } else {
        campaign
            .current_mission()
            .map(|m| m.name)
            .unwrap_or("MISSION")
    };

    let bonus_text = if active.is_elder_fleet() {
        "ENEMY COMMANDER DEFEATED"
    } else if campaign.bonus_complete {
        "BONUS OBJECTIVE COMPLETE!"
    } else if let Some(m) = campaign.current_mission() {
        m.bonus_objective.unwrap_or("")
    } else {
        ""
    };

    // Check if any ships were unlocked by completing this stage
    let completed_stage = if active.is_elder_fleet() {
        ef.current_mission
    } else {
        campaign.stage_number()
    };
    let ships = session.player_ships();
    let unlocked_ships: Vec<&str> = ships
        .iter()
        .filter(|s| s.unlock_stage == completed_stage)
        .map(|s| s.name)
        .collect();

    commands
        .spawn((
            StageCompleteRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.05, 0.0, 0.9)),
        ))
        .with_children(|parent| {
            // Victory header
            parent.spawn((
                Text::new("MISSION COMPLETE"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
            ));

            parent.spawn((
                Text::new(mission_name),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Stats
            parent.spawn((
                Text::new(format!("Score: {}", score.score)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(if active.is_elder_fleet() {
                    transport
                        .as_deref()
                        .filter(|t| {
                            t.phase == crate::games::elder_fleet::transport::TransportPhase::Secured
                        })
                        .map(|t| t.success_label())
                        .unwrap_or("Sector secured")
                        .to_string()
                } else {
                    format!("Souls Liberated: {}", campaign.mission_souls)
                }),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.8, 1.0)),
            ));

            parent.spawn((
                Text::new(if active.is_elder_fleet() {
                    score.run.summary()
                } else {
                    format!("Time: {:.1}s", campaign.mission_timer)
                }),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Bonus objective
            if !bonus_text.is_empty() {
                parent.spawn(Node {
                    height: Val::Px(10.0),
                    ..default()
                });

                let bonus_color = if campaign.bonus_complete {
                    Color::srgb(1.0, 0.85, 0.2) // Gold
                } else {
                    Color::srgb(0.5, 0.5, 0.5) // Gray
                };

                parent.spawn((
                    Text::new(bonus_text),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(bonus_color),
                ));
            }

            // Ship unlock notification
            if !unlocked_ships.is_empty() {
                parent.spawn(Node {
                    height: Val::Px(15.0),
                    ..default()
                });

                parent
                    .spawn((
                        Node {
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(5.0),
                            ..default()
                        },
                        BorderColor(Color::srgb(0.2, 0.8, 0.4)),
                        BackgroundColor(Color::srgba(0.1, 0.3, 0.15, 0.9)),
                    ))
                    .with_children(|unlock_box| {
                        unlock_box.spawn((
                            Text::new("NEW SHIP UNLOCKED!"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 1.0, 0.4)),
                        ));

                        for ship_name in &unlocked_ships {
                            unlock_box.spawn((
                                Text::new(*ship_name),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.3)),
                            ));
                        }
                    });
            }

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Clickable continue action uses the same navigation event path.
            parent.spawn((
                MenuItem { index: 0 },
                Interaction::default(),
                Text::new(menu_hint(&bindings, "Continue", "Main menu")),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

pub(crate) fn stage_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    joystick: Res<JoystickState>,
    mut campaign: ResMut<CampaignState>,
    active: Res<crate::games::ActiveModule>,
    ef: Res<crate::games::elder_fleet::ElderFleetCampaignState>,
    mut transitions: EventWriter<TransitionEvent>,
) {
    if is_confirm(&keyboard, &joystick, &bindings) {
        if active.is_elder_fleet() {
            transitions.send(TransitionEvent::to(
                if ef.current_mission < ef.total_missions() {
                    GameState::MissionBriefing
                } else {
                    GameState::Victory
                },
            ));
            return;
        }
        // Advance to next mission
        if campaign.complete_mission() {
            // More missions available
            transitions.send(TransitionEvent::to(GameState::Playing));
        } else {
            // Campaign complete!
            transitions.send(TransitionEvent::slow(GameState::Victory));
        }
    }

    if is_cancel(&keyboard, &joystick, &bindings) {
        transitions.send(TransitionEvent::to(GameState::MainMenu));
    }
}

#[cfg(test)]
mod run_statistics_ui_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn elder_fleet_stage_displays_run_statistics_after_combo_expires() {
        let mut app = App::new();
        let mut active = crate::games::ActiveModule::default();
        active.set_module("elder_fleet");
        let mut score = ScoreSystem::default();
        for _ in 0..7 {
            score.on_kill(10);
        }
        score.update(60.0);
        score.run.combat_seconds = 125.0;
        app.insert_resource(active)
            .insert_resource(score)
            .insert_resource(GameSession::new(Faction::Minmatar, Faction::Amarr))
            .insert_resource(crate::games::elder_fleet::ElderFleetCampaignState {
                current_mission: 1,
                ..default()
            })
            .init_resource::<CampaignState>()
            .init_resource::<KeyBindings>();
        app.world_mut()
            .run_system_once(spawn_stage_complete)
            .unwrap();
        let labels: Vec<_> = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|text| text.0.clone())
            .collect();
        assert!(labels
            .iter()
            .any(|text| text == "Best chain: 7x  |  Combat: 2:05"));
    }
}
