//! Platform Plugin
//!
//! Platform-specific input, window, and runtime integration:
//! keyboard, gamepad, touch, WASM bindings, native window.
//!
//! Polls raw input and reduces it into game-intent structures consumed by
//! gameplay and simulation systems.

use bevy::prelude::*;

use crate::core::{GameState, InputConfig};
use crate::systems::joystick::{JoystickPlugin, JoystickState};
pub mod controller;

/// Plugin that registers all platform input and runtime systems.
pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputConfig>()
            .init_resource::<crate::systems::touch_joystick::MobileMode>()
            .add_plugins((JoystickPlugin, controller::ControllerOnlyPlugin));
    }
}

/// System that triggers pause when ESC or Start button is pressed during gameplay
fn pause_trigger_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || joystick.just_pressed(9) {
        next_state.set(GameState::Paused);
    }
}
