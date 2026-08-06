//! Integration tests for the Options menu slider parity pass (Phase 7).
//!
//! Two layers of coverage:
//!
//! - **Resource-level round-trip** (Group B/C): mutate `SoundSettings`,
//!   `ScreenShake.multiplier`, `RumbleSettings.intensity` through the
//!   Bevy resource API and confirm the next update tick propagates the
//!   change into `SaveData.settings`. The headless build registers
//!   `SavePlugin` (`src/app_builder.rs:165-198`), so
//!   `sync_settings_to_save` runs after each `app.update()`.
//!
//! - **Source-level guards** (Group A/D): `include_str!` checks of
//!   `options.rs` that pin the wiring code. The headless build does
//!   not include `MenuPlugin`, so the spawn system and
//!   `options_menu_input` never run during integration tests. The
//!   guards are the only way to verify the new rows / reset affordance
//!   exist in the source code at all.

use bevy::prelude::*;
use rebellion::app_builder::build_headless_app;
use rebellion::core::{GameSettings, SaveData};
use rebellion::systems::audio::SoundSettings;
use rebellion::systems::effects::screen_effects::ScreenShake;
use rebellion::systems::joystick::{RumbleRequest, RumbleSettings, RumbleType};

const OPTIONS_SRC: &str = include_str!("../src/ui/menu/options.rs");
const SAVE_SRC: &str = include_str!("../src/core/save.rs");

// ============================================================================
// Setup helpers
// ============================================================================

/// Build a headless app with the three persisted settings resources
/// and a default `SaveData` seeded explicitly. The headless build
/// registers all three via `configure_headless_plugins`, but the
/// explicit insert makes these tests robust to registration-order
/// changes elsewhere.
fn setup_app_with_settings() -> App {
    let mut app = build_headless_app();
    app.insert_resource(SoundSettings::default());
    app.insert_resource(ScreenShake::default());
    app.insert_resource(RumbleSettings::default());
    app.insert_resource(SaveData::default());
    app
}

// ============================================================================
// Group A — Source-level guards for the Options menu
// ============================================================================

/// `OptionsMenuState.total` must be 7 — covers Master, Music, Sfx,
/// Shake, Rumble, RESET, CONTROLS.
#[test]
fn options_total_reflects_seven_rows() {
    assert!(
        OPTIONS_SRC.contains("total: 7"),
        "OptionsMenuState.default.total must be 7 (5 sliders + RESET + CONTROLS)"
    );
}

/// The `SliderSetting` enum must expose all five persisted settings.
#[test]
fn options_exposes_all_five_slider_settings() {
    for variant in [
        "SliderSetting::Master",
        "SliderSetting::Music",
        "SliderSetting::Sfx",
        "SliderSetting::Shake",
        "SliderSetting::Rumble",
    ] {
        assert!(
            OPTIONS_SRC.contains(variant),
            "options.rs must reference `{variant}`"
        );
    }
}

/// The new sliders must write through the matching Bevy resources
/// (`ScreenShake.multiplier` and `RumbleSettings.intensity`), not
/// stash values somewhere unreachable.
#[test]
fn options_writes_shake_and_rumble_through_resources() {
    assert!(
        OPTIONS_SRC.contains("screen_shake.multiplier"),
        "options.rs must mutate ScreenShake.multiplier directly (it is the persisted field)"
    );
    assert!(
        OPTIONS_SRC.contains("rumble.intensity"),
        "options.rs must mutate RumbleSettings.intensity directly (it is the persisted field)"
    );
}

/// The RESET row marker + confirm handler must exist.
#[test]
fn options_handles_reset_nav_row() {
    assert!(
        OPTIONS_SRC.contains("pub(crate) struct ResetNavItem"),
        "options.rs must define a ResetNavItem marker component"
    );
    assert!(
        OPTIONS_SRC.contains("state.selected == 5") && OPTIONS_SRC.contains("is_confirm"),
        "options.rs must route a confirm press on row 5 (RESET) through is_confirm()"
    );
    assert!(
        OPTIONS_SRC.contains("SoundSettings::default()")
            && OPTIONS_SRC.contains("ScreenShake::default()")
            && OPTIONS_SRC.contains("RumbleSettings::default()"),
        "RESET must call Default::default() on all three persisted resources"
    );
}

/// The CONTROLS row sits at index 6 (was 3 before the parity pass).
#[test]
fn options_controls_row_is_at_index_six() {
    assert!(
        OPTIONS_SRC.contains("state.selected == 6"),
        "the CONTROLS confirm gate must check state.selected == 6"
    );
}

/// The slider adjust guard must cover all five slider rows (0..=4)
/// and reject nav-row edits at indices 5 and 6.
#[test]
fn options_adjust_guard_covers_five_sliders_only() {
    assert!(
        OPTIONS_SRC.contains("state.selected < 5"),
        "the left/right adjust branch must only fire on indices 0..=4"
    );
}

/// `options.rs` must not write directly to `save.settings.*` — it
/// must go through the resource layer so `sync_settings_to_save`
/// can pick up the change. This is a regression guard against
/// "shortcut" code that bypasses the persistence layer.
#[test]
fn options_does_not_bypass_save_layer() {
    let bypasses = [
        "save.settings.master_volume",
        "save.settings.sfx_volume",
        "save.settings.music_volume",
        "save.settings.screen_shake_intensity",
        "save.settings.rumble_intensity",
    ];
    for needle in bypasses {
        assert!(
            !OPTIONS_SRC.contains(needle),
            "options.rs must not write directly to `{needle}` — go through the resource"
        );
    }
}

/// The spawn function must take the two new resource arguments.
#[test]
fn options_spawn_takes_all_three_settings_resources() {
    assert!(
        OPTIONS_SRC.contains("sound_settings: Res<crate::systems::audio::SoundSettings>")
            && OPTIONS_SRC.contains("screen_shake: Res<crate::systems::effects::screen_effects::ScreenShake>")
            && OPTIONS_SRC.contains("rumble: Res<crate::systems::joystick::RumbleSettings>"),
        "spawn_options_menu must take Res<SoundSettings>, Res<ScreenShake>, Res<RumbleSettings>"
    );
}

// ============================================================================
// Group B — Resource-level round-trip (headless, real Bevy schedules)
// ============================================================================

/// Mutating `SoundSettings.master_volume` on a real Bevy world
/// triggers `is_changed()` → `sync_settings_to_save` writes it to
/// `SaveData.settings.master_volume` on the next tick.
#[test]
fn sound_settings_master_propagates_to_save_data() {
    let mut app = setup_app_with_settings();

    // Run one frame first to let init systems fire — `apply_saved_settings`
    // runs in PostStartup and may overwrite the resource with the saved
    // value, so the first Update will see whatever SaveData already held.
    app.update();

    {
        let mut s = app.world_mut().resource_mut::<SoundSettings>();
        s.master_volume = 0.42;
    }
    app.update();

    let save = app.world().resource::<SaveData>();
    assert!(
        (save.settings.master_volume - 0.42).abs() < 0.001,
        "SoundSettings.master_volume must propagate to SaveData.settings.master_volume (got {})",
        save.settings.master_volume
    );
}

/// Same propagation for the `ScreenShake.multiplier` field. The save
/// key is `screen_shake_intensity` (the persisted field name); the
/// runtime field is `multiplier`. The mapping lives in
/// `sync_settings_to_save` (`src/core/save.rs:627-629`).
#[test]
fn screen_shake_multiplier_propagates_to_save_data() {
    let mut app = setup_app_with_settings();
    app.update(); // flush PostStartup apply_saved_settings

    {
        let mut s = app.world_mut().resource_mut::<ScreenShake>();
        s.multiplier = 0.33;
    }
    app.update();

    let save = app.world().resource::<SaveData>();
    assert!(
        (save.settings.screen_shake_intensity - 0.33).abs() < 0.001,
        "ScreenShake.multiplier must propagate to SaveData.settings.screen_shake_intensity"
    );
}

/// Same propagation for `RumbleSettings.intensity`.
#[test]
fn rumble_intensity_propagates_to_save_data() {
    let mut app = setup_app_with_settings();
    app.update(); // flush PostStartup apply_saved_settings

    {
        let mut r = app.world_mut().resource_mut::<RumbleSettings>();
        r.intensity = 0.5;
    }
    app.update();

    let save = app.world().resource::<SaveData>();
    assert!(
        (save.settings.rumble_intensity - 0.5).abs() < 0.001,
        "RumbleSettings.intensity must propagate to SaveData.settings.rumble_intensity"
    );
}

/// Stale `SaveData` is corrected on the next sync tick. This proves
/// the round-trip works in **both** directions: a saved value that
/// disagrees with the runtime resource gets overwritten when the
/// resource is touched.
#[test]
fn stale_save_data_is_corrected_by_sync() {
    let mut app = setup_app_with_settings();
    app.update(); // flush PostStartup apply_saved_settings

    // Seed a stale saved value that disagrees with the current
    // resource. Then touch the resource to force `is_changed()` on
    // this frame.
    app.world_mut()
        .resource_mut::<SaveData>()
        .settings
        .master_volume = 0.99;
    {
        let mut s = app.world_mut().resource_mut::<SoundSettings>();
        s.master_volume = 0.7; // same as default; the write fires is_changed()
    }
    app.update();

    let save = app.world().resource::<SaveData>();
    assert!(
        (save.settings.master_volume - 0.7).abs() < 0.001,
        "sync_settings_to_save must overwrite a stale saved master_volume"
    );
}

// ============================================================================
// Group C — Reset-to-defaults at the resource level
// ============================================================================

/// Mimic the RESET confirm handler — overwrite all three resources
/// with their defaults — and verify `SaveData.settings` reflects
/// the change on the next tick. This is the data-side counterpart
/// to the source-level `options_handles_reset_nav_row` guard.
#[test]
fn options_reset_restores_all_three_resources() {
    let mut app = setup_app_with_settings();
    app.update(); // flush PostStartup apply_saved_settings

    // First, set every persisted field to a non-default value.
    {
        let mut s = app.world_mut().resource_mut::<SoundSettings>();
        s.master_volume = 0.1;
        s.sfx_volume = 0.2;
        s.music_volume = 0.3;
    }
    {
        let mut s = app.world_mut().resource_mut::<ScreenShake>();
        s.multiplier = 0.0;
    }
    {
        let mut r = app.world_mut().resource_mut::<RumbleSettings>();
        r.intensity = 0.0;
    }
    app.update();
    // Sanity: the persisted values drifted from defaults.
    {
        let save = app.world().resource::<SaveData>();
        assert!((save.settings.master_volume - 0.1).abs() < 0.001);
        assert!((save.settings.screen_shake_intensity - 0.0).abs() < 0.001);
        assert!((save.settings.rumble_intensity - 0.0).abs() < 0.001);
    }

    // Now mimic the UI handler: overwrite each resource with its
    // canonical default. The sync system picks up the change on the
    // next tick and writes through to SaveData.
    {
        let mut s = app.world_mut().resource_mut::<SoundSettings>();
        *s = SoundSettings::default();
    }
    {
        let mut s = app.world_mut().resource_mut::<ScreenShake>();
        s.multiplier = ScreenShake::default().multiplier;
    }
    {
        let mut r = app.world_mut().resource_mut::<RumbleSettings>();
        *r = RumbleSettings::default();
    }
    app.update();

    let save = app.world().resource::<SaveData>();
    let defaults = GameSettings::default();
    assert!(
        (save.settings.master_volume - defaults.master_volume).abs() < 0.001,
        "RESET must restore SoundSettings.master_volume to its default"
    );
    assert!(
        (save.settings.sfx_volume - defaults.sfx_volume).abs() < 0.001,
        "RESET must restore SoundSettings.sfx_volume to its default"
    );
    assert!(
        (save.settings.music_volume - defaults.music_volume).abs() < 0.001,
        "RESET must restore SoundSettings.music_volume to its default"
    );
    assert!(
        (save.settings.screen_shake_intensity - defaults.screen_shake_intensity).abs() < 0.001,
        "RESET must restore ScreenShake.multiplier to its default"
    );
    assert!(
        (save.settings.rumble_intensity - defaults.rumble_intensity).abs() < 0.001,
        "RESET must restore RumbleSettings.intensity to its default"
    );
}

// ============================================================================
// Group D — Defaults shape + serde migration safety
// ============================================================================

/// `GameSettings::default()` must populate all five persisted fields
/// with the canonical values.
#[test]
fn game_settings_default_has_all_five_fields() {
    let g = GameSettings::default();
    assert!((g.master_volume - 0.7).abs() < f32::EPSILON);
    assert!((g.sfx_volume - 0.8).abs() < f32::EPSILON);
    assert!((g.music_volume - 0.5).abs() < f32::EPSILON);
    assert!((g.screen_shake_intensity - 1.0).abs() < f32::EPSILON);
    assert!((g.rumble_intensity - 1.0).abs() < f32::EPSILON);
}

/// A legacy save blob missing `screen_shake_intensity` and
/// `rumble_intensity` must deserialize cleanly. `#[serde(default)]`
/// on those two fields gives 1.0 for both.
#[test]
fn legacy_save_without_shake_or_rumble_loads_defaults() {
    // A pre-Phase-7 save blob — only the three audio fields, no
    // screen_shake_intensity / rumble_intensity.
    let blob = r#"{"master_volume":0.1,"sfx_volume":0.2,"music_volume":0.3}"#;
    let g: GameSettings = serde_json::from_str(blob).expect("legacy save must deserialize");

    assert!((g.screen_shake_intensity - 1.0).abs() < f32::EPSILON);
    assert!((g.rumble_intensity - 1.0).abs() < f32::EPSILON);
}

/// The save layer (`src/core/save.rs`) must continue to handle all
/// five fields end-to-end. Pinning the field names protects against
/// a future refactor that drops one of them.
#[test]
fn save_layer_round_trips_all_five_fields() {
    assert!(
        SAVE_SRC.contains("settings.master_volume")
            && SAVE_SRC.contains("settings.sfx_volume")
            && SAVE_SRC.contains("settings.music_volume")
            && SAVE_SRC.contains("settings.screen_shake_intensity")
            && SAVE_SRC.contains("settings.rumble_intensity"),
        "save.rs must touch every one of the five GameSettings fields in the round-trip"
    );

    // The 0.001 epsilon gate is the ping-pong guard.
    assert!(
        SAVE_SRC.contains("> 0.001"),
        "sync_settings_to_save must use the 0.001 epsilon gate to avoid redundant writes"
    );
}

// ============================================================================
// Group E — Layout literal guard
// ============================================================================

/// The on-screen RESET label is exactly `"RESET TO DEFAULTS"`. Pin
/// the literal so a future edit doesn't silently rename the
/// affordance.
#[test]
fn options_layout_pins_reset_label_literal() {
    assert!(
        OPTIONS_SRC.contains(r#"Text::new("RESET TO DEFAULTS")"#),
        "options.rs must render the literal 'RESET TO DEFAULTS' label"
    );
}

// ============================================================================
// Group F — Audio/haptic previews (Phase 10)
// ============================================================================

// Source-level guard: the SFX preview must spawn an AudioPlayer when
// the SFX slider is adjusted. Pins the wiring literally.
#[test]
fn options_sfx_preview_plays_menu_select() {
    assert!(
        OPTIONS_SRC.contains("AudioPlayer")
            && OPTIONS_SRC.contains("PlaybackMode::Despawn")
            && OPTIONS_SRC.contains("sounds.menu_select"),
        "options.rs must spawn an AudioPlayer(Handle) + PlaybackMode::Despawn for the menu_select sound when adjusting the SFX slider"
    );
}

// Source-level guard: the haptic preview must send a RumbleRequest with
// a Custom rumble type when the Rumble slider is adjusted.
#[test]
fn options_rumble_preview_fires_custom_request() {
    assert!(
        OPTIONS_SRC.contains("RumbleRequest")
            && OPTIONS_SRC.contains("RumbleType::Custom")
            && OPTIONS_SRC.contains("duration_ms"),
        "options.rs must send a RumbleRequest::new(RumbleType::Custom) on Rumble slider adjust"
    );
}

// Source-level guard: the preview triggers are gated on the SFX and
// Rumble slider indices, not the audio sliders (0/1/2) or nav rows
// (5/6). Master preview is intentionally absent.
#[test]
fn options_preview_targets_sfx_and_rumble_only() {
    assert!(
        OPTIONS_SRC.contains("current_setting == SliderSetting::Sfx")
            && OPTIONS_SRC.contains("current_setting == SliderSetting::Rumble"),
        "options.rs must only fire previews for Sfx and Rumble slider indices"
    );
    // Anti-pattern: no preview on Master.
    assert!(
        !OPTIONS_SRC.contains("current_setting == SliderSetting::Master"),
        "options.rs must NOT preview the Master slider (would mid-playback change its own volume)"
    );
}

// The SFX preview volume must be `sfx_volume * master_volume * 0.7`.
// At default settings (0.8 * 0.7 * 0.7) ≈ 0.392 — a comfortable UI blip.
// Without the `0.7` factor the preview would match the in-game SFX
// level exactly, which is louder than a menu confirmation should be.
#[test]
fn options_sfx_preview_volume_uses_combined_factor() {
    assert!(
        OPTIONS_SRC.contains("sfx_volume")
            && OPTIONS_SRC.contains("master_volume")
            && OPTIONS_SRC.contains("* 0.7"),
        "options.rs must compute SFX preview volume as sfx_volume * master_volume * 0.7"
    );
}

// ============================================================================
// Group G — Rumble round-trip (resource-level, headless)
// ============================================================================

// The RumbleRequest event flows through process_rumble_requests
// which scales by RumbleSettings.intensity. Sending a Custom
// RumbleRequest must not panic, regardless of intensity.
//
// (No gamepads exist in headless, so we cannot verify the actual
// GamepadRumbleRequest is sent. The point of this test is to confirm
// the system chain is registered end-to-end in the headless build —
// which only works if JoystickPlugin + Events<GamepadRumbleRequest>
// are both present.)
#[test]
fn rumble_request_event_chains_through_process_rumble_requests() {
    let mut app = build_headless_app();
    app.world_mut().resource_mut::<RumbleSettings>().intensity = 0.5;
    app.world_mut()
        .send_event(RumbleRequest::new(RumbleType::Custom {
            strong: 0.6,
            weak: 0.4,
            duration_ms: 100,
        }));

    // Run several frames to let the event flow through.
    for _ in 0..5 {
        app.update();
    }
}

// With intensity = 0.0, the early-return guard in
// process_rumble_requests must fire and skip the event. The test
// passes if `process_rumble_requests` does not panic on an empty
// gamepad list.
#[test]
fn rumble_request_with_zero_intensity_is_skipped() {
    let mut app = build_headless_app();
    app.world_mut().resource_mut::<RumbleSettings>().intensity = 0.0;
    app.world_mut()
        .send_event(RumbleRequest::new(RumbleType::Custom {
            strong: 1.0,
            weak: 1.0,
            duration_ms: 1000,
        }));

    // The early-return means the system doesn't read or clear
    // pending events. Run a frame; no panic.
    for _ in 0..3 {
        app.update();
    }
}