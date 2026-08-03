//! UI Systems
//!
//! HUD, menus, and visual feedback.

pub mod backgrounds;
pub mod capacitor;
pub mod combat_wheel;
pub mod hud;
pub mod menu;
pub mod transitions;

pub use backgrounds::*;
pub use capacitor::*;
pub use combat_wheel::*;
pub use hud::*;
pub use menu::*;
pub use transitions::*;

use bevy::prelude::*;

/// Plugin that registers all UI systems
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            HudPlugin,
            MenuPlugin,
            CapacitorWheelPlugin,
            BackgroundPlugin,
            TransitionPlugin,
            CombatWheelPlugin,
        ));
    }
}
