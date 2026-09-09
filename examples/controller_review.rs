//! Native visual fixture: missing-controller prompt and compact control guide.
//! Synthetic Gamepad input is explicitly confined to this development example.
use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};
use rebellion::{app_builder::RebellionAppConfig, core::GameState};
use std::path::PathBuf;
mod support;

#[derive(Resource)]
struct Review {
    output: PathBuf,
    seconds: f32,
    phase: u8,
}

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .expect("supply an output directory");
    std::fs::create_dir_all(&output).unwrap();
    let profile = std::env::temp_dir().join(format!(
        "rebellion-controller-review-{}",
        std::process::id()
    ));
    std::fs::create_dir(&profile).unwrap();
    std::fs::write(profile.join("save.json"), r#"{"stage_progress":[],"unlocked_ships":[],"lifetime_credits":0,"high_scores":[],"settings":{"master_volume":0,"music_volume":0,"sfx_volume":0}}"#).unwrap();
    std::env::set_var("REBELLION_HOME", profile);
    let mut app = RebellionAppConfig::native().build();
    support::install_controller(&mut app, 4.0);
    app.insert_resource(Review {
        output,
        seconds: 0.0,
        phase: 0,
    })
    .add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    })
    .add_systems(PostUpdate, review)
    .run();
}

fn review(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut review: ResMut<Review>,
    mut next: ResMut<NextState<GameState>>,
    mut exit: EventWriter<AppExit>,
) {
    review.seconds += time.delta_secs();
    match review.phase {
        0 if review.seconds > 3.0 => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(review.output.join("connection.png")));
            review.phase = 1;
        }
        1 if review.seconds > 7.0 => {
            next.set(GameState::Controls);
            review.phase = 2;
        }
        2 if review.seconds > 9.0 => {
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(review.output.join("controls.png")));
            review.phase = 3;
        }
        3 if review.seconds > 11.0 => {
            exit.send(AppExit::Success);
            review.phase = 4;
        }
        _ => {}
    }
}
