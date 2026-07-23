//! Heads-Up Display
//!
//! In-game UI: health bars, score, combo, heat, salt miner meter, powerup indicators.
//! Status panel with capacitor and health rings.

#![allow(dead_code)]

mod achievements;
mod boss;
mod combat;
mod common;
mod inventory;
mod meters;
mod mission;
mod powerups;
mod score;
mod spawn;

use bevy::prelude::*;

use crate::core::GameState;

use achievements::update_achievement_popup;
use boss::update_boss_health_bar;
use combat::{
    update_ability_indicator, update_ammo_display, update_drone_status, update_wingman_gauge,
};
use inventory::{spawn_inventory_hud, update_inventory_hud};
use meters::{update_heat_display, update_salt_miner_meter};
use mission::{update_dialogue_display, update_mission_display, update_wave_display};
use powerups::{update_buff_expiration_warnings, update_powerup_indicators};
use score::{
    update_combo_display, update_combo_kills, update_combo_timer_bar, update_score_display,
};
use spawn::{despawn_hud, spawn_hud};

/// HUD plugin
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (spawn_hud, spawn_inventory_hud).run_if(not_last_stand),
        )
        .add_systems(
            Update,
            (
                update_score_display,
                update_salt_miner_meter,
                update_combo_display,
                update_heat_display,
                update_combo_kills,
                update_combo_timer_bar,
                update_powerup_indicators,
                update_buff_expiration_warnings,
                update_wave_display,
                update_mission_display,
                update_boss_health_bar,
                update_dialogue_display,
                update_wingman_gauge,
                update_drone_status,
                update_ability_indicator,
                update_ammo_display,
                update_achievement_popup,
                update_inventory_hud,
            )
                .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight)))
                .run_if(not_last_stand),
        )
        .add_systems(
            OnExit(GameState::Playing),
            despawn_hud.run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
        );
    }
}

/// Run condition: NOT in Last Stand mode
fn not_last_stand(last_stand: Option<Res<crate::games::caldari_gallente::LastStandState>>) -> bool {
    match last_stand {
        Some(ls) => !ls.active,
        None => true,
    }
}
