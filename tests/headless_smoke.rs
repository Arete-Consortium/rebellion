//! Headless smoke test
//!
//! Builds the headless app and runs a few ticks without panicking.

use rebellion::app_builder::build_headless_app;

#[test]
fn headless_app_runs_without_panic() {
    let mut app = build_headless_app();

    // Run a few updates to exercise startup + fixed update + update schedules
    for _ in 0..10 {
        app.update();
    }
}
