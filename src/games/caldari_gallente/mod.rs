//! Battle of Caldari Prime Module
//!
//! Caldari vs Gallente faction warfare over Caldari Prime.

use super::{ActiveModule, FactionInfo, GameModuleInfo, ModuleRegistry};
use crate::core::GameState;
use bevy::ecs::schedule::common_conditions::not;
use bevy::prelude::*;

pub mod campaign;
pub mod cg_campaign;
pub mod cg_screens;
pub mod faction_select;
pub mod last_stand;
pub mod last_stand_gameplay;
pub mod mode_select;
pub mod nightmare;
pub mod ships;

pub use campaign::{CGCampaignState, ShiigeruNightmare, VerticalSliceMode};
pub use last_stand::LastStandState;
pub use ships::*;

/// State for mode selection
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CGModeSelect {
    #[default]
    Inactive,
    Active,
}

/// Caldari/Gallente module plugin
pub struct CaldariGallentePlugin;

impl Plugin for CaldariGallentePlugin {
    fn build(&self, app: &mut App) {
        // Register module
        app.add_systems(Startup, register_module);

        // Initialize state for mode select
        app.init_state::<CGModeSelect>();

        // Faction select screen - only when this module is active
        app.add_systems(
            OnEnter(GameState::FactionSelect),
            faction_select::spawn_faction_select.run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            faction_select::faction_select_input
                .run_if(in_state(GameState::FactionSelect))
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            OnExit(GameState::FactionSelect),
            faction_select::despawn_faction_select.run_if(is_caldari_gallente),
        );

        // Mode select screen (Campaign vs Nightmare) - Caldari only
        app.add_systems(
            OnEnter(CGModeSelect::Active),
            mode_select::spawn_mode_select,
        )
        .add_systems(
            Update,
            mode_select::mode_select_input.run_if(in_state(CGModeSelect::Active)),
        )
        .add_systems(
            OnExit(CGModeSelect::Active),
            mode_select::despawn_mode_select,
        );

        // Initialize resources
        app.init_resource::<CaldariGallenteShips>();
        app.init_resource::<ShiigeruNightmare>();
        app.init_resource::<CGCampaignState>();
        app.init_resource::<LastStandState>();
        app.init_resource::<VerticalSliceMode>();

        // CG Campaign systems - run instead of main campaign when CG module is active
        // Skip when nightmare mode or Last Stand mode is active
        app.add_systems(
            OnEnter(GameState::Playing),
            (
                cg_campaign::cleanup_cg_entities,
                cg_campaign::start_cg_mission,
            )
                .chain()
                .run_if(is_caldari_gallente)
                .run_if(not(nightmare_active))
                .run_if(not(last_stand_active)),
        )
        .add_systems(
            Update,
            (
                cg_campaign::update_cg_mission,
                cg_campaign::check_cg_wave_complete,
                cg_campaign::spawn_cg_wave,
            )
                .chain()
                .run_if(in_state(GameState::Playing))
                .run_if(is_caldari_gallente)
                .run_if(not(nightmare_active))
                .run_if(not(last_stand_active)),
        )
        .add_systems(
            Update,
            cg_campaign::update_cg_telegraphs
                .run_if(in_state(GameState::Playing))
                .run_if(is_caldari_gallente)
                .run_if(not(nightmare_active))
                .run_if(not(last_stand_active)),
        )
        .add_systems(
            OnEnter(GameState::BossIntro),
            (cg_campaign::spawn_cg_boss, cg_campaign::spawn_cg_boss_intro)
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            (
                cg_campaign::cg_boss_intro,
                cg_campaign::cg_boss_intro_update,
            )
                .run_if(in_state(GameState::BossIntro))
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            OnExit(GameState::BossIntro),
            cg_campaign::despawn_cg_boss_intro.run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            (
                cg_campaign::update_cg_boss,
                cg_campaign::check_cg_boss_defeated,
            )
                .run_if(in_state(GameState::BossFight))
                .run_if(is_caldari_gallente),
        );

        // Nightmare mode systems
        app.add_systems(
            Update,
            (
                nightmare::update_nightmare_mode,
                nightmare::spawn_nightmare_enemies,
                nightmare::update_nightmare_hud,
                nightmare::update_wave_announcements,
                nightmare::update_miniboss_intros,
            )
                .chain()
                .run_if(in_state(GameState::Playing))
                .run_if(nightmare_active),
        );

        // Spawn nightmare HUD when entering Playing in nightmare mode
        app.add_systems(
            OnEnter(GameState::Playing),
            nightmare::spawn_nightmare_hud.run_if(nightmare_active),
        );

        // Last Stand systems - fixed platform defense mode
        app.add_systems(
            OnEnter(GameState::Playing),
            last_stand_gameplay::spawn_last_stand.run_if(last_stand_active),
        )
        .add_systems(
            Update,
            (
                last_stand_gameplay::update_last_stand,
                last_stand_gameplay::last_stand_input,
                last_stand_gameplay::update_last_stand_hud,
                last_stand_gameplay::spawn_last_stand_enemies,
                last_stand_gameplay::update_titan_fighters,
            )
                .chain()
                .run_if(in_state(GameState::Playing))
                .run_if(last_stand_active),
        )
        .add_systems(
            OnExit(GameState::Playing),
            last_stand_gameplay::despawn_last_stand.run_if(last_stand_active),
        );

        // CG Stage Complete screen
        app.add_systems(
            OnEnter(GameState::StageComplete),
            cg_screens::spawn_cg_stage_complete.run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            cg_screens::cg_stage_complete_input
                .run_if(in_state(GameState::StageComplete))
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            OnExit(GameState::StageComplete),
            cg_screens::despawn_cg_stage_complete.run_if(is_caldari_gallente),
        );

        // CG Victory screen
        app.add_systems(
            OnEnter(GameState::Victory),
            cg_screens::spawn_cg_victory_screen.run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            (
                cg_screens::update_cg_victory_particles,
                cg_screens::cg_victory_input,
            )
                .run_if(in_state(GameState::Victory))
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            OnExit(GameState::Victory),
            cg_screens::despawn_cg_victory.run_if(is_caldari_gallente),
        );

        // Vertical Slice Complete screen
        app.add_systems(
            OnEnter(GameState::SliceComplete),
            cg_screens::spawn_cg_slice_complete.run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            cg_screens::cg_slice_complete_input
                .run_if(in_state(GameState::SliceComplete))
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            OnExit(GameState::SliceComplete),
            cg_screens::despawn_cg_slice_complete.run_if(is_caldari_gallente),
        );
    }
}

/// Run condition: is the active module Caldari vs Gallente?
fn is_caldari_gallente(active_module: Res<ActiveModule>) -> bool {
    active_module.is_caldari_gallente()
}

/// Run condition: is nightmare mode active?
fn nightmare_active(nightmare: Res<ShiigeruNightmare>) -> bool {
    nightmare.active
}

/// Run condition: is Last Stand mode active?
fn last_stand_active(last_stand: Res<LastStandState>) -> bool {
    last_stand.active
}

fn register_module(mut registry: ResMut<ModuleRegistry>) {
    registry.register(GameModuleInfo {
        id: "caldari_gallente",
        display_name: "Battle of Caldari Prime",
        subtitle: "The War for Caldari Prime",
        description: "Experience the brutal conflict between Caldari and Gallente forces.",
        factions: vec![
            FactionInfo {
                id: "caldari",
                name: "Caldari State",
                primary_color: Color::srgb(0.1, 0.29, 0.48),
                secondary_color: Color::srgb(0.29, 0.6, 0.79),
                accent_color: Color::srgb(0.48, 0.79, 0.79),
                doctrine: vec!["Missiles", "Shields", "ECM"],
                description: "Corporate efficiency meets military precision.",
            },
            FactionInfo {
                id: "gallente",
                name: "Gallente Federation",
                primary_color: Color::srgb(0.16, 0.35, 0.16),
                secondary_color: Color::srgb(0.35, 0.79, 0.35),
                accent_color: Color::srgb(0.54, 0.92, 0.54),
                doctrine: vec!["Drones", "Armor", "Blasters"],
                description: "Freedom through firepower.",
            },
        ],
    });
}
