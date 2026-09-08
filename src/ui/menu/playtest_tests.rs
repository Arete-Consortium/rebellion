//! Automated route proof: real schedules/menus, with combat cleared by the
//! harness. This checks finishability, not human difficulty or artwork quality.
use super::*;
use crate::app_builder::build_headless_app;
use crate::entities::{Enemy, EnemyStats};
use crate::games::caldari_gallente::{CGCampaignState, VerticalSliceMode};
use crate::games::GameModulesPlugin;
use crate::ui::TransitionPlugin;

fn tick(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}
fn state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}
fn confirm(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    tick(app, 65); // complete both transition fades
}

#[test]
fn caldari_three_missions_results_and_replay_use_the_actual_menu_route() {
    prove_three_mission_route(0);
}

#[test]
fn gallente_three_missions_results_and_replay_use_the_actual_menu_route() {
    prove_three_mission_route(1);
}

fn prove_three_mission_route(side: usize) {
    let mut app = build_headless_app();
    app.add_plugins((GameModulesPlugin, MenuPlugin, TransitionPlugin));
    app.add_event::<AppExit>();
    app.init_resource::<crate::systems::audio::SoundAssets>();
    app.init_resource::<crate::assets::FactionIconCache>();
    app.init_resource::<crate::systems::touch_joystick::MobileMode>();
    tick(&mut app, 125);
    assert_eq!(state(&app), GameState::MainMenu);
    choose_chapter_and_hull(&mut app, 1, side);
    assert_eq!(state(&app), GameState::MissionBriefing);
    assert!(app.world().resource::<VerticalSliceMode>().is_slice());
    confirm(&mut app);
    assert_eq!(state(&app), GameState::Playing);
    let mut completed = Vec::new();
    let mut bosses_seen = std::collections::BTreeSet::new();
    for _ in 0..16000 {
        let current = state(&app);
        let mission = app.world().resource::<CGCampaignState>().mission_index;
        if current == GameState::BossFight {
            bosses_seen.insert(mission);
        }
        if current == GameState::StageComplete {
            assert_eq!(
                mission,
                completed.len(),
                "each result advances exactly once"
            );
            completed.push(mission);
            confirm(&mut app);
            if state(&app) == GameState::SliceComplete {
                break;
            }
        }
        // Use the production death/event path; do not directly despawn waves,
        // set completion flags, or jump the campaign/state forwards.
        for mut stats in app
            .world_mut()
            .query_filtered::<&mut EnemyStats, With<Enemy>>()
            .iter_mut(app.world_mut())
        {
            stats.health = 0.0;
        }
        app.update();
    }
    assert_eq!(
        completed,
        [0, 1, 2],
        "stopped in {:?}, mission {:?}",
        state(&app),
        app.world().resource::<CGCampaignState>()
    );
    assert_eq!(bosses_seen.into_iter().collect::<Vec<_>>(), [1, 2]);
    assert_eq!(state(&app), GameState::SliceComplete);
    confirm(&mut app);
    assert_eq!(state(&app), GameState::MainMenu);
    choose_chapter_and_hull(&mut app, 1, side);
    assert_eq!(state(&app), GameState::MissionBriefing);
    assert_eq!(app.world().resource::<CGCampaignState>().mission_index, 0);
    assert!(app.world().resource::<VerticalSliceMode>().is_slice());
}

fn choose_chapter_and_hull(app: &mut App, chapter: usize, faction: usize) {
    confirm(app);
    assert_eq!(state(app), GameState::ModuleSelect);
    app.world_mut()
        .resource_mut::<common::MenuSelection>()
        .index = chapter;
    confirm(app);
    assert_eq!(state(app), GameState::FactionSelect);
    app.world_mut()
        .resource_mut::<common::MenuSelection>()
        .index = faction;
    confirm(app);
    assert_eq!(state(app), GameState::DifficultySelect);
    confirm(app);
    assert_eq!(state(app), GameState::ShipSelect);
    confirm(app);
}

#[test]
fn both_chapters_offer_both_factions_and_their_own_hulls() {
    for (chapter, side, player, enemy) in [
        (0, 0, Faction::Minmatar, Faction::Amarr),
        (0, 1, Faction::Amarr, Faction::Minmatar),
        (1, 0, Faction::Caldari, Faction::Gallente),
        (1, 1, Faction::Gallente, Faction::Caldari),
    ] {
        let mut app = build_headless_app();
        app.add_plugins((GameModulesPlugin, MenuPlugin, TransitionPlugin));
        app.add_event::<AppExit>();
        app.init_resource::<crate::assets::FactionIconCache>();
        app.init_resource::<crate::systems::audio::SoundAssets>();
        app.init_resource::<crate::systems::touch_joystick::MobileMode>();
        tick(&mut app, 125);
        choose_chapter_and_hull(&mut app, chapter, side);
        assert_eq!(state(&app), GameState::MissionBriefing);
        let session = app.world().resource::<GameSession>();
        assert_eq!(session.player_faction, player);
        assert_eq!(session.enemy_faction, enemy);
        assert_eq!(
            session.selected_ship().type_id,
            player.player_ships()[0].type_id
        );
        assert!(session.player_ships().len() >= 3);
        confirm(&mut app);
        assert_eq!(state(&app), GameState::Playing);
        tick(&mut app, 180);
        assert!(
            app.world_mut()
                .query_filtered::<Entity, With<Enemy>>()
                .iter(app.world())
                .count()
                > 0,
            "carrier must release a wave"
        );
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../../assets/ships/ship_manifest.json")).unwrap();
        let roster = manifest["ships"].as_array().unwrap();
        for stats in app
            .world_mut()
            .query_filtered::<&EnemyStats, With<Enemy>>()
            .iter(app.world())
        {
            assert!(
                roster
                    .iter()
                    .any(|h| h["type_id"].as_u64() == Some(stats.type_id as u64)
                        && h["faction"].as_str()
                            == Some(enemy.short_name().to_ascii_lowercase().as_str())),
                "foreign hull {} in {:?} waves",
                stats.type_id,
                enemy
            );
        }
    }
}
