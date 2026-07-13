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
use crate::systems::touch_joystick::TouchJoystickPlugin;

/// Plugin that registers all platform input and runtime systems.
pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputConfig>()
            .add_plugins((JoystickPlugin, TouchJoystickPlugin))
            // Pause system — ESC or Start button during gameplay triggers pause
            .add_systems(
                Update,
                pause_trigger_system
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
            );
    }
}

/// System that triggers pause when ESC or Start button is pressed during gameplay
fn pause_trigger_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || joystick.start() {
        next_state.set(GameState::Paused);
    }
}
