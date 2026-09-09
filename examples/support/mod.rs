//! Synthetic input for native rendering fixtures. Never linked into the game.
use bevy::prelude::*;
use rebellion::systems::JoystickState;

#[derive(Resource)]
struct ScriptedController {
    delay: f32,
    seconds: f32,
    entity: Option<Entity>,
    frames: u8,
}

pub fn install_controller(app: &mut App, delay: f32) {
    app.insert_resource(ScriptedController {
        delay,
        seconds: 0.0,
        entity: None,
        frames: 0,
    })
    .add_systems(First, acknowledge_connection.after(bevy::time::TimeSystem));
}

fn acknowledge_connection(
    mut commands: Commands,
    mut fixture: ResMut<ScriptedController>,
    time: Res<Time<Real>>,
    mut pads: Query<&mut Gamepad>,
    mut input: ResMut<JoystickState>,
) {
    fixture.seconds += time.delta_secs();
    if fixture.entity.is_none() && fixture.seconds >= fixture.delay {
        let entity = commands
            .spawn((
                Gamepad::default(),
                Name::new("Rebellion scripted controller fixture"),
            ))
            .id();
        fixture.entity = Some(entity);
        input.active_gamepad = Some(entity);
    } else if let Some(entity) = fixture.entity {
        if let Ok(mut pad) = pads.get_mut(entity) {
            fixture.frames = fixture.frames.saturating_add(1);
            if fixture.frames == 2 {
                pad.digital_mut().press(GamepadButton::South);
            }
            if fixture.frames == 3 {
                pad.digital_mut().reset_all();
            }
        }
    }
}
