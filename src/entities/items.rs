//! Persistent stacking weapon-mod system
//!
//! Ported from items_mvp. Collectibles of the weapon-mod kind (ScatterLauncher,
//! RailSpike, PlasmaLance, HomingSwarm, VortonProjector) stack additively in the
//! player's Inventory until ship death. `recompute_stats` folds the stacks into
//! EffectiveStats, which `player_shooting` reads at fire time.

#![allow(dead_code)]

use crate::core::events::CollectibleType;
use crate::core::WeaponType;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════
// Registry
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stat {
    Damage,
    FireRate,
    ProjectileCount,
    Spread,
    Pierce,
    CritChance,
    CritDamage,
    HomingStrength,
    ChainTargets,
    BurnDoT,
}

#[derive(Clone, Copy, Debug)]
pub enum StackCurve {
    Linear(f32),
    Diminishing(f32),
    Capped(f32, u8),
}

pub fn eval_curve(c: StackCurve, n: u32) -> f32 {
    match c {
        StackCurve::Linear(v) => v * n as f32,
        StackCurve::Diminishing(v) => v * (1.0 - 0.5_f32.powi(n as i32)),
        StackCurve::Capped(v, cap) => v * n.min(cap as u32) as f32,
    }
}

/// EVE-authentic display name for each weapon-mod pickup.
pub fn display_name(t: CollectibleType) -> &'static str {
    use CollectibleType as C;
    match t {
        C::ScatterLauncher => "Scourge Rage Missile",
        C::RailSpike => "Republic Fleet Barrage",
        C::PlasmaLance => "Conflagration Pulse",
        C::HomingSwarm => "Warrior Drone Swarm",
        C::VortonProjector => "Vorton Projector",
        _ => "",
    }
}

/// Which weapon family this mod attaches to. Stacks are inert unless the
/// player's Weapon matches this WeaponType.
pub fn required_weapon_type(t: CollectibleType) -> Option<WeaponType> {
    use CollectibleType as C;
    Some(match t {
        C::ScatterLauncher => WeaponType::MissileLauncher, // Caldari
        C::RailSpike => WeaponType::Autocannon,            // Minmatar
        C::PlasmaLance => WeaponType::Laser,               // Amarr
        C::HomingSwarm => WeaponType::Drone,               // Gallente
        C::VortonProjector => WeaponType::Vorton,          // EDENCOM
        _ => return None,
    })
}

/// Returns the stat effects for a weapon-mod collectible, or None if not a mod.
pub fn mod_effects(t: CollectibleType) -> Option<&'static [(Stat, StackCurve)]> {
    use CollectibleType as C;
    Some(match t {
        C::ScatterLauncher => &[
            (Stat::ProjectileCount, StackCurve::Capped(1.0, 5)),
            (Stat::Spread, StackCurve::Linear(0.09)),
            (Stat::Damage, StackCurve::Diminishing(-0.10)),
        ],
        C::RailSpike => &[
            (Stat::Damage, StackCurve::Linear(0.18)),
            (Stat::FireRate, StackCurve::Diminishing(-0.25)),
            (Stat::Pierce, StackCurve::Capped(1.0, 3)),
        ],
        C::PlasmaLance => &[
            (Stat::Damage, StackCurve::Linear(0.12)),
            (Stat::CritChance, StackCurve::Capped(0.05, 6)),
            (Stat::BurnDoT, StackCurve::Linear(2.0)),
        ],
        C::HomingSwarm => &[
            (Stat::ProjectileCount, StackCurve::Capped(1.0, 4)),
            (Stat::HomingStrength, StackCurve::Diminishing(0.30)),
            (Stat::Damage, StackCurve::Diminishing(-0.12)),
        ],
        C::VortonProjector => &[
            (Stat::ChainTargets, StackCurve::Capped(1.0, 4)),
            (Stat::Damage, StackCurve::Linear(0.08)),
        ],
        _ => return None,
    })
}

// ═══════════════════════════════════════════════════════════════
// Components
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Copy)]
pub struct ItemStack {
    pub id: CollectibleType,
    pub count: u32,
}

#[derive(Component, Default)]
pub struct Inventory {
    pub stacks: Vec<ItemStack>,
    pub dirty: bool,
}

impl Inventory {
    pub fn add(&mut self, id: CollectibleType) {
        match self.stacks.iter_mut().find(|s| s.id == id) {
            Some(s) => s.count += 1,
            None => self.stacks.push(ItemStack { id, count: 1 }),
        }
        self.dirty = true;
    }
}

/// Derived stats folded from Inventory stacks each frame (when dirty).
/// Zeroed deltas — player's baseline Weapon component values are multiplied/added on top.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct EffectiveStats {
    pub damage_mult: f32,    // 1.0 = no change
    pub fire_rate_mult: f32, // 1.0 = no change
    pub extra_projectiles: u32,
    pub spread_radians: f32,
    pub pierce: u32,
    pub crit_chance: f32,
    pub crit_damage_mult: f32, // 1.0 = no bonus
    pub homing: f32,           // 0.0..1.0
    pub chain_targets: u32,
    pub burn_dps: f32,
}

impl EffectiveStats {
    pub fn neutral() -> Self {
        Self {
            damage_mult: 1.0,
            fire_rate_mult: 1.0,
            extra_projectiles: 0,
            spread_radians: 0.0,
            pierce: 0,
            crit_chance: 0.0,
            crit_damage_mult: 1.0,
            homing: 0.0,
            chain_targets: 0,
            burn_dps: 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// Recompute
// ═══════════════════════════════════════════════════════════════

pub fn recompute_stats(
    mut q: Query<(&mut Inventory, &mut EffectiveStats, &super::player::Weapon)>,
) {
    for (mut inv, mut eff, weapon) in &mut q {
        if !inv.dirty {
            continue;
        }

        let mut mu_dmg = 0.0_f32;
        let mut mu_fire = 0.0_f32;
        let mut fl_proj = 0.0_f32;
        let mut fl_spread = 0.0_f32;
        let mut fl_pierce = 0.0_f32;
        let mut mu_crit = 0.0_f32;
        let mut mu_crit_dmg = 0.0_f32;
        let mut mu_homing = 0.0_f32;
        let mut fl_chain = 0.0_f32;
        let mut fl_burn = 0.0_f32;

        for stack in &inv.stacks {
            // Skip mods that don't match this ship's weapon family
            if let Some(req) = required_weapon_type(stack.id) {
                if weapon.weapon_type != req {
                    continue;
                }
            }
            if let Some(effects) = mod_effects(stack.id) {
                for (stat, curve) in effects {
                    let d = eval_curve(*curve, stack.count);
                    match stat {
                        Stat::Damage => mu_dmg += d,
                        Stat::FireRate => mu_fire += d,
                        Stat::ProjectileCount => fl_proj += d,
                        Stat::Spread => fl_spread += d,
                        Stat::Pierce => fl_pierce += d,
                        Stat::CritChance => mu_crit += d,
                        Stat::CritDamage => mu_crit_dmg += d,
                        Stat::HomingStrength => mu_homing += d,
                        Stat::ChainTargets => fl_chain += d,
                        Stat::BurnDoT => fl_burn += d,
                    }
                }
            }
        }

        *eff = EffectiveStats {
            damage_mult: (1.0 + mu_dmg).max(0.1),
            fire_rate_mult: (1.0 + mu_fire).max(0.1),
            extra_projectiles: fl_proj.max(0.0) as u32,
            spread_radians: fl_spread,
            pierce: fl_pierce.max(0.0) as u32,
            crit_chance: mu_crit.clamp(0.0, 1.0),
            crit_damage_mult: 1.0 + mu_crit_dmg,
            homing: mu_homing.clamp(0.0, 1.0),
            chain_targets: fl_chain.max(0.0) as u32,
            burn_dps: fl_burn.max(0.0),
        };

        inv.dirty = false;
    }
}
