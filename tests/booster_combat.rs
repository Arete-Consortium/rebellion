//! Exercise boosters through pickup events, real player input and spawned shots.
//! Getter-only tests cannot detect a timer that never affects combat.
use bevy::prelude::*;
use rebellion::app_builder::build_headless_app;
use rebellion::core::{CollectiblePickedUpEvent, CollectibleType, GameState, WeaponType};
use rebellion::entities::boosters::{BoosterInventory, BoosterKind};
use rebellion::entities::{
    Movement, Player, PlayerProjectile, PowerupEffects, ProjectileDamage, Weapon,
};
use rebellion::games::ActiveModule;
use rebellion::systems::{Ability, AbilityType};

fn tick(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

fn setup(state: GameState) -> (App, Entity) {
    let mut app = build_headless_app();
    // Disable legacy waves without registering a chapter spawner: this is a
    // controlled combat fixture using the complete production player pipeline.
    app.world_mut()
        .resource_mut::<ActiveModule>()
        .set_module("caldari_gallente");
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    tick(&mut app, 4);
    let player = app
        .world_mut()
        .query_filtered::<Entity, With<Player>>()
        .single(app.world());
    if state != GameState::Playing {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(state);
        tick(&mut app, 2);
    }
    (app, player)
}

fn pickup(app: &mut App, kind: CollectibleType) {
    app.world_mut().send_event(CollectiblePickedUpEvent {
        collectible_type: kind,
        value: 0,
        position: Vec2::ZERO,
    });
    tick(app, 2);
    if let Some(kind) = BoosterKind::from_pickup(kind) {
        // The pickup stores a dose; use it through actual gamepad input.
        while app.world().resource::<BoosterInventory>().selected() != kind {
            app.world_mut()
                .resource_mut::<BoosterInventory>()
                .cycle(true);
        }
        let mut pad = Gamepad::default();
        pad.digital_mut().press(GamepadButton::North);
        let controller = app.world_mut().spawn(pad).id();
        tick(app, 2);
        app.world_mut().despawn(controller);
        tick(app, 1);
    }
}

fn speed(app: &mut App, player: Entity) -> f32 {
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    tick(app, 70); // settle acceleration/friction without altering base stats
    let speed = app
        .world()
        .get::<Movement>(player)
        .unwrap()
        .velocity
        .length();
    // Also prove the velocity moves the ship, away from the screen boundary.
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation
        .x = 0.0;
    tick(app, 6);
    let distance = app.world().get::<Transform>(player).unwrap().translation.x;
    assert!(
        (distance - speed * 0.1).abs() < 0.1,
        "velocity must move the ship"
    );
    speed
}

fn shot_damage(app: &mut App, player: Entity) -> f32 {
    let shots: Vec<_> = app
        .world_mut()
        .query_filtered::<Entity, With<PlayerProjectile>>()
        .iter(app.world())
        .collect();
    for shot in shots {
        app.world_mut().despawn(shot);
    }
    app.world_mut().get_mut::<Weapon>(player).unwrap().cooldown = 0.0;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    tick(app, 2);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    app.world_mut()
        .query_filtered::<&ProjectileDamage, With<PlayerProjectile>>()
        .iter(app.world())
        .next()
        .expect("normal fire input spawns a shot")
        .damage
}

#[test]
fn overclocker_changes_actual_speed_stores_repeats_and_expires_in_both_combat_states() {
    for state in [GameState::Playing, GameState::BossFight] {
        let (mut app, player) = setup(state);
        let base = app.world().get::<Movement>(player).unwrap().clone();
        let normal = speed(&mut app, player);
        pickup(&mut app, CollectibleType::Overdrive);
        assert!((speed(&mut app, player) / normal - 1.5).abs() < 0.01);
        pickup(&mut app, CollectibleType::Overdrive);
        assert!(
            (speed(&mut app, player) / normal - 1.5).abs() < 0.01,
            "a second collected dose stays stored while the effect is active"
        );
        tick(&mut app, 310);
        assert!(
            app.world()
                .get::<PowerupEffects>(player)
                .unwrap()
                .overdrive_timer
                <= 0.0
        );
        assert!((speed(&mut app, player) / normal - 1.0).abs() < 0.01);
        let after = app.world().get::<Movement>(player).unwrap();
        assert_eq!(after.max_speed, base.max_speed);
        assert_eq!(after.acceleration, base.acceleration);
    }
}

#[test]
fn pyrolancea_doubles_real_projectiles_without_changing_the_weapon() {
    for state in [GameState::Playing, GameState::BossFight] {
        for family in [
            WeaponType::Autocannon,
            WeaponType::Laser,
            WeaponType::Railgun,
            WeaponType::MissileLauncher,
        ] {
            let (mut app, player) = setup(state);
            app.world_mut()
                .get_mut::<Weapon>(player)
                .unwrap()
                .weapon_type = family;
            let base_damage = app.world().get::<Weapon>(player).unwrap().damage;
            let normal = shot_damage(&mut app, player);
            pickup(&mut app, CollectibleType::DamageBoost);
            assert!(
                (shot_damage(&mut app, player) / normal - 2.0).abs() < 0.001,
                "{state:?} / {family:?}: timer must double projectile damage"
            );
            pickup(&mut app, CollectibleType::DamageBoost);
            assert!((shot_damage(&mut app, player) / normal - 2.0).abs() < 0.001);
            tick(&mut app, 610);
            assert!((shot_damage(&mut app, player) / normal - 1.0).abs() < 0.001);
            assert_eq!(
                app.world().get::<Weapon>(player).unwrap().damage,
                base_damage
            );
        }
    }
}

#[test]
fn ship_overdrive_and_booster_compose_once_and_restore_base_speed() {
    let (mut app, player) = setup(GameState::Playing);
    let base = app.world().get::<Movement>(player).unwrap().max_speed;
    let normal = speed(&mut app, player);
    // Activate the ability independently of the maneuver sharing Shift.
    // Its production effect/cooldown systems still run on every fixed tick.
    {
        let mut ability = app.world_mut().get_mut::<Ability>(player).unwrap();
        ability.ability_type = AbilityType::Overdrive;
        ability.is_active = true;
        ability.effect_remaining = 3.0;
    }
    assert!((speed(&mut app, player) / normal - 1.5).abs() < 0.01);
    pickup(&mut app, CollectibleType::Overdrive);
    assert!((speed(&mut app, player) / normal - 2.25).abs() < 0.01);
    tick(&mut app, 310);
    assert!((speed(&mut app, player) / normal - 1.0).abs() < 0.01);
    assert_eq!(app.world().get::<Movement>(player).unwrap().max_speed, base);
}
