//! Exercise the shipped boundary using actual Bevy Gamepad components.
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use rebellion::entities::boosters::{BoosterActivatedEvent, BoosterInventory, BoosterKind};
use rebellion::{
    app_builder::{build_headless_app, ControllerGate, ControllerOnlyPlugin},
    core::*,
    entities::*,
    games::{ActiveModule, GameModulesPlugin},
    systems::{Ability, AbilityType, ManeuverState},
    ui::{
        menu::{common::MenuSelection, MenuPlugin},
        TransitionPlugin,
    },
};

fn tick(app: &mut App, count: usize) {
    for _ in 0..count {
        app.update();
    }
}
fn state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}
fn pad(app: &mut App, entity: Entity, buttons: &[GamepadButton], axes: &[(GamepadAxis, f32)]) {
    let mut g = Gamepad::default();
    for &button in buttons {
        g.digital_mut().press(button);
    }
    for &(axis, value) in axes {
        g.analog_mut().set(axis, value);
    }
    app.world_mut().entity_mut(entity).insert(g);
}
fn press(app: &mut App, entity: Entity, button: GamepadButton) {
    pad(app, entity, &[button], &[]);
    tick(app, 1);
    pad(app, entity, &[], &[]);
    tick(app, 12);
}
fn connect(app: &mut App) -> Entity {
    let entity = app.world_mut().spawn(Gamepad::default()).id();
    tick(app, 2);
    assert!(app.world().resource::<ControllerGate>().is_blocked());
    press(app, entity, GamepadButton::South);
    assert!(!app.world().resource::<ControllerGate>().is_blocked());
    entity
}
fn app() -> App {
    // This suite deliberately loads/remaps legacy controls. Give its process
    // a private save directory so SavePlugin cannot change the profile used
    // by later integration-test executables (or the developer's real save).
    static PROFILE: std::sync::Once = std::sync::Once::new();
    PROFILE.call_once(|| {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let profile = std::env::temp_dir().join(format!(
            "rebellion-controller-tests-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&profile).unwrap();
        std::env::set_var("REBELLION_HOME", profile);
    });
    let mut app = build_headless_app();
    app.add_plugins(ControllerOnlyPlugin);
    app
}

#[derive(Resource, Default)]
struct FixedTicks(usize);
fn count_fixed(mut ticks: ResMut<FixedTicks>) {
    ticks.0 += 1;
}

#[test]
fn missing_controller_freezes_simulation_and_blocks_keyboard_and_pointer() {
    let mut app = app();
    app.init_resource::<FixedTicks>()
        .add_systems(FixedUpdate, count_fixed);
    let button = app.world_mut().spawn(Interaction::Pressed).id();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);
    tick(&mut app, 20);
    assert_eq!(app.world().resource::<FixedTicks>().0, 0);
    assert!(!app
        .world()
        .resource::<ButtonInput<KeyCode>>()
        .pressed(KeyCode::Enter));
    assert_eq!(
        *app.world().get::<Interaction>(button).unwrap(),
        Interaction::None
    );
    assert!(app.world().resource::<ControllerGate>().is_blocked());
}

#[test]
fn held_controls_cannot_resume_and_resume_press_does_not_leak() {
    let mut app = app();
    app.init_resource::<FixedTicks>()
        .add_systems(FixedUpdate, count_fixed);
    let entity = app.world_mut().spawn(Gamepad::default()).id();
    pad(
        &mut app,
        entity,
        &[GamepadButton::South],
        &[(GamepadAxis::LeftStickX, 1.0), (GamepadAxis::RightZ, 1.0)],
    );
    tick(&mut app, 12);
    assert!(app.world().resource::<ControllerGate>().is_blocked());
    assert_eq!(app.world().resource::<FixedTicks>().0, 0);
    pad(&mut app, entity, &[], &[]);
    tick(&mut app, 1);
    pad(&mut app, entity, &[GamepadButton::South], &[]);
    tick(&mut app, 4);
    assert!(app.world().resource::<ControllerGate>().is_blocked());
    pad(&mut app, entity, &[], &[]);
    tick(&mut app, 1);
    assert!(!app.world().resource::<ControllerGate>().is_blocked());
    assert!(!app
        .world()
        .resource::<rebellion::systems::JoystickState>()
        .confirm());
    tick(&mut app, 1);
    assert!(app.world().resource::<FixedTicks>().0 > 0);
}

fn combat(game_state: GameState) -> (App, Entity, Entity) {
    let mut app = app();
    let controller = connect(&mut app);
    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("caldari_gallente");
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    tick(&mut app, 3);
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .single(app.world());
    app.world_mut()
        .get_mut::<PowerupEffects>(player)
        .unwrap()
        .invuln_timer = 0.0;
    {
        let mut m = app.world_mut().get_mut::<ManeuverState>(player).unwrap();
        m.invincible = false;
        m.invincibility_timer = 0.0;
    }
    app.world_mut()
        .entity_mut(player)
        .insert(Ability::new(AbilityType::ShieldBoost));
    if game_state != GameState::Playing {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(game_state);
        tick(&mut app, 2);
    }
    (app, controller, player)
}

fn collect_booster(app: &mut App, player: Entity, kind: CollectibleType, copies: usize) {
    let position = app
        .world()
        .get::<Transform>(player)
        .unwrap()
        .translation
        .truncate();
    app.world_mut()
        .run_system_once(move |mut commands: Commands| {
            for _ in 0..copies {
                rebellion::entities::collectible::spawn_collectible(
                    &mut commands,
                    position,
                    kind,
                    None,
                );
            }
        })
        .unwrap();
    tick(app, 2);
}

#[test]
fn timed_boosters_wait_for_y_and_never_spend_empty_or_already_active_doses() {
    for game_state in [GameState::Playing, GameState::BossFight] {
        let (mut app, controller, player) = combat(game_state);
        for kind in BoosterKind::ALL {
            collect_booster(&mut app, player, kind.pickup(), 2);
            while app.world().resource::<BoosterInventory>().selected() != kind {
                press(&mut app, controller, GamepadButton::DPadDown);
            }
            assert_eq!(
                kind.remaining(app.world().get::<PowerupEffects>(player).unwrap()),
                0.0
            );
            assert_eq!(app.world().resource::<BoosterInventory>().count(kind), 2);
            let mut activations = app
                .world()
                .resource::<Events<BoosterActivatedEvent>>()
                .get_cursor();
            // One update retains the activation event for this independent reader.
            pad(&mut app, controller, &[GamepadButton::North], &[]);
            tick(&mut app, 1);
            assert_eq!(
                activations
                    .read(app.world().resource::<Events<BoosterActivatedEvent>>())
                    .filter(|event| event.0 == kind)
                    .count(),
                1
            );
            pad(&mut app, controller, &[], &[]);
            tick(&mut app, 1);
            assert!(kind.remaining(app.world().get::<PowerupEffects>(player).unwrap()) > 0.0);
            assert_eq!(app.world().resource::<BoosterInventory>().count(kind), 1);
            press(&mut app, controller, GamepadButton::North);
            assert_eq!(
                app.world().resource::<BoosterInventory>().count(kind),
                1,
                "active effect cannot consume a second dose"
            );
            pad(&mut app, controller, &[GamepadButton::North], &[]);
            tick(&mut app, 650);
            assert_eq!(
                app.world().resource::<BoosterInventory>().count(kind),
                1,
                "holding Y through expiry must not consume again"
            );
            pad(&mut app, controller, &[], &[]);
            tick(&mut app, 1);
            press(&mut app, controller, GamepadButton::North);
            assert_eq!(app.world().resource::<BoosterInventory>().count(kind), 0);
            press(&mut app, controller, GamepadButton::North);
            assert_eq!(app.world().resource::<BoosterInventory>().count(kind), 0);
        }
    }
}

#[test]
fn full_booster_slots_leave_excess_pickups_in_space_and_repairs_stay_instant() {
    let (mut app, controller, player) = combat(GameState::Playing);
    collect_booster(&mut app, player, CollectibleType::DamageBoost, 5);
    assert_eq!(
        app.world()
            .resource::<BoosterInventory>()
            .count(BoosterKind::Pyrolancea),
        3
    );
    let count = app
        .world_mut()
        .query_filtered::<Entity, With<Collectible>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 2, "same-frame overflow stays available");
    press(&mut app, controller, GamepadButton::North);
    assert_eq!(
        app.world()
            .resource::<BoosterInventory>()
            .count(BoosterKind::Pyrolancea),
        3,
        "one leftover pickup refills the freed space"
    );
    app.world_mut().get_mut::<ShipStats>(player).unwrap().shield = 0.0;
    collect_booster(&mut app, player, CollectibleType::ShieldBoost, 1);
    assert!(app.world().get::<ShipStats>(player).unwrap().shield >= 25.0);
}

#[test]
fn booster_selection_skips_empty_slots_and_does_not_steer_change_ammo_or_activate_overload() {
    let (mut app, controller, player) = combat(GameState::Playing);
    app.world_mut()
        .get_mut::<Weapon>(player)
        .unwrap()
        .weapon_type = WeaponType::Autocannon;
    collect_booster(&mut app, player, CollectibleType::Overdrive, 1);
    collect_booster(&mut app, player, CollectibleType::Invulnerability, 1);
    let before = app.world().get::<Transform>(player).unwrap().translation;
    let ammo = app.world().get::<Weapon>(player).unwrap().ammo_type;
    press(&mut app, controller, GamepadButton::DPadDown);
    assert_eq!(
        app.world().resource::<BoosterInventory>().selected(),
        BoosterKind::XInstinct
    );
    press(&mut app, controller, GamepadButton::DPadUp);
    assert_eq!(
        app.world().resource::<BoosterInventory>().selected(),
        BoosterKind::Overclocker
    );
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        before
    );
    assert_eq!(app.world().get::<Weapon>(player).unwrap().ammo_type, ammo);
    app.world_mut().resource_mut::<SaltMinerSystem>().meter = 100.0;
    press(&mut app, controller, GamepadButton::North);
    assert!(!app.world().resource::<SaltMinerSystem>().is_active);
    assert!(
        !app.world()
            .get::<ManeuverState>(player)
            .unwrap()
            .thrust_active
    );
    assert_eq!(
        app.world()
            .get::<Ability>(player)
            .unwrap()
            .cooldown_remaining,
        0.0
    );
}

#[test]
fn doses_survive_mission_continuation_but_reset_on_death_and_new_hull_selection() {
    let (mut app, _, player) = combat(GameState::Playing);
    collect_booster(&mut app, player, CollectibleType::DamageBoost, 2);
    for state in [
        GameState::BossFight,
        GameState::StageComplete,
        GameState::Playing,
    ] {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(state);
        tick(&mut app, 3);
        assert_eq!(
            app.world()
                .resource::<BoosterInventory>()
                .count(BoosterKind::Pyrolancea),
            2
        );
    }
    for state in [
        GameState::GameOver,
        GameState::ShipSelect,
        GameState::MainMenu,
    ] {
        app.world_mut()
            .resource_mut::<BoosterInventory>()
            .store(BoosterKind::Overclocker);
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(state);
        tick(&mut app, 3);
        assert!(app.world().resource::<BoosterInventory>().is_empty());
    }
}

#[test]
fn disconnect_preserves_booster_time_and_requires_a_fresh_y_after_acknowledgement() {
    let (mut app, controller, player) = combat(GameState::BossFight);
    collect_booster(&mut app, player, CollectibleType::DamageBoost, 2);
    press(&mut app, controller, GamepadButton::North);
    let timer = app
        .world()
        .get::<PowerupEffects>(player)
        .unwrap()
        .damage_boost_timer;
    app.world_mut().despawn(controller);
    tick(&mut app, 100);
    assert_eq!(
        app.world()
            .get::<PowerupEffects>(player)
            .unwrap()
            .damage_boost_timer,
        timer
    );
    assert_eq!(
        app.world()
            .resource::<BoosterInventory>()
            .count(BoosterKind::Pyrolancea),
        1
    );
    let replacement = app.world_mut().spawn(Gamepad::default()).id();
    pad(&mut app, replacement, &[GamepadButton::North], &[]);
    tick(&mut app, 5);
    assert!(app.world().resource::<ControllerGate>().is_blocked());
    pad(&mut app, replacement, &[], &[]);
    tick(&mut app, 1);
    press(&mut app, replacement, GamepadButton::South);
    assert!(!app.world().resource::<ControllerGate>().is_blocked());
    assert_eq!(
        app.world()
            .resource::<BoosterInventory>()
            .count(BoosterKind::Pyrolancea),
        1
    );
}

#[test]
fn pause_freezes_active_doses_and_explicit_restart_clears_inventory() {
    let (mut app, controller, player) = combat(GameState::Playing);
    collect_booster(&mut app, player, CollectibleType::DamageBoost, 2);
    press(&mut app, controller, GamepadButton::North);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Paused);
    tick(&mut app, 2);
    let timer = app
        .world()
        .get::<PowerupEffects>(player)
        .unwrap()
        .damage_boost_timer;
    pad(&mut app, controller, &[GamepadButton::North], &[]);
    tick(&mut app, 100);
    assert_eq!(
        app.world()
            .get::<PowerupEffects>(player)
            .unwrap()
            .damage_boost_timer,
        timer
    );
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    tick(&mut app, 3);
    assert_eq!(
        app.world()
            .resource::<BoosterInventory>()
            .count(BoosterKind::Pyrolancea),
        1
    );
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Paused);
    tick(&mut app, 2);
    app.world_mut()
        .resource_mut::<PauseContext>()
        .request_restart();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    tick(&mut app, 3);
    assert!(app.world().resource::<BoosterInventory>().is_empty());
}

#[test]
fn manual_x_instinct_blocks_real_projectile_damage_and_expiry_restores_damage() {
    let (mut app, controller, player) = combat(GameState::BossFight);
    collect_booster(&mut app, player, CollectibleType::Invulnerability, 1);
    press(&mut app, controller, GamepadButton::North);
    for protected in [true, false] {
        let position = app.world().get::<Transform>(player).unwrap().translation;
        let shield = app.world().get::<ShipStats>(player).unwrap().shield;
        app.world_mut().spawn((
            EnemyProjectile,
            Transform::from_translation(position),
            ProjectilePhysics {
                velocity: Vec2::ZERO,
                lifetime: 2.0,
            },
            ProjectileDamage {
                damage: 10.0,
                damage_type: DamageType::Kinetic,
                ammo_type: AmmoType::Sabot,
                crit_chance: 0.0,
                crit_multiplier: 1.0,
            },
        ));
        tick(&mut app, 2);
        let after = app.world().get::<ShipStats>(player).unwrap().shield;
        if protected {
            assert!(after >= shield);
        } else {
            assert!(after < shield, "expired protection must allow real damage");
        }
        tick(&mut app, 200);
    }
}

#[test]
fn disconnect_freezes_health_position_projectiles_and_ability_timers_in_both_combat_states() {
    for game_state in [GameState::Playing, GameState::BossFight] {
        let (mut app, controller, player) = combat(game_state);
        app.world_mut()
            .get_mut::<Ability>(player)
            .unwrap()
            .cooldown_remaining = 5.0;
        let position = app.world().get::<Transform>(player).unwrap().translation;
        let hp = app.world().get::<ShipStats>(player).unwrap().clone();
        let bullet = app
            .world_mut()
            .spawn((
                EnemyProjectile,
                Transform::from_translation(position),
                ProjectilePhysics {
                    velocity: Vec2::new(0.0, -200.0),
                    lifetime: 2.0,
                },
                ProjectileDamage {
                    damage: 10.0,
                    damage_type: DamageType::Kinetic,
                    ammo_type: AmmoType::Sabot,
                    crit_chance: 0.0,
                    crit_multiplier: 1.0,
                },
            ))
            .id();
        app.world_mut().despawn(controller);
        tick(&mut app, 60);
        assert_eq!(
            app.world().get::<Transform>(player).unwrap().translation,
            position
        );
        assert_eq!(app.world().get::<ShipStats>(player).unwrap().hull, hp.hull);
        assert_eq!(
            app.world().get::<ShipStats>(player).unwrap().shield,
            hp.shield
        );
        assert_eq!(
            app.world()
                .get::<Ability>(player)
                .unwrap()
                .cooldown_remaining,
            5.0
        );
        assert_eq!(
            app.world()
                .get::<ProjectilePhysics>(bullet)
                .unwrap()
                .lifetime,
            2.0
        );
        assert_eq!(state(&app), game_state);
        let replacement = app.world_mut().spawn(Gamepad::default()).id();
        tick(&mut app, 3);
        assert_eq!(
            app.world()
                .get::<Ability>(player)
                .unwrap()
                .cooldown_remaining,
            5.0
        );
        press(&mut app, replacement, GamepadButton::South);
        assert!(!app.world().resource::<ControllerGate>().is_blocked());
        assert!(
            app.world()
                .get::<Ability>(player)
                .unwrap()
                .cooldown_remaining
                < 5.0
        );
    }
}

#[test]
fn right_stick_fires_in_its_direction_and_centering_stops_fire() {
    for game_state in [GameState::Playing, GameState::BossFight] {
        let (mut app, controller, player) = combat(game_state);
        let mut shots = app
            .world()
            .resource::<Events<PlayerFireEvent>>()
            .get_cursor();
        for direction in [
            Vec2::X,
            Vec2::Y,
            Vec2::NEG_X,
            Vec2::NEG_Y,
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
        ] {
            pad(
                &mut app,
                controller,
                &[],
                &[
                    (GamepadAxis::RightStickX, direction.x),
                    (GamepadAxis::RightStickY, direction.y),
                ],
            );
            app.world_mut().get_mut::<Weapon>(player).unwrap().cooldown = 0.0;
            tick(&mut app, 1);
            let fired: Vec<_> = shots
                .read(app.world().resource::<Events<PlayerFireEvent>>())
                .collect();
            assert!(!fired.is_empty(), "stick direction {direction:?} must fire");
            assert!(fired
                .iter()
                .all(|shot| shot.direction.distance(direction.normalize()) < 0.001));
            assert_eq!(
                app.world()
                    .get::<Ability>(player)
                    .unwrap()
                    .cooldown_remaining,
                0.0
            );
            // Small centered-stick noise must neither fire nor change the saved aim.
            pad(
                &mut app,
                controller,
                &[],
                &[(GamepadAxis::RightStickX, 0.1)],
            );
            app.world_mut().get_mut::<Weapon>(player).unwrap().cooldown = 0.0;
            tick(&mut app, 1);
            assert_eq!(
                shots
                    .read(app.world().resource::<Events<PlayerFireEvent>>())
                    .count(),
                0
            );
            assert!(
                app.world()
                    .get::<Weapon>(player)
                    .unwrap()
                    .aim_direction
                    .distance(direction.normalize())
                    < 0.001
            );
        }
    }
}

#[test]
fn face_buttons_triggers_and_old_saved_bindings_cannot_fire_the_primary_weapon() {
    let (mut app, controller, player) = combat(GameState::Playing);
    app.world_mut()
        .resource_mut::<KeyBindings>()
        .set(Action::Fire, Binding::GamepadButton(0));
    pad(
        &mut app,
        controller,
        &[
            GamepadButton::South,
            GamepadButton::East,
            GamepadButton::West,
            GamepadButton::North,
        ],
        &[(GamepadAxis::RightZ, 0.8)],
    );
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    tick(&mut app, 12);
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PlayerProjectile>>()
            .iter(app.world())
            .count(),
        0
    );
    assert_eq!(
        app.world()
            .get::<Ability>(player)
            .unwrap()
            .cooldown_remaining,
        0.0
    );
    assert!(
        app.world()
            .get::<ManeuverState>(player)
            .unwrap()
            .thrust_cooldown
            > 0.0
    );
}

#[test]
fn ability_and_maneuvers_require_a_new_press_and_have_separate_controls() {
    for game_state in [GameState::Playing, GameState::BossFight] {
        let (mut app, controller, player) = combat(game_state);
        // Analog-button LT reporting, without a LeftZ axis value.
        app.world_mut()
            .get_mut::<Gamepad>(controller)
            .unwrap()
            .analog_mut()
            .set(GamepadButton::LeftTrigger2, 0.8);
        tick(&mut app, 1);
        assert!(
            app.world()
                .get::<Ability>(player)
                .unwrap()
                .cooldown_remaining
                > 0.0
        );
        assert!(
            !app.world()
                .get::<ManeuverState>(player)
                .unwrap()
                .thrust_active
        );
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<PlayerProjectile>>()
                .iter(app.world())
                .count(),
            0
        );
        app.world_mut()
            .get_mut::<Ability>(player)
            .unwrap()
            .cooldown_remaining = 0.0;
        tick(&mut app, 4);
        assert_eq!(
            app.world()
                .get::<Ability>(player)
                .unwrap()
                .cooldown_remaining,
            0.0,
            "held LT cannot recast"
        );
        pad(&mut app, controller, &[], &[]);
        tick(&mut app, 1);
        pad(&mut app, controller, &[GamepadButton::RightTrigger2], &[]);
        tick(&mut app, 1);
        assert!(
            app.world()
                .get::<ManeuverState>(player)
                .unwrap()
                .thrust_active
        );
        assert_eq!(
            app.world()
                .get::<Ability>(player)
                .unwrap()
                .cooldown_remaining,
            0.0
        );
        {
            let mut m = app.world_mut().get_mut::<ManeuverState>(player).unwrap();
            m.thrust_active = false;
            m.thrust_cooldown = 0.0;
        }
        tick(&mut app, 4);
        assert!(
            !app.world()
                .get::<ManeuverState>(player)
                .unwrap()
                .thrust_active
        );
        pad(
            &mut app,
            controller,
            &[GamepadButton::RightTrigger],
            &[(GamepadAxis::LeftStickX, -1.0)],
        );
        tick(&mut app, 1);
        assert!(
            app.world()
                .get::<ManeuverState>(player)
                .unwrap()
                .barrel_roll_active
        );
    }
}

#[test]
fn dpad_ammo_does_not_move_ship_and_analog_stick_preserves_strength() {
    let (mut app, controller, player) = combat(GameState::Playing);
    app.world_mut()
        .get_mut::<Weapon>(player)
        .unwrap()
        .weapon_type = WeaponType::Autocannon;
    let start = app.world().get::<Transform>(player).unwrap().translation;
    let initial_ammo = app.world().get::<Weapon>(player).unwrap().ammo_type;
    press(&mut app, controller, GamepadButton::DPadRight);
    assert_eq!(
        app.world().get::<Weapon>(player).unwrap().ammo_type,
        initial_ammo.next()
    );
    press(&mut app, controller, GamepadButton::West);
    assert_eq!(
        app.world().get::<Weapon>(player).unwrap().ammo_type,
        initial_ammo
    );
    pad(&mut app, controller, &[GamepadButton::East], &[]);
    tick(&mut app, 20);
    assert_eq!(
        app.world().get::<Weapon>(player).unwrap().ammo_type,
        initial_ammo.next(),
        "holding B cycles only once"
    );
    assert_eq!(
        app.world().get::<Transform>(player).unwrap().translation,
        start
    );
    let mut speeds = Vec::new();
    for value in [0.4, 1.0] {
        app.world_mut()
            .get_mut::<Movement>(player)
            .unwrap()
            .velocity = Vec2::ZERO;
        pad(
            &mut app,
            controller,
            &[],
            &[(GamepadAxis::LeftStickX, value)],
        );
        tick(&mut app, 1);
        speeds.push(
            app.world()
                .get::<Movement>(player)
                .unwrap()
                .velocity
                .length(),
        );
    }
    assert!(speeds[0] > 0.0 && speeds[1] > speeds[0] * 2.0);
}

#[test]
fn overload_uses_lb_without_spending_maneuver_capacitor_or_triggering_on_y() {
    for game_state in [GameState::Playing, GameState::BossFight] {
        let (mut app, controller, player) = combat(game_state);
        app.world_mut().resource_mut::<SaltMinerSystem>().meter = 100.0;
        press(&mut app, controller, GamepadButton::North);
        assert!(!app.world().resource::<SaltMinerSystem>().is_active);
        let cap = app.world().get::<ShipStats>(player).unwrap().capacitor;
        pad(&mut app, controller, &[GamepadButton::LeftTrigger], &[]);
        tick(&mut app, 1);
        assert!(app.world().resource::<SaltMinerSystem>().is_active);
        assert!(
            !app.world()
                .get::<ManeuverState>(player)
                .unwrap()
                .thrust_active
        );
        assert_eq!(
            app.world()
                .get::<Ability>(player)
                .unwrap()
                .cooldown_remaining,
            0.0
        );
        assert!(app.world().get::<ShipStats>(player).unwrap().capacitor >= cap);
        {
            let mut overload = app.world_mut().resource_mut::<SaltMinerSystem>();
            overload.is_active = false;
            overload.meter = 100.0;
        }
        tick(&mut app, 2);
        assert!(
            !app.world().resource::<SaltMinerSystem>().is_active,
            "held LB cannot reactivate overload"
        );
    }
}

fn menu_app() -> (App, Entity) {
    let mut app = app();
    app.add_plugins((GameModulesPlugin, MenuPlugin, TransitionPlugin));
    app.add_event::<AppExit>();
    app.init_resource::<rebellion::systems::audio::SoundAssets>();
    app.init_resource::<rebellion::systems::touch_joystick::MobileMode>();
    let controller = connect(&mut app);
    tick(&mut app, 125);
    assert_eq!(state(&app), GameState::MainMenu);
    (app, controller)
}
fn choose(app: &mut App, controller: Entity, index: usize, expected: GameState) {
    for _ in 0..10 {
        if app.world().resource::<MenuSelection>().index == index {
            break;
        }
        press(app, controller, GamepadButton::DPadRight);
    }
    assert_eq!(app.world().resource::<MenuSelection>().index, index);
    press(app, controller, GamepadButton::South);
    tick(app, 65);
    assert_eq!(state(app), expected);
}

#[test]
fn controller_navigates_both_chapters_and_all_four_factions_to_combat() {
    for chapter in 0..2 {
        for side in 0..2 {
            let (mut app, controller) = menu_app();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Enter);
            tick(&mut app, 2);
            assert_eq!(state(&app), GameState::MainMenu);
            choose(&mut app, controller, 0, GameState::ModuleSelect);
            choose(&mut app, controller, chapter, GameState::FactionSelect);
            choose(&mut app, controller, side, GameState::DifficultySelect);
            choose(&mut app, controller, 1, GameState::ShipSelect);
            choose(&mut app, controller, 0, GameState::MissionBriefing);
            press(&mut app, controller, GamepadButton::South);
            tick(&mut app, 65);
            assert_eq!(state(&app), GameState::Playing);
        }
    }
}

#[test]
fn controller_guide_has_no_keyboard_capture_and_returns_with_b() {
    let (mut app, controller) = menu_app();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Controls);
    tick(&mut app, 2);
    let labels: Vec<String> = app
        .world_mut()
        .query::<&Text>()
        .iter(app.world())
        .map(|t| t.0.clone())
        .collect();
    assert!(labels.iter().any(|s| s == "HULL ABILITY"));
    assert!(!labels
        .iter()
        .any(|s| s.contains("(kbd)") || s.contains("Rebind")));
    press(&mut app, controller, GamepadButton::South);
    assert!(app
        .world()
        .resource::<rebellion::ui::menu::controls::ControlsCaptureState>()
        .capturing
        .is_none());
    press(&mut app, controller, GamepadButton::East);
    assert_eq!(state(&app), GameState::Options);
}

#[test]
fn second_controller_cannot_steal_input_and_device_replacement_requires_acknowledgement() {
    let (mut app, first, player) = combat(GameState::Playing);
    let second = app.world_mut().spawn(Gamepad::default()).id();
    pad(
        &mut app,
        second,
        &[GamepadButton::South],
        &[(GamepadAxis::RightZ, 1.0)],
    );
    tick(&mut app, 4);
    assert_eq!(
        app.world()
            .resource::<rebellion::systems::JoystickState>()
            .active_gamepad,
        Some(first)
    );
    assert_eq!(app.world().get::<Weapon>(player).unwrap().cooldown, 0.0);
    app.world_mut().despawn(first);
    tick(&mut app, 3);
    assert!(app.world().resource::<ControllerGate>().is_blocked());
    pad(&mut app, second, &[], &[]);
    tick(&mut app, 2);
    press(&mut app, second, GamepadButton::South);
    assert!(!app.world().resource::<ControllerGate>().is_blocked());
}

#[test]
fn menu_pause_and_disconnect_preserve_boss_state_until_player_resumes() {
    let (mut app, controller, player) = combat(GameState::BossFight);
    app.add_plugins((GameModulesPlugin, MenuPlugin, TransitionPlugin));
    app.init_resource::<rebellion::systems::audio::SoundAssets>();
    app.init_resource::<rebellion::systems::touch_joystick::MobileMode>();
    app.add_event::<AppExit>();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);
    tick(&mut app, 2);
    assert_eq!(state(&app), GameState::BossFight);
    press(&mut app, controller, GamepadButton::Select);
    assert_eq!(state(&app), GameState::BossFight);
    press(&mut app, controller, GamepadButton::Start);
    assert_eq!(state(&app), GameState::Paused);
    app.world_mut().despawn(controller);
    tick(&mut app, 5);
    let replacement = connect(&mut app);
    assert_eq!(
        state(&app),
        GameState::Paused,
        "connection A cannot also select Resume"
    );
    press(&mut app, replacement, GamepadButton::East);
    tick(&mut app, 65);
    assert_eq!(state(&app), GameState::BossFight);
    assert!(
        app.world().get::<Player>(player).is_some(),
        "pause preserves the same player"
    );
}
