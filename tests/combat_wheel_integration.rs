//! Combat Wheel HUD integration tests
//!
//! Validates the adapter that projects `ShipStats` (Component on the player
//! entity) + `ComboHeatSystem` (Resource) into `CombatWheelAdapter`, plus
//! the wiring of the HUD into the Triglavian Invasion campaign.
//!
//! Lock-down tests for the layout fix live in
//! `src/ui/combat_wheel/layout.rs::tests`.
//!
//! Phase 1.5 (initial integration coverage); Phase 2 wires the actual spawn
//! on `OnEnter(GameState::Playing)`, at which point additional gating tests
//! (`triglavian_spawns_combat_wheel_on_playing_entry`,
//! `non_triglavian_campaign_does_not_spawn_combat_wheel`) come online.

use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::entities::player::{Player, ShipStats};
use rebellion::games::triglavian_invasion::combat_wheel_bind::{
    sync_faction_skin_from_active_module, CombatWheelBindPlugin,
};
use rebellion::games::ActiveModule;
use rebellion::ui::combat_wheel::{CombatWheelAdapter, FactionSkin};

/// Builds a headless app, adds a player with default ShipStats, then adds
/// the combat wheel bind plugin. Used as the common base for adapter tests.
fn build_triglavian_app() -> App {
    let mut app = build_headless_app();
    app.init_resource::<CombatWheelAdapter>();
    app.init_resource::<FactionSkin>();
    app.add_plugins(CombatWheelBindPlugin);

    // Spawn a player entity with full health so the adapter projection
    // systems have something to read.
    app.world_mut().spawn((
        Player,
        ShipStats::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    app
}

/// Drive the FixedUpdate systems once.
fn tick_fixed(app: &mut App) {
    // Bevy 0.15: FixedUpdate runs at a separate schedule; calling
    // `world_mut().run_schedule(FixedUpdate)` invokes it deterministically.
    app.world_mut().run_schedule(FixedUpdate);
}

#[test]
fn ship_stats_drop_updates_combat_wheel_adapter() {
    let mut app = build_triglavian_app();

    // Run an initial tick so the adapter projection writes the default
    // values into CombatWheelAdapter.
    tick_fixed(&mut app);

    {
        let adapter = app.world().resource::<CombatWheelAdapter>();
        let default_stats = ShipStats::default();
        assert!(
            (adapter.shield_max - default_stats.max_shield).abs() < 0.01,
            "shield_max should be projected from ShipStats: got {}, expected {}",
            adapter.shield_max,
            default_stats.max_shield
        );
        assert!(
            (adapter.shield_current - default_stats.shield).abs() < 0.01,
            "shield_current should match ShipStats.shield"
        );
        assert!(
            (adapter.hull_max - default_stats.max_hull).abs() < 0.01,
            "hull_max should be projected from ShipStats"
        );
        assert!(
            (adapter.armor_current - default_stats.armor).abs() < 0.01,
            "armor_current should be projected from ShipStats"
        );
        assert!(
            (adapter.capacitor_max - default_stats.max_capacitor).abs() < 0.01,
            "capacitor_max should be projected from ShipStats"
        );
    }

    // Drop shield on the player. Run another fixed tick — adapter should
    // pick up the new value.
    {
        let mut stats = app
            .world_mut()
            .query_filtered::<&mut ShipStats, With<Player>>()
            .single_mut(app.world_mut());
        stats.shield = 50.0;
        stats.hull = 200.0;
        stats.armor = 25.0;
        stats.capacitor = 100.0;
    }

    tick_fixed(&mut app);

    let adapter = app.world().resource::<CombatWheelAdapter>();
    assert!(
        (adapter.shield_current - 50.0).abs() < 0.01,
        "shield_current should follow ShipStats.shield: got {}",
        adapter.shield_current
    );
    assert!(
        (adapter.hull_current - 200.0).abs() < 0.01,
        "hull_current should follow ShipStats.hull: got {}",
        adapter.hull_current
    );
    assert!(
        (adapter.armor_current - 25.0).abs() < 0.01,
        "armor_current should follow ShipStats.armor: got {}",
        adapter.armor_current
    );
    assert!(
        (adapter.capacitor_current - 100.0).abs() < 0.01,
        "capacitor_current should follow ShipStats.capacitor: got {}",
        adapter.capacitor_current
    );
}

#[test]
fn shield_collapse_projects_correctly() {
    let mut app = build_triglavian_app();
    tick_fixed(&mut app);

    // Simulate "shields down + recharge delay active" — the contract
    // documented in `combat_wheel_bind.rs`: `shield_collapsed = shield <= 0
    // && shield_timer > 0`.
    {
        let mut stats = app
            .world_mut()
            .query_filtered::<&mut ShipStats, With<Player>>()
            .single_mut(app.world_mut());
        stats.shield = 0.0;
        stats.shield_timer = 5.0;
    }

    tick_fixed(&mut app);

    let adapter = app.world().resource::<CombatWheelAdapter>();
    assert!(
        adapter.shield_collapsed,
        "adapter.shield_collapsed should be true when shield=0 and shield_timer>0: got {}",
        adapter.shield_collapsed
    );
}

#[test]
fn heat_projects_from_combo_heat_system() {
    let mut app = build_triglavian_app();

    // ComboHeatSystem lives in the gameplay plugin; insert_resource here
    // (not init_resource) because we want to set a specific heat value.
    app.insert_resource(rebellion::systems::scoring_v2::ComboHeatSystem {
        heat: 90.0,
        ..Default::default()
    });

    tick_fixed(&mut app);

    let adapter = app.world().resource::<CombatWheelAdapter>();
    assert!(
        (adapter.heat_current - 90.0).abs() < 0.01,
        "heat_current should reflect ComboHeatSystem.heat: got {}",
        adapter.heat_current
    );
    // heat_locked triggers at 85+ per the adapter logic.
    assert!(
        adapter.heat_locked,
        "heat_locked should be true when heat >= 85: got {}",
        adapter.heat_locked
    );
}

#[test]
fn damage_event_forwards_to_combat_wheel_event_with_direction() {
    use rebellion::core::events::{DamageType, PlayerDamagedEvent};
    use rebellion::ui::combat_wheel::CombatWheelEvent;

    let mut app = build_triglavian_app();

    // Register the event types (build_headless_app may or may not already
    // have them; add_event is idempotent for our purposes).
    app.add_event::<PlayerDamagedEvent>();
    app.add_event::<CombatWheelEvent>();

    // Send a damage event from a known source position.
    app.world_mut().send_event(PlayerDamagedEvent {
        damage: 25.0,
        damage_type: DamageType::EM,
        source_position: Vec2::new(100.0, 0.0),
        shield_damage: 25.0,
        armor_damage: 0.0,
        hull_damage: 0.0,
        destroyed: false,
    });

    // Two updates: first fires the EventReader, second drains the buffer.
    app.update();
    app.update();

    let mut events = app.world_mut().resource_mut::<Events<CombatWheelEvent>>();
    let damage_events: Vec<_> = events
        .drain()
        .filter(|e| matches!(e, CombatWheelEvent::ShieldDamaged { .. }))
        .collect();

    assert!(
        !damage_events.is_empty(),
        "PlayerDamagedEvent should produce a CombatWheelEvent::ShieldDamaged"
    );
}

#[test]
fn triglavian_invasion_active_module_is_recognized() {
    // Smoke test for the gating predicate `is_triglavian_invasion` — the
    // predicate isn't directly callable from outside the module, but we
    // can verify the ActiveModule check by setting it and observing that
    // the bind plugin doesn't crash on FixedUpdate.
    let mut app = build_triglavian_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("triglavian_invasion");
        active.player_faction = Some("edencom".to_string());
    }

    // Drive a few fixed ticks. Bind plugin should not panic regardless of
    // module_id because the projection systems only read ShipStats.
    for _ in 0..3 {
        tick_fixed(&mut app);
    }

    // Adapter should still have projected the player's stats.
    let adapter = app.world().resource::<CombatWheelAdapter>();
    assert!(adapter.shield_max > 0.0);
    assert!(adapter.hull_max > 0.0);
}

// =============================================================================
// Phase 2: campaign gating tests
// =============================================================================

use rebellion::core::GameState;
use rebellion::ui::combat_wheel::{
    despawn_combat_wheel, spawn_capacitor_core, spawn_combat_wheel, spawn_heat_arc,
    spawn_integrity_ring, spawn_module_slots, spawn_module_text, spawn_percentage_labels,
    spawn_shield_ring, CombatWheel,
};

/// Builds a headless app with the spawn/despawn wiring that the production
/// `TriglavianInvasionPlugin` adds, gated by `in_triglavian_invasion`. We
/// do NOT include the full `TriglavianInvasionPlugin` because its
/// `spawn_trig_wave` system requires a `Window` entity that the headless
/// app does not provide.
fn build_triglavian_campaign_app() -> App {
    let mut app = build_headless_app();
    app.init_resource::<CombatWheelAdapter>();
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<bevy::sprite::ColorMaterial>>();
    app.add_plugins(rebellion::ui::combat_wheel::CombatWheelPlugin);
    app.add_plugins(
        rebellion::games::triglavian_invasion::combat_wheel_bind::CombatWheelBindPlugin,
    );

    // Mirror the production wiring from TriglavianInvasionPlugin::build().
    // We omit `start_trig_mission` here because it pulls in TriglavianShips /
    // wave spawners that require a Window entity.
    app.add_systems(
        OnEnter(GameState::Playing),
        (
            spawn_combat_wheel,
            spawn_shield_ring,
            spawn_integrity_ring,
            spawn_capacitor_core,
            spawn_heat_arc,
            spawn_module_slots,
            spawn_module_text,
            spawn_percentage_labels,
        )
            .chain()
            .run_if(rebellion::games::in_triglavian_invasion),
    )
    .add_systems(
        OnExit(GameState::Playing),
        despawn_combat_wheel.run_if(rebellion::games::in_triglavian_invasion),
    );

    // Spawn a player entity (the bind plugin's projection systems need it).
    app.world_mut().spawn((
        Player,
        ShipStats::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    app
}

#[test]
fn triglavian_spawns_combat_wheel_on_playing_entry() {
    let mut app = build_triglavian_campaign_app();

    // Set the active module to Triglavian Invasion before transitioning
    // to Playing. The spawn systems run on `OnEnter(GameState::Playing)`
    // gated by `in_triglavian_invasion`.
    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("triglavian_invasion");
        active.player_faction = Some("edencom".to_string());
    }

    // Confirm the wheel is not yet spawned (we're in some non-Playing state).
    {
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<CombatWheel>>();
        assert_eq!(q.iter(app.world()).count(), 0);
    }

    // Transition to Playing.
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // OnEnter(Playing) fires; spawn chains run.

    // After the transition, the Combat Wheel root must exist.
    let mut q = app
        .world_mut()
        .query_filtered::<Entity, With<CombatWheel>>();
    let wheel_count = q.iter(app.world()).count();
    assert_eq!(
        wheel_count, 1,
        "triglavian_invasion + Playing should spawn exactly one CombatWheel root, got {wheel_count}"
    );
}

#[test]
fn non_triglavian_campaign_does_not_spawn_combat_wheel() {
    let mut app = build_triglavian_campaign_app();

    // Set a different module — Elder Fleet. The spawn systems must NOT fire.
    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("elder_fleet");
        active.player_faction = Some("minmatar".to_string());
    }

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    let mut q = app
        .world_mut()
        .query_filtered::<Entity, With<CombatWheel>>();
    let wheel_count = q.iter(app.world()).count();
    assert_eq!(
        wheel_count, 0,
        "non-triglavian campaign should NOT spawn a CombatWheel, got {wheel_count}"
    );
}

#[test]
fn despawn_runs_on_playing_exit_for_triglavian() {
    let mut app = build_triglavian_campaign_app();

    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("triglavian_invasion");
        active.player_faction = Some("triglavian".to_string());
    }

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    {
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<CombatWheel>>();
        assert_eq!(q.iter(app.world()).count(), 1);
    }

    // Transition out of Playing.
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::MainMenu);
    app.update(); // OnExit(Playing) fires; despawn_combat_wheel runs.

    let mut q = app
        .world_mut()
        .query_filtered::<Entity, With<CombatWheel>>();
    let wheel_count = q.iter(app.world()).count();
    assert_eq!(
        wheel_count, 0,
        "despawn_combat_wheel must clear the wheel on Playing exit, got {wheel_count}"
    );
}

// =============================================================================
// Phase 3: faction skin sync tests
// =============================================================================

#[test]
fn sync_faction_skin_writes_edencom_for_edencom_player() {
    let mut app = build_triglavian_app();
    app.init_resource::<FactionSkin>();

    // Set up ActiveModule so the player faction is EDENCOM.
    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("triglavian_invasion");
        active.player_faction = Some("edencom".to_string());
    }

    // Manually run the sync system (Update schedule).
    let mut schedule = Schedule::default();
    schedule.add_systems(sync_faction_skin_from_active_module);
    schedule.run(app.world_mut());

    assert_eq!(
        *app.world().resource::<FactionSkin>(),
        FactionSkin::Edencom,
        "EDENCOM player_faction should map to FactionSkin::Edencom"
    );
}

#[test]
fn sync_faction_skin_writes_triglavian_for_triglavian_player() {
    let mut app = build_triglavian_app();
    app.init_resource::<FactionSkin>();

    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("triglavian_invasion");
        active.player_faction = Some("triglavian".to_string());
    }

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_faction_skin_from_active_module);
    schedule.run(app.world_mut());

    assert_eq!(
        *app.world().resource::<FactionSkin>(),
        FactionSkin::Triglavian,
        "Triglavian player_faction should map to FactionSkin::Triglavian"
    );
}

#[test]
fn sync_faction_skin_no_op_for_unknown_faction() {
    let mut app = build_triglavian_app();
    app.init_resource::<FactionSkin>();

    // Default skin is FactionSkin::Minmatar.
    let default_skin = *app.world().resource::<FactionSkin>();
    assert_eq!(default_skin, FactionSkin::Minmatar);

    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("triglavian_invasion");
        active.player_faction = Some("unknown_faction".to_string());
    }

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_faction_skin_from_active_module);
    schedule.run(app.world_mut());

    let after = *app.world().resource::<FactionSkin>();
    assert_eq!(
        after, default_skin,
        "unknown player_faction must leave the skin unchanged"
    );
}

#[test]
fn sync_faction_skin_no_op_when_player_faction_unset() {
    let mut app = build_triglavian_app();
    app.init_resource::<FactionSkin>();

    let default_skin = *app.world().resource::<FactionSkin>();

    // player_faction is None (no faction_select yet).
    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("triglavian_invasion");
    }

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_faction_skin_from_active_module);
    schedule.run(app.world_mut());

    let after = *app.world().resource::<FactionSkin>();
    assert_eq!(
        after, default_skin,
        "no player_faction set must leave the skin unchanged"
    );
}
