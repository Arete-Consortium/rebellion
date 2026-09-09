//! Exercise the shipped boundary using actual Bevy Gamepad components.
use bevy::prelude::*;
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
fn aim_interact_and_old_saved_bindings_cannot_fire_or_activate_ability() {
    let (mut app, controller, player) = combat(GameState::Playing);
    app.world_mut()
        .resource_mut::<KeyBindings>()
        .set(Action::Fire, Binding::GamepadButton(0));
    pad(
        &mut app,
        controller,
        &[GamepadButton::South, GamepadButton::West],
        &[(GamepadAxis::RightStickX, 1.0)],
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
        app.world().get::<Weapon>(player).unwrap().aim_direction,
        Vec2::X
    );
    assert_eq!(
        app.world()
            .get::<Ability>(player)
            .unwrap()
            .cooldown_remaining,
        0.0
    );
    pad(&mut app, controller, &[], &[(GamepadAxis::RightZ, 0.8)]);
    tick(&mut app, 12);
    assert!(
        app.world_mut()
            .query_filtered::<Entity, With<PlayerProjectile>>()
            .iter(app.world())
            .count()
            > 0
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
        pad(&mut app, controller, &[GamepadButton::LeftTrigger], &[]);
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
    press(&mut app, controller, GamepadButton::DPadRight);
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
