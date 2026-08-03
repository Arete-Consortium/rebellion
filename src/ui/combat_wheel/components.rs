#![allow(dead_code)] // Public API surface; fully wired in Phase 2 (event forwarding + layout).

use bevy::prelude::*;

/// Root entity of the Combat Wheel.
#[derive(Component)]
pub struct CombatWheel;

/// Shield ring root. Children are `ShieldSegment` entities.
#[derive(Component)]
pub struct ShieldRing;

/// Per-segment shield state.
#[derive(Component, Clone, Copy, Default)]
pub struct ShieldSegment {
    pub index: usize,
    pub health: f32,
    pub max_health: f32,
    pub state: SegmentState,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SegmentState {
    #[default]
    Healthy,
    Damaged,
    Recharging,
    Collapsed,
}

/// Temporary surge animation triggered by damage from a specific direction.
#[derive(Component)]
pub struct ShieldSurge {
    pub origin_angle: f32,
    pub intensity: f32,
    pub decay: f32,
}

/// Integrity ring root. Children are `IntegritySegment` entities.
#[derive(Component)]
pub struct IntegrityRing;

/// Armor is texture state, not a separate ring.
#[derive(Component, Clone, Copy, Default)]
pub struct IntegritySegment {
    pub index: usize,
    pub health: f32,
    pub armor: f32,
    pub max_health: f32,
    pub max_armor: f32,
    pub state: IntegrityVisual,
}

// Renamed to `IntegrityVisual` to avoid name collision with the prototype's
// gameplay-side `IntegrityState` Resource. Visual state only.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum IntegrityVisual {
    #[default]
    Pristine,
    Fractured,
    Breached,
    Critical,
    Destroyed,
}

/// Center capacitor / energy core.
#[derive(Component)]
pub struct CapacitorCore;

/// Heat arc root. Children are `HeatSegment` entities.
#[derive(Component)]
pub struct HeatArc;

#[derive(Component)]
pub struct HeatSegment {
    pub index: usize,
}

/// Module slot surrounding the wheel.
#[derive(Component, Default)]
pub struct ModuleSlot {
    pub slot_id: ModuleSlotId,
    pub category: ModuleCategory,
    pub visual_state: ModuleVisualState,
    pub cooldown_normalized: f32,
    pub heat_normalized: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ModuleSlotId {
    #[default]
    PrimaryWeapon,
    SecondaryWeapon,
    Propulsion,
    Defense,
    Ability,
    Deployable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ModuleCategory {
    #[default]
    PrimaryWeapon,
    SecondaryWeapon,
    Propulsion,
    Defense,
    Tactical,
    Deployable,
    Powerup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ModuleVisualState {
    #[default]
    Ready,
    Focused,
    Active,
    Cycling,
    CoolingDown,
    Reloading,
    Heating,
    Overheated,
    Disabled,
    Jammed,
    OutOfEnergy,
    OutOfCharges,
    Locked,
}

/// Dynamic input glyph displayed on the module slot.
#[derive(Component)]
pub struct ModuleInputGlyph {
    pub binding: InputBinding,
}

/// Marker for a numeric percentage label (accessibility).
#[derive(Component)]
pub struct PercentageLabel {
    pub stat: StatType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatType {
    Shield,
    Integrity,
    Capacitor,
    Heat,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum InputBinding {
    Keyboard(KeyCode),
    Mouse(MouseButton),
    Gamepad(GamepadButton),
    #[default]
    Unbound,
}
