// Quick gamepad diagnostic — print every button/axis event
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Gamepad Test".into(),
                resolution: (400.0, 300.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, print_gamepad_input)
        .run();
}

fn print_gamepad_input(gamepads: Query<&Gamepad>) {
    for gamepad in gamepads.iter() {
        // Check all buttons
        for button in [
            GamepadButton::South, GamepadButton::East, 
            GamepadButton::West, GamepadButton::North,
            GamepadButton::LeftTrigger, GamepadButton::RightTrigger,
            GamepadButton::LeftTrigger2, GamepadButton::RightTrigger2,
            GamepadButton::Select, GamepadButton::Start,
            GamepadButton::LeftThumb, GamepadButton::RightThumb,
            GamepadButton::DPadUp, GamepadButton::DPadDown,
            GamepadButton::DPadLeft, GamepadButton::DPadRight,
        ] {
            if gamepad.just_pressed(button) {
                println!("BUTTON PRESSED: {:?}", button);
            }
        }
        
        // Check axes with deadzone
        let lx = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
        let ly = gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0);
        let rx = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
        let ry = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);
        let lt = gamepad.get(GamepadAxis::LeftZ).unwrap_or(0.0);
        let rt = gamepad.get(GamepadAxis::RightZ).unwrap_or(0.0);
        
        if lx.abs() > 0.15 || ly.abs() > 0.15 {
            println!("LEFT STICK: x={:.2} y={:.2}", lx, ly);
        }
        if rx.abs() > 0.15 || ry.abs() > 0.15 {
            println!("RIGHT STICK: x={:.2} y={:.2}", rx, ry);
        }
        if lt.abs() > 0.1 {
            println!("LEFT TRIGGER: {:.2}", lt);
        }
        if rt.abs() > 0.1 {
            println!("RIGHT TRIGGER: {:.2}", rt);
        }
    }
}
