use super::components::*;
use bevy::prelude::*;

/// Cross-system events emitted by gameplay and consumed by the Combat Wheel HUD.
///
/// Mirrors the prototype `CombatWheelEvent` shape (Bevy 0.14) but kept inside
/// the new combat_wheel module path. Adapter systems project
/// `PlayerDamagedEvent` / `DamageLayerEvent` from the real game into these.
#[derive(Event, Clone, Debug)]
pub enum CombatWheelEvent {
    ModuleActivated {
        slot_id: ModuleSlotId,
    },
    ModuleRejected {
        slot_id: ModuleSlotId,
        reason: RejectionReason,
    },
    ModuleCooledDown {
        slot_id: ModuleSlotId,
    },
    ModuleStateChanged {
        slot_id: ModuleSlotId,
        new_state: ModuleVisualState,
    },
    PowerupAcquired {
        effect: PowerupEffect,
    },
    PowerupExpiring {
        effect: PowerupEffect,
        remaining_secs: f32,
    },
    PowerupExpired {
        effect: PowerupEffect,
    },
    ShieldDamaged {
        amount: f32,
        direction: Vec2,
    },
    ShieldCollapsed,
    ShieldRecharged,
    IntegrityDamaged {
        amount: f32,
    },
    ArmorBreached,
    IntegrityCritical,
    CapacitorDrained {
        amount: f32,
    },
    CapacitorReplenished,
    InsufficientCapacitor {
        attempted_action: ModuleSlotId,
    },
    HeatWarning,
    HeatCritical,
    Overheated,
    CoolingComplete,
    SentryDeployed {
        count: u32,
    },
    SentryDisabled {
        count: u32,
    },
    SentryNetworkDegraded,
    SentryNetworkRestored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReason {
    InsufficientCapacitor,
    Overheated,
    OutOfCharges,
    Disabled,
    Locked,
    Cooldown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerupEffect {
    FireRate,
    HeatReduction,
    ShieldRecharge,
    PiercingAmmo,
    MissileSwarm,
    Invulnerability,
    ScoreMultiplier,
}
