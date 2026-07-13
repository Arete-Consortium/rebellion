//! Entity Components and Bundles
//!
//! All game entities: player, enemies, projectiles, collectibles, etc.

pub mod boss;
pub mod collectible;
pub mod drone;
pub mod enemy;
pub mod items;
pub mod player;
pub mod projectile;
pub mod wingman;

pub use boss::*;
pub use collectible::*;
pub use drone::*;
pub use enemy::*;
pub use items::{EffectiveStats, Inventory};
pub use player::*;
pub use projectile::*;
pub use wingman::*;

use bevy::prelude::*;

/// Plugin that registers entity systems that have not yet been migrated.
pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ProjectilePlugin, CollectiblePlugin));
    }
}
