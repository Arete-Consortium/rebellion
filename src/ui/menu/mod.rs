//! Menu Systems
//!
//! Complete menu flow: Title -> Difficulty -> Ship -> Playing
//! Supports keyboard, mouse, and joystick input.

pub mod boss_intro;
pub mod common;
pub mod death_screen;
pub mod difficulty_select;
pub mod endless_announcements;
pub mod faction_select;
pub mod loading;
pub mod main_menu;
pub mod mission_briefing;
pub mod module_select;
pub mod options;
pub mod pause;
pub mod ship_select;
pub mod stage_complete;
pub mod stage_select;
pub mod upgrade_shop;
pub mod victory;

pub(crate) use common::*;

use crate::core::*;
use bevy::prelude::*;

/// Menu plugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Loading
            .add_systems(OnEnter(GameState::Loading), loading::spawn_loading_screen)
            .add_systems(
                Update,
                loading::loading_progress.run_if(in_state(GameState::Loading)),
            )
            .add_systems(
                OnExit(GameState::Loading),
                despawn_menu::<loading::LoadingRoot>,
            )
            // Main Menu
            .add_systems(OnEnter(GameState::MainMenu), main_menu::spawn_main_menu)
            .add_systems(
                Update,
                (
                    main_menu::main_menu_input,
                    update_menu_selection::<main_menu::MainMenuRoot>,
                )
                    .run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                OnExit(GameState::MainMenu),
                despawn_menu::<main_menu::MainMenuRoot>,
            )
            // Module Select
            .add_systems(
                OnEnter(GameState::ModuleSelect),
                module_select::spawn_module_select,
            )
            .add_systems(
                Update,
                (
                    module_select::module_select_input,
                    update_menu_selection::<module_select::ModuleSelectRoot>,
                )
                    .run_if(in_state(GameState::ModuleSelect)),
            )
            .add_systems(
                OnExit(GameState::ModuleSelect),
                despawn_menu::<module_select::ModuleSelectRoot>,
            )
            // Options Menu
            .add_systems(OnEnter(GameState::Options), options::spawn_options_menu)
            .add_systems(
                Update,
                options::options_menu_input.run_if(in_state(GameState::Options)),
            )
            .add_systems(
                OnExit(GameState::Options),
                despawn_menu::<options::OptionsMenuRoot>,
            )
            // Faction Select (unified 4-faction) - only for Elder Fleet module
            .add_systems(
                OnEnter(GameState::FactionSelect),
                faction_select::spawn_faction_select.run_if(module_select::is_elder_fleet),
            )
            .add_systems(
                Update,
                faction_select::faction_select_input
                    .run_if(in_state(GameState::FactionSelect))
                    .run_if(module_select::is_elder_fleet),
            )
            .add_systems(
                OnExit(GameState::FactionSelect),
                despawn_menu::<faction_select::FactionSelectRoot>
                    .run_if(module_select::is_elder_fleet),
            )
            // Difficulty Select
            .add_systems(
                OnEnter(GameState::DifficultySelect),
                difficulty_select::spawn_difficulty_menu,
            )
            .add_systems(
                Update,
                (
                    difficulty_select::difficulty_menu_input,
                    update_menu_selection::<difficulty_select::DifficultyMenuRoot>,
                )
                    .run_if(in_state(GameState::DifficultySelect)),
            )
            .add_systems(
                OnExit(GameState::DifficultySelect),
                despawn_menu::<difficulty_select::DifficultyMenuRoot>,
            )
            // Stage Select
            .add_systems(
                OnEnter(GameState::StageSelect),
                stage_select::spawn_stage_select,
            )
            .add_systems(
                Update,
                (
                    stage_select::stage_select_input,
                    update_menu_selection::<stage_select::StageSelectRoot>,
                )
                    .run_if(in_state(GameState::StageSelect)),
            )
            .add_systems(
                OnExit(GameState::StageSelect),
                despawn_menu::<stage_select::StageSelectRoot>,
            )
            // Ship Select
            .add_systems(OnEnter(GameState::ShipSelect), ship_select::spawn_ship_menu)
            .add_systems(
                Update,
                (
                    ship_select::ship_menu_input,
                    update_menu_selection::<ship_select::ShipMenuRoot>,
                    ship_select::update_ship_detail_panel,
                )
                    .run_if(in_state(GameState::ShipSelect)),
            )
            .add_systems(
                OnExit(GameState::ShipSelect),
                despawn_menu::<ship_select::ShipMenuRoot>,
            )
            // Mission Briefing — pre-gameplay lore + objectives
            .add_systems(
                OnEnter(GameState::MissionBriefing),
                mission_briefing::spawn_mission_briefing,
            )
            .add_systems(
                Update,
                mission_briefing::mission_briefing_input
                    .run_if(in_state(GameState::MissionBriefing)),
            )
            .add_systems(
                OnExit(GameState::MissionBriefing),
                despawn_menu::<mission_briefing::MissionBriefingRoot>,
            )
            // Upgrade Shop
            .add_systems(
                OnEnter(GameState::UpgradeShop),
                upgrade_shop::spawn_upgrade_shop,
            )
            .add_systems(
                Update,
                (
                    upgrade_shop::upgrade_shop_input,
                    upgrade_shop::update_upgrade_shop_selection,
                )
                    .run_if(in_state(GameState::UpgradeShop)),
            )
            .add_systems(
                OnExit(GameState::UpgradeShop),
                despawn_menu::<upgrade_shop::UpgradeShopRoot>,
            )
            // Pause Menu
            .add_systems(OnEnter(GameState::Paused), pause::spawn_pause_menu)
            .add_systems(
                Update,
                pause::pause_menu_input.run_if(in_state(GameState::Paused)),
            )
            .add_systems(
                OnExit(GameState::Paused),
                despawn_menu::<pause::PauseMenuRoot>,
            )
            // Game Over (Death Screen with corpse and debris)
            .add_systems(
                OnEnter(GameState::GameOver),
                death_screen::spawn_death_screen,
            )
            .add_systems(
                Update,
                (
                    death_screen::update_death_screen_animation,
                    death_screen::death_screen_input,
                )
                    .run_if(in_state(GameState::GameOver)),
            )
            .add_systems(
                OnExit(GameState::GameOver),
                death_screen::despawn_death_screen,
            )
            // Boss Intro (Elder Fleet only - CG has its own)
            .add_systems(
                OnEnter(GameState::BossIntro),
                boss_intro::spawn_boss_intro.run_if(module_select::is_elder_fleet),
            )
            .add_systems(
                Update,
                boss_intro::boss_intro_update
                    .run_if(in_state(GameState::BossIntro))
                    .run_if(module_select::is_elder_fleet),
            )
            .add_systems(
                OnExit(GameState::BossIntro),
                despawn_menu::<boss_intro::BossIntroRoot>,
            )
            // Stage Complete (Elder Fleet only - CG has its own)
            .add_systems(
                OnEnter(GameState::StageComplete),
                stage_complete::spawn_stage_complete.run_if(module_select::is_elder_fleet),
            )
            .add_systems(
                Update,
                stage_complete::stage_complete_input
                    .run_if(in_state(GameState::StageComplete))
                    .run_if(module_select::is_elder_fleet),
            )
            .add_systems(
                OnExit(GameState::StageComplete),
                despawn_menu::<stage_complete::StageCompleteRoot>
                    .run_if(module_select::is_elder_fleet),
            )
            // Victory (Elder Fleet only - CG has its own)
            .add_systems(
                OnEnter(GameState::Victory),
                victory::spawn_victory_screen.run_if(module_select::is_elder_fleet),
            )
            .add_systems(
                Update,
                (
                    victory::victory_input,
                    victory::update_victory_particles,
                    victory::update_victory_buttons,
                )
                    .run_if(in_state(GameState::Victory))
                    .run_if(module_select::is_elder_fleet),
            )
            .add_systems(
                OnExit(GameState::Victory),
                victory::despawn_victory_screen.run_if(module_select::is_elder_fleet),
            )
            // Endless Mode Announcements (Elder Fleet only)
            .add_systems(
                Update,
                (
                    endless_announcements::spawn_endless_wave_announcements,
                    endless_announcements::spawn_endless_miniboss_announcement,
                    endless_announcements::update_endless_announcements,
                )
                    .run_if(in_state(GameState::Playing))
                    .run_if(module_select::is_elder_fleet),
            )
            // Init menu selection resource
            .init_resource::<MenuSelection>();
    }
}
