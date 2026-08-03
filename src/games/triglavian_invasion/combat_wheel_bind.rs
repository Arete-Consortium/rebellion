//! Adapter systems that project the real game's `ShipStats` Component +
//! `ComboHeatSystem` Resource into the `CombatWheelAdapter` Resource that
//! the Combat Wheel HUD's bind systems read.
//!
//! Lives under `triglavian_invasion/` because the Combat Wheel is gated to
//! that campaign only — the bind systems run when the campaign is active and
//! are otherwise inert.
//!
//! Wiring is the campaign plugin's responsibility
//! (`TriglavianInvasionPlugin::build()`).

use bevy::prelude::*;

use crate::core::events::PlayerDamagedEvent;
use crate::entities::player::{Player, ShipStats};
use crate::games::ActiveModule;
use crate::systems::scoring_v2::ComboHeatSystem;
use crate::ui::combat_wheel::{CombatWheelAdapter, CombatWheelEvent, FactionSkin};

/// Projects the player's `ShipStats` Component into the
/// `CombatWheelAdapter` Resource every `FixedUpdate`. Polling matches the
/// existing `capacitor.rs` pattern (`src/ui/capacitor.rs:75`) — there is no
/// `Changed<ShipStats>` filter anywhere in the UI layer.
///
/// `last_damage_direction` is left untouched here; it's set by
/// `forward_damage_events` when a `PlayerDamagedEvent` is observed.
pub fn project_ship_stats_to_combat_wheel(
    player_query: Query<&ShipStats, With<Player>>,
    mut adapter: ResMut<CombatWheelAdapter>,
) {
    let Ok(stats) = player_query.get_single() else {
        return;
    };

    adapter.shield_current = stats.shield;
    adapter.shield_max = stats.max_shield;
    adapter.shield_recharge_rate = stats.shield_recharge;

    // Shield collapsed when current is zero AND recharge delay hasn't
    // elapsed. Real-game `shield_timer > 0` means recharge is pending.
    adapter.shield_collapsed = stats.shield <= 0.0 && stats.shield_timer > 0.0;

    adapter.hull_current = stats.hull;
    adapter.hull_max = stats.max_hull;
    adapter.armor_current = stats.armor;
    adapter.armor_max = stats.max_armor;
    // No "repair active" concept in the real game's ShipStats — repair
    // happens via abilities, not a separate flag. Default to false.
    adapter.repair_active = false;

    adapter.capacitor_current = stats.capacitor;
    adapter.capacitor_max = stats.max_capacitor;
    adapter.capacitor_regen_rate = stats.capacitor_recharge;
}

/// Projects `ComboHeatSystem.heat` (0-100 scale) into the adapter's heat
/// fields. `ComboHeatSystem` doesn't expose warning/critical thresholds as
/// separate fields, so we use the prototype's defaults (60% / 85% of max).
pub fn project_combo_heat_to_combat_wheel(
    heat: Res<ComboHeatSystem>,
    mut adapter: ResMut<CombatWheelAdapter>,
) {
    adapter.heat_current = heat.heat;
    adapter.heat_maximum = 100.0;
    adapter.heat_warning_threshold = 60.0;
    adapter.heat_critical_threshold = 85.0;
    adapter.heat_locked = heat.heat >= 85.0;
}

/// Forwards `PlayerDamagedEvent` into `CombatWheelEvent::ShieldDamaged` with
/// the damage direction derived from the source position relative to the
/// player. Also updates `adapter.last_damage_direction` so the bind systems
/// that read it (e.g. `animate_shield_surge`) have correct data.
///
/// Falls back to zero direction if either position is unavailable.
pub fn forward_damage_events(
    mut events: EventReader<PlayerDamagedEvent>,
    player_query: Query<&Transform, With<Player>>,
    mut adapter: ResMut<CombatWheelAdapter>,
    mut outgoing: EventWriter<CombatWheelEvent>,
) {
    let player_pos = player_query
        .get_single()
        .ok()
        .map(|t| t.translation.truncate());

    for event in events.read() {
        let direction = match player_pos {
            Some(p) => {
                let v = event.source_position - p;
                if v.length_squared() > 1e-6 {
                    v.normalize()
                } else {
                    Vec2::ZERO
                }
            }
            None => Vec2::ZERO,
        };
        adapter.last_damage_direction = Some(direction);
        outgoing.send(CombatWheelEvent::ShieldDamaged {
            amount: event.damage,
            direction,
        });
    }
}

/// Syncs `FactionSkin` from `ActiveModule.player_faction`. Runs every
/// `Update`; the guard ensures we only write when the campaign is active and
/// the player faction is set. Unknown factions leave the skin untouched.
pub fn sync_faction_skin_from_active_module(
    active: Res<ActiveModule>,
    mut skin: ResMut<FactionSkin>,
) {
    let Some(player_faction) = active.player_faction.as_deref() else {
        return;
    };
    let Some(target) = FactionSkin::from_player_faction(player_faction) else {
        return;
    };
    if *skin != target {
        *skin = target;
    }
}

/// Bundle type re-export so the campaign plugin can `add_systems` cleanly.
pub struct CombatWheelBindPlugin;

impl Plugin for CombatWheelBindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                project_ship_stats_to_combat_wheel,
                project_combo_heat_to_combat_wheel,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (forward_damage_events, sync_faction_skin_from_active_module),
        );
    }
}
