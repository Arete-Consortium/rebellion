//! Triglavian Invasion Module
//!
//! EDENCOM vs Triglavian Collective - defend New Eden or embrace Pochven.
//! Set during the Triglavian invasion of YC122.

use super::{ActiveModule, FactionInfo, GameModuleInfo, ModuleRegistry};
use crate::core::GameState;
use bevy::prelude::*;

pub mod campaign;
pub mod ships;

pub use campaign::*;
pub use ships::*;

/// Triglavian Invasion module plugin
pub struct TriglavianInvasionPlugin;

impl Plugin for TriglavianInvasionPlugin {
    fn build(&self, app: &mut App) {
        // Register module
        app.add_systems(Startup, register_module);

        // Initialize resources
        app.init_resource::<TriglavianShips>();
        app.init_resource::<TriglavianCampaignState>();

        // Faction select screen
        app.add_systems(
            OnEnter(GameState::FactionSelect),
            spawn_faction_select.run_if(is_triglavian_invasion),
        )
        .add_systems(
            Update,
            faction_select_input
                .run_if(in_state(GameState::FactionSelect))
                .run_if(is_triglavian_invasion),
        )
        .add_systems(
            OnExit(GameState::FactionSelect),
            despawn_faction_select.run_if(is_triglavian_invasion),
        );

        // Campaign systems
        app.add_systems(
            OnEnter(GameState::Playing),
            start_trig_mission.run_if(is_triglavian_invasion),
        )
        .add_systems(
            Update,
            (
                update_trig_mission,
                check_trig_wave_complete,
                spawn_trig_wave,
            )
                .chain()
                .run_if(in_state(GameState::Playing))
                .run_if(is_triglavian_invasion),
        );

        // Boss systems
        app.add_systems(
            OnEnter(GameState::BossIntro),
            (spawn_trig_boss, spawn_trig_boss_intro_ui).run_if(is_triglavian_invasion),
        )
        .add_systems(
            Update,
            (trig_boss_intro, update_trig_boss_intro_ui)
                .run_if(in_state(GameState::BossIntro))
                .run_if(is_triglavian_invasion),
        )
        .add_systems(
            OnExit(GameState::BossIntro),
            (despawn_trig_boss_intro, despawn_trig_boss_intro_ui).run_if(is_triglavian_invasion),
        )
        .add_systems(
            Update,
            (update_trig_boss, check_trig_boss_defeated)
                .run_if(in_state(GameState::BossFight))
                .run_if(is_triglavian_invasion),
        )
        // Victory screen
        .add_systems(
            OnEnter(GameState::Victory),
            spawn_trig_victory_screen.run_if(is_triglavian_invasion),
        )
        .add_systems(
            Update,
            trig_victory_input
                .run_if(in_state(GameState::Victory))
                .run_if(is_triglavian_invasion),
        )
        .add_systems(
            OnExit(GameState::Victory),
            despawn_trig_victory.run_if(is_triglavian_invasion),
        );
    }
}

/// Check if Triglavian Invasion module is active
fn is_triglavian_invasion(active: Res<ActiveModule>) -> bool {
    active.module_id.as_deref() == Some("triglavian_invasion")
}

/// Register the Triglavian Invasion module
fn register_module(mut registry: ResMut<ModuleRegistry>) {
    registry.register(GameModuleInfo {
        id: "triglavian_invasion",
        display_name: "Triglavian Invasion",
        subtitle: "YC122 - The Flow of Vyraj",
        description: "Defend New Eden from the Triglavian Collective, or embrace the Flow and fight for Pochven.",
        factions: vec![
            FactionInfo {
                id: "edencom",
                name: "EDENCOM",
                primary_color: Color::srgb(0.2, 0.6, 0.9),    // Blue
                secondary_color: Color::srgb(0.9, 0.9, 0.95), // White
                accent_color: Color::srgb(0.3, 0.8, 1.0),     // Cyan
                doctrine: vec!["Unified Defense", "Shield Tanking", "Coordinated Fire"],
                description: "The unified defense force of the four empires, standing against the Triglavian invasion.",
            },
            FactionInfo {
                id: "triglavian",
                name: "Triglavian Collective",
                primary_color: Color::srgb(0.8, 0.2, 0.2),    // Red
                secondary_color: Color::srgb(0.1, 0.1, 0.12), // Dark gray
                accent_color: Color::srgb(1.0, 0.4, 0.2),     // Orange-red
                doctrine: vec!["Entropic Disintegration", "Bioadaptation", "Proving"],
                description: "Ancient Jove descendants from Abyssal Deadspace, seeking to claim systems for Pochven.",
            },
        ],
    });
}

// =============================================================================
// FACTION SELECT UI
// =============================================================================

/// Marker for faction select UI
#[derive(Component)]
struct TrigFactionSelectUI;

/// Spawn faction selection screen
fn spawn_faction_select(mut commands: Commands) {
    info!("Spawning Triglavian faction select");

    commands
        .spawn((
            TrigFactionSelectUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.04, 0.95)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("TRIGLAVIAN INVASION"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.3, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("Choose Your Side"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Faction choices
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(60.0),
                    ..default()
                })
                .with_children(|row| {
                    // EDENCOM
                    spawn_faction_card(
                        row,
                        "EDENCOM",
                        "Defend New Eden",
                        "Shield the empires from\nthe Triglavian threat",
                        Color::srgb(0.2, 0.6, 0.9),
                        "[A] or LEFT",
                    );

                    // Triglavian
                    spawn_faction_card(
                        row,
                        "TRIGLAVIAN",
                        "Embrace the Flow",
                        "Prove yourself worthy\nand claim Pochven",
                        Color::srgb(0.8, 0.2, 0.2),
                        "[D] or RIGHT",
                    );
                });

            // Back instruction
            parent.spawn((
                Text::new("[ESC] Back to Main Menu"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_faction_card(
    parent: &mut ChildBuilder,
    name: &str,
    tagline: &str,
    description: &str,
    color: Color,
    controls: &str,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor(color),
            BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.8)),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(name),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(color),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(tagline),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(description),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(controls),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(color.with_alpha(0.7)),
            ));
        });
}

/// Handle faction selection input
fn faction_select_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut active: ResMut<ActiveModule>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // EDENCOM (left)
    if keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft) {
        active.set_faction("edencom", "triglavian");
        next_state.set(GameState::Playing);
    }

    // Triglavian (right)
    if keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight) {
        active.set_faction("triglavian", "edencom");
        next_state.set(GameState::Playing);
    }

    // Back to menu
    if keys.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::MainMenu);
    }
}

/// Despawn faction select UI
fn despawn_faction_select(mut commands: Commands, query: Query<Entity, With<TrigFactionSelectUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// =============================================================================
// BOSS INTRO UI
// =============================================================================

#[derive(Component)]
struct TrigBossIntroRoot;

#[derive(Component)]
struct TrigBossIntroWarning {
    timer: f32,
}

fn spawn_trig_boss_intro_ui(
    mut commands: Commands,
    state: Res<TriglavianCampaignState>,
    active: Res<super::ActiveModule>,
) {
    let Some(mission) = edencom_missions()
        .into_iter()
        .chain(triglavian_missions())
        .nth(state.current_mission as usize)
    else {
        return;
    };

    let phase_text = match state.current_mission {
        0..=2 => "Single Phase",
        3..=5 => "Two Phases",
        6..=7 => "Three Phases • Challenging",
        _ => "Multi-Phase",
    };

    let color = if active.player_faction.as_deref() == Some("edencom") {
        Color::srgb(0.2, 0.6, 0.9)
    } else {
        Color::srgb(0.8, 0.2, 0.2)
    };

    commands
        .spawn((
            TrigBossIntroRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("⚠ WARNING ⚠"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                TrigBossIntroWarning { timer: 0.0 },
            ));

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            parent.spawn((
                Text::new(mission.boss_name),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(color),
            ));

            parent.spawn((
                Text::new(mission.name),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            parent.spawn((
                Text::new(phase_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            parent.spawn((
                Text::new("Prepare for battle..."),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

fn update_trig_boss_intro_ui(
    time: Res<Time>,
    mut query: Query<(&mut TextColor, &mut TrigBossIntroWarning)>,
) {
    let dt = time.delta_secs();
    for (mut color, mut warning) in query.iter_mut() {
        warning.timer += dt * 4.0;
        let pulse = (warning.timer.sin() * 0.3 + 0.7).clamp(0.4, 1.0);
        color.0 = Color::srgb(1.0, 0.2 * pulse, 0.2 * pulse);
    }
}

fn despawn_trig_boss_intro_ui(
    mut commands: Commands,
    query: Query<Entity, With<TrigBossIntroRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// =============================================================================
// VICTORY SCREEN
// =============================================================================

#[derive(Component)]
struct TrigVictoryRoot;

fn spawn_trig_victory_screen(
    mut commands: Commands,
    active: Res<super::ActiveModule>,
    score: Res<crate::core::ScoreSystem>,
    mut save_data: ResMut<crate::core::SaveData>,
) {
    let faction = active.player_faction.as_deref().unwrap_or("unknown");
    let (header, subtitle, quote, color) = match faction {
        "edencom" => (
            "NEW EDEN DEFENDED",
            "The Invasion Has Been Repelled",
            "\"We stood together. That was our strength.\"\n— EDENCOM Command",
            Color::srgb(0.2, 0.6, 0.9),
        ),
        "triglavian" => (
            "POCHVEN CLAIMED",
            "The Flow Is Proven",
            "\"Those who resist are unworthy. Those who prove are Clade.\"\n— Zorya Triglav",
            Color::srgb(0.8, 0.2, 0.2),
        ),
        _ => (
            "CAMPAIGN COMPLETE",
            "Victory Achieved",
            "\"Well fought.\"",
            Color::WHITE,
        ),
    };

    // Persist high score
    let faction_key = format!("trig_{}", faction);
    let enemy_key = format!(
        "trig_{}",
        active.enemy_faction.as_deref().unwrap_or("unknown")
    );
    let previous_high = save_data.get_high_score(&faction_key, &enemy_key);
    if score.score > previous_high {
        save_data.record_score(&faction_key, &enemy_key, score.score, 9);
    }

    commands
        .spawn((
            TrigVictoryRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.05, 0.92)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(header),
                TextFont {
                    font_size: 44.0,
                    ..default()
                },
                TextColor(color),
            ));

            parent.spawn((
                Text::new(subtitle),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            parent.spawn((
                Text::new(quote),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            parent.spawn((
                Text::new(format!("Final Score: {}", score.score)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.2)),
            ));

            parent.spawn((
                Text::new("[SPACE] Continue  •  [ESC] Main Menu"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn trig_victory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<crate::systems::JoystickState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        next_state.set(GameState::MainMenu);
    }
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
    }
}

fn despawn_trig_victory(mut commands: Commands, query: Query<Entity, With<TrigVictoryRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
