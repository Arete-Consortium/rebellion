//! Mission Briefing screen
//!
//! Shown after ship select, before gameplay starts. Displays the act/mission
//! title, lore context, primary objective, boss preview, and a "Press A to
//! deploy" prompt. Reads from CampaignState when available, falls back to
//! GameSession for Caldari/Gallente mode.

#![allow(dead_code)]

use crate::core::*;
use crate::systems::JoystickState;
use bevy::prelude::*;

#[derive(Component)]
pub struct MissionBriefingRoot;

/// Content resolver — returns (act_line, mission_name, lore, objective, boss_line, accent)
fn resolve_briefing(
    campaign: Option<&CampaignState>,
    session: Option<&GameSession>,
) -> (String, String, String, String, String, Color) {
    // Prefer CampaignState when present
    if let Some(c) = campaign {
        if let Some(m) = c.current_mission() {
            let act_num = match c.act {
                Act::Act1 => 1,
                Act::Act2 => 2,
                Act::Act3 => 3,
            };
            let act_title = match c.act {
                Act::Act1 => "THE CALL",
                Act::Act2 => "THE STORM",
                Act::Act3 => "LIBERATION",
            };
            let accent = session
                .map(|s| s.player_faction.primary_color())
                .unwrap_or(Color::srgb(0.71, 0.39, 0.20));
            return (
                format!("ACT {} · {}", act_num, act_title),
                m.name.to_string(),
                m.description.to_string(),
                m.primary_objective.to_string(),
                format!("⚠ BOSS: {}", m.boss.name()),
                accent,
            );
        }
    }

    // Fallback to GameSession (Caldari/Gallente mode, etc.)
    if let Some(s) = session {
        let accent = s.player_faction.primary_color();
        let rival = s.enemy_faction;
        let lore = match (s.player_faction, rival) {
            (Faction::Caldari, Faction::Gallente) => {
                "Federation forces probe Caldari orbital defenses. First contact \
                 over contested space. The State demands its stars back."
                    .to_string()
            }
            (Faction::Gallente, Faction::Caldari) => {
                "Caldari encroachment threatens Federation trade lanes. Your \
                 squadron is the first line of response. Liberty does not wait."
                    .to_string()
            }
            (Faction::Minmatar, Faction::Amarr) => {
                "The chains that bind a thousand cannot hold the one who wakes \
                 the rest. Strike the slavers before they reinforce."
                    .to_string()
            }
            (Faction::Amarr, Faction::Minmatar) => {
                "Rebel elements threaten Imperial holdings. The Empress commands \
                 their correction. Amarr Victor."
                    .to_string()
            }
            _ => format!(
                "{} hostiles contest this region. Engage and establish dominance.",
                rival.short_name()
            ),
        };
        return (
            format!("BATTLE OF {}", s.player_faction.short_name()),
            format!(
                "{} vs {}",
                s.player_faction.short_name(),
                rival.short_name()
            ),
            lore,
            format!(
                "Defeat {} forces. Survive all waves. Destroy the flagship.",
                rival.short_name()
            ),
            format!("⚠ ENEMY FLAGSHIP: {} CARRIER", rival.short_name()),
            accent,
        );
    }

    (
        "DEPLOYMENT".into(),
        "UNKNOWN SECTOR".into(),
        "Contact imminent. Prepare for engagement.".into(),
        "Survive. Destroy hostiles.".into(),
        "⚠ HEAVY RESISTANCE".into(),
        Color::srgb(0.3, 0.6, 1.0),
    )
}

pub fn spawn_mission_briefing(
    mut commands: Commands,
    campaign: Option<Res<CampaignState>>,
    session: Option<Res<GameSession>>,
) {
    let (act_line, mission_name, lore, objective, boss_line, accent) =
        resolve_briefing(campaign.as_deref(), session.as_deref());

    commands
        .spawn((
            MissionBriefingRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(40.0)),
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.04, 0.08)),
        ))
        .with_children(|p| {
            // Top label — act line
            p.spawn((
                Text::new(act_line),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(accent.with_alpha(0.85)),
                Node {
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
            ));

            // Main title — mission name
            p.spawn((
                Text::new(mission_name.to_uppercase()),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));

            // Accent separator line
            p.spawn((
                Node {
                    width: Val::Px(320.0),
                    height: Val::Px(2.0),
                    margin: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(accent),
            ));

            // Lore block
            p.spawn((
                Text::new(lore),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.82, 0.88, 0.95)),
                Node {
                    max_width: Val::Px(720.0),
                    margin: UiRect::vertical(Val::Px(12.0)),
                    ..default()
                },
            ));

            // Objective box
            p.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    margin: UiRect::top(Val::Px(8.0)),
                    min_width: Val::Px(520.0),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BorderColor(accent.with_alpha(0.6)),
                BackgroundColor(Color::srgba(0.04, 0.08, 0.14, 0.9)),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("PRIMARY OBJECTIVE"),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(accent.with_alpha(0.9)),
                ));
                b.spawn((
                    Text::new(objective),
                    TextFont {
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.97, 1.0)),
                ));
            });

            // Boss warning
            p.spawn((
                Text::new(boss_line),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.55, 0.35)),
                Node {
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                },
            ));

            // Controller prompt (controller-only per design)
            p.spawn((
                Text::new("— PRESS A TO DEPLOY —"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::top(Val::Px(28.0)),
                    ..default()
                },
            ));

            p.spawn((
                Text::new("B: back to ship select"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.65, 0.7, 0.75)),
            ));
        });
}

pub fn mission_briefing_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    itch_mode: Res<crate::core::ItchMode>,
    mut next: ResMut<NextState<GameState>>,
) {
    if joystick.confirm()
        || keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::Space)
    {
        next.set(GameState::Playing);
    } else if joystick.back() || keyboard.just_pressed(KeyCode::Escape) {
        if itch_mode.enabled {
            next.set(GameState::MainMenu);
        } else {
            next.set(GameState::ShipSelect);
        }
    }
}
