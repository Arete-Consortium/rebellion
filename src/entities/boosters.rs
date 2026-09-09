//! Run-local doses of timed combat boosters. Repairs remain immediate pickups.
use super::{Player, PowerupEffects};
use crate::{core::CollectibleType, systems::JoystickState};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoosterKind {
    Overclocker,
    Pyrolancea,
    XInstinct,
}

impl BoosterKind {
    pub const ALL: [Self; 3] = [Self::Overclocker, Self::Pyrolancea, Self::XInstinct];

    pub fn from_pickup(kind: CollectibleType) -> Option<Self> {
        match kind {
            CollectibleType::Overdrive => Some(Self::Overclocker),
            CollectibleType::DamageBoost => Some(Self::Pyrolancea),
            CollectibleType::Invulnerability => Some(Self::XInstinct),
            _ => None,
        }
    }

    pub fn pickup(self) -> CollectibleType {
        match self {
            Self::Overclocker => CollectibleType::Overdrive,
            Self::Pyrolancea => CollectibleType::DamageBoost,
            Self::XInstinct => CollectibleType::Invulnerability,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Overclocker => "OVERCLOCKER",
            Self::Pyrolancea => "PYROLANCEA",
            Self::XInstinct => "X-INSTINCT",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Overclocker => "Speed x1.5 / 5s",
            Self::Pyrolancea => "Damage x2 / 10s",
            Self::XInstinct => "Invulnerable / 3s",
        }
    }

    pub fn remaining(self, effects: &PowerupEffects) -> f32 {
        match self {
            Self::Overclocker => effects.overdrive_timer,
            Self::Pyrolancea => effects.damage_boost_timer,
            Self::XInstinct => effects.invuln_timer,
        }
    }

    fn activate(self, effects: &mut PowerupEffects) {
        match self {
            Self::Overclocker => effects.overdrive_timer = 5.0,
            Self::Pyrolancea => effects.damage_boost_timer = 10.0,
            Self::XInstinct => effects.invuln_timer = 3.0,
        }
    }
}

/// Persists through mission continuation, boss transitions and pause. Not saved
/// to disk: death, a new run or an explicit restart discards unused doses.
#[derive(Resource, Clone, Debug, Default)]
pub struct BoosterInventory {
    counts: [u8; 3],
    selected: usize,
}

impl BoosterInventory {
    pub const CAPACITY_PER_KIND: u8 = 3;

    pub fn count(&self, kind: BoosterKind) -> u8 {
        self.counts[kind as usize]
    }

    pub fn selected(&self) -> BoosterKind {
        BoosterKind::ALL[self.selected]
    }

    pub fn is_empty(&self) -> bool {
        self.counts.iter().all(|&count| count == 0)
    }

    /// Returns false when full, allowing the pickup to stay in space.
    pub fn store(&mut self, kind: BoosterKind) -> bool {
        let index = kind as usize;
        if self.counts[index] >= Self::CAPACITY_PER_KIND {
            return false;
        }
        if self.is_empty() {
            self.selected = index;
        }
        self.counts[index] += 1;
        true
    }

    /// Select the next occupied slot, wrapping in either direction.
    pub fn cycle(&mut self, forward: bool) {
        for offset in 1..=BoosterKind::ALL.len() {
            let index = if forward {
                (self.selected + offset) % BoosterKind::ALL.len()
            } else {
                (self.selected + BoosterKind::ALL.len() - offset) % BoosterKind::ALL.len()
            };
            if self.counts[index] > 0 {
                self.selected = index;
                break;
            }
        }
    }

    /// Empty or already-active doses are never spent. Different effects may
    /// coexist; pressing again cannot stack or refresh an active effect.
    pub fn use_selected(&mut self, effects: &mut PowerupEffects) -> Option<BoosterKind> {
        let kind = self.selected();
        if self.count(kind) == 0 || kind.remaining(effects) > 0.0 {
            return None;
        }
        self.counts[self.selected] -= 1;
        kind.activate(effects);
        Some(kind)
    }
}

#[derive(Event, Clone, Copy)]
pub struct BoosterActivatedEvent(pub BoosterKind);

pub(crate) fn reset_boosters(mut inventory: ResMut<BoosterInventory>) {
    *inventory = BoosterInventory::default();
}

pub(crate) fn handle_booster_input(
    input: Res<JoystickState>,
    mut inventory: ResMut<BoosterInventory>,
    mut player: Query<&mut PowerupEffects, With<Player>>,
    mut activated: EventWriter<BoosterActivatedEvent>,
    mut rumble: EventWriter<crate::systems::RumbleRequest>,
) {
    let Ok(mut effects) = player.get_single_mut() else {
        return;
    };
    if input.dpad_just_up() {
        inventory.cycle(false);
    } else if input.dpad_just_down() {
        inventory.cycle(true);
    }
    if input.y_button() {
        if let Some(kind) = inventory.use_selected(&mut effects) {
            activated.send(BoosterActivatedEvent(kind));
            rumble.send(crate::systems::RumbleRequest::powerup());
        }
    }
}
