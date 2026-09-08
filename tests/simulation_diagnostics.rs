//! State hashing remains available for tests without taxing ordinary gameplay.

use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::GameState;
use rebellion::simulation::sim_id::SimId;
use rebellion::simulation::state_hash::SimStateHash;
use rebellion::simulation::SimulationDiagnostics;

#[test]
fn instrumentation_can_be_disabled_and_explicitly_enabled() {
    let mut app = build_headless_app();
    assert!(app.world().resource::<SimulationDiagnostics>().enabled);
    app.world_mut()
        .resource_mut::<SimulationDiagnostics>()
        .enabled = false;
    let probe = app.world_mut().spawn(Transform::default()).id();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    for _ in 0..3 {
        app.update();
    }
    assert!(app.world().get::<SimId>(probe).is_none());
    assert_eq!(app.world().resource::<SimStateHash>().0, 0);

    app.world_mut()
        .resource_mut::<SimulationDiagnostics>()
        .enabled = true;
    app.update();
    assert!(app.world().get::<SimId>(probe).is_some());
    let recorded = app.world().resource::<SimStateHash>().0;
    assert_ne!(recorded, 0);

    app.world_mut()
        .resource_mut::<SimulationDiagnostics>()
        .enabled = false;
    let later_probe = app
        .world_mut()
        .spawn(Transform::from_xyz(1.0, 2.0, 3.0))
        .id();
    app.update();
    assert!(app.world().get::<SimId>(later_probe).is_none());
    assert_eq!(app.world().resource::<SimStateHash>().0, recorded);

    app.world_mut()
        .resource_mut::<SimulationDiagnostics>()
        .enabled = true;
    app.update();
    assert!(app.world().get::<SimId>(later_probe).is_some());
    assert_ne!(app.world().resource::<SimStateHash>().0, recorded);
}
