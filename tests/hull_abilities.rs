//! Abilities must change real shots and damage, not only their HUD timers.
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use rebellion::app_builder::build_headless_app;
use rebellion::core::{DamageType, GameState, WeaponType};
use rebellion::entities::environment::{
    EnvironmentCollider, EnvironmentContactDamage, EnvironmentObject,
};
use rebellion::entities::*;
use rebellion::games::ActiveModule;
use rebellion::systems::{Ability, AbilityType, ManeuverState};

fn tick(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

fn setup(state: GameState) -> (App, Entity) {
    let mut app = build_headless_app();
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
    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(0.0, -300.0, 0.0);
    // Isolate the ship ability from the separate thrust action sharing Shift.
    let mut maneuver = app.world_mut().get_mut::<ManeuverState>(player).unwrap();
    maneuver.thrust_cooldown = 1000.0;
    maneuver.invincible = false;
    maneuver.invincibility_timer = 0.0;
    if state != GameState::Playing {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(state);
        tick(&mut app, 2);
    }
    (app, player)
}

fn activate(app: &mut App, player: Entity, kind: AbilityType) {
    app.world_mut()
        .entity_mut(player)
        .insert(Ability::new(kind));
    app.world_mut()
        .get_mut::<ShipStats>(player)
        .unwrap()
        .capacitor = 100.0;
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);
    tick(app, 1);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    tick(app, 1);
    let cap = app.world().get::<ShipStats>(player).unwrap().capacitor;
    assert!(
        (cap - (100.0 - kind.capacitor_cost())).abs() < 1.0,
        "ability pays its cost once: {cap}"
    );
    assert!(
        app.world()
            .get::<Ability>(player)
            .unwrap()
            .cooldown_remaining
            > 0.0
    );
}

fn hp(app: &App, player: Entity) -> f32 {
    let s = app.world().get::<ShipStats>(player).unwrap();
    s.shield + s.armor + s.hull
}

#[derive(Clone, Copy, Debug)]
enum DamageSource {
    Projectile,
    Terrain,
    Beam,
}

fn receive_damage(app: &mut App, player: Entity, source: DamageSource) -> f32 {
    // Compare incoming damage without shield regeneration during the sample.
    app.world_mut()
        .get_mut::<ShipStats>(player)
        .unwrap()
        .shield_timer = 10.0;
    let pos = app.world().get::<Transform>(player).unwrap().translation;
    let before = hp(app, player);
    let entity = match source {
        DamageSource::Projectile => app
            .world_mut()
            .spawn(EnemyProjectileBundle {
                transform: Transform::from_translation(pos),
                physics: ProjectilePhysics {
                    velocity: Vec2::ZERO,
                    lifetime: 2.0,
                },
                damage: ProjectileDamage {
                    damage: 20.0,
                    damage_type: DamageType::EM,
                    ..default()
                },
                ..default()
            })
            .id(),
        DamageSource::Terrain => app
            .world_mut()
            .spawn((
                EnvironmentObject,
                EnvironmentCollider { radius: 25.0 },
                EnvironmentContactDamage {
                    amount: 20.0,
                    damage_type: DamageType::EM,
                    cooldown_ticks: 60,
                },
                Transform::from_translation(pos),
            ))
            .id(),
        DamageSource::Beam => app
            .world_mut()
            .spawn((
                Enemy,
                EnemyAI {
                    active: true,
                    ..default()
                },
                DisintegratorRamp::new(600.0, 1.0, 1.0),
                Transform::from_translation(pos + Vec3::Y * 100.0),
            ))
            .id(),
    };
    tick(app, 2);
    let loss = before - hp(app, player);
    if app.world().get_entity(entity).is_ok() {
        app.world_mut().despawn(entity);
    }
    loss
}

#[test]
fn defensive_abilities_protect_against_projectiles_terrain_and_beams_then_expire() {
    for state in [GameState::Playing, GameState::BossFight] {
        for source in [
            DamageSource::Projectile,
            DamageSource::Terrain,
            DamageSource::Beam,
        ] {
            for (kind, factor) in [
                (AbilityType::ArmorHardener, 0.5),
                (AbilityType::Afterburner, 0.0),
            ] {
                let (mut app, player) = setup(state);
                let normal = receive_damage(&mut app, player, source);
                assert!(normal > 0.0, "fixture must deal real damage");
                activate(&mut app, player, kind);
                let protected = receive_damage(&mut app, player, source);
                assert!(
                    (protected - normal * factor).abs() < 0.05,
                    "{state:?}/{source:?}/{kind:?}: {normal} -> {protected}"
                );
                tick(&mut app, (kind.duration() * 60.0) as usize + 2);
                let ordinary = receive_damage(&mut app, player, source);
                assert!(
                    (ordinary - normal).abs() < 0.05,
                    "protection must expire: {ordinary} vs {normal}"
                );
            }
        }
    }
}

fn shoot_target(kind: AbilityType, family: WeaponType, distance: f32, expire_first: bool) -> f32 {
    let (mut app, player) = setup(GameState::Playing);
    {
        let mut weapon = app.world_mut().get_mut::<Weapon>(player).unwrap();
        weapon.weapon_type = family;
        weapon.cooldown = 0.0;
        weapon.damage = 10.0;
    }
    if kind != AbilityType::None {
        activate(&mut app, player, kind);
        if expire_first {
            tick(&mut app, (kind.duration() * 60.0) as usize + 2);
        }
    }
    // Shots use the ship's position as their launch origin.
    let target = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyStats {
                health: 10000.0,
                max_health: 10000.0,
                ..default()
            },
            Transform::from_xyz(0.0, -300.0 + distance, 0.0),
        ))
        .id();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Space);
    tick(&mut app, 2);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    // Remove random critical hits from this controlled damage comparison.
    for mut d in app
        .world_mut()
        .query_filtered::<&mut ProjectileDamage, With<PlayerProjectile>>()
        .iter_mut(app.world_mut())
    {
        d.crit_chance = 0.0;
    }
    tick(&mut app, 75);
    10000.0 - app.world().get::<EnemyStats>(target).unwrap().health
}

#[test]
fn scorch_reaches_a_target_beyond_normal_laser_range_and_expires() {
    assert!(shoot_target(AbilityType::None, WeaponType::Laser, 400.0, false) > 0.0);
    assert_eq!(
        shoot_target(AbilityType::None, WeaponType::Laser, 600.0, false),
        0.0
    );
    assert!(shoot_target(AbilityType::Scorch, WeaponType::Laser, 600.0, false) > 0.0);
    assert_eq!(
        shoot_target(AbilityType::Scorch, WeaponType::Laser, 600.0, true),
        0.0
    );
}

#[test]
fn close_range_only_doubles_nearby_hits_and_expires() {
    let near = shoot_target(AbilityType::None, WeaponType::Railgun, 100.0, false);
    let far = shoot_target(AbilityType::None, WeaponType::Railgun, 350.0, false);
    assert!(near > 0.0 && far > 0.0);
    assert_eq!(
        shoot_target(AbilityType::CloseRange, WeaponType::Railgun, 100.0, false),
        near * 2.0
    );
    assert_eq!(
        shoot_target(AbilityType::CloseRange, WeaponType::Railgun, 350.0, false),
        far
    );
    assert_eq!(
        shoot_target(AbilityType::CloseRange, WeaponType::Railgun, 100.0, true),
        near
    );
}

#[test]
fn burst_abilities_fire_once_even_during_weapon_cooldown_without_holding_fire() {
    for state in [GameState::Playing, GameState::BossFight] {
        for (kind, family, count) in [
            (AbilityType::RocketBarrage, WeaponType::Autocannon, 3),
            (AbilityType::Salvo, WeaponType::MissileLauncher, 4),
        ] {
            let (mut app, player) = setup(state);
            {
                let mut weapon = app.world_mut().get_mut::<Weapon>(player).unwrap();
                weapon.weapon_type = family;
                weapon.cooldown = 0.8;
            }
            activate(&mut app, player, kind);
            tick(&mut app, 3);
            assert_eq!(
                app.world_mut()
                    .query_filtered::<Entity, With<PlayerProjectile>>()
                    .iter(app.world())
                    .count(),
                count,
                "{state:?}/{kind:?}: activation must fire exactly one burst"
            );
            tick(&mut app, 8);
            assert_eq!(
                app.world_mut()
                    .query_filtered::<Entity, With<PlayerProjectile>>()
                    .iter(app.world())
                    .count(),
                count,
                "no duplicate burst on later ticks"
            );
        }
    }
}

fn travel(app: &mut App, enemy: Entity, y: f32) -> f32 {
    app.world_mut()
        .get_mut::<Transform>(enemy)
        .unwrap()
        .translation = Vec3::new(0.0, y, 0.0);
    tick(app, 6);
    y - app.world().get::<Transform>(enemy).unwrap().translation.y
}

#[test]
fn disruptor_halves_nearby_enemy_motion_without_changing_base_speed() {
    for state in [GameState::Playing, GameState::BossFight] {
        let (mut app, player) = setup(state);
        let enemy = app
            .world_mut()
            .spawn((
                Enemy,
                EnemyStats {
                    speed: 60.0,
                    ..default()
                },
                EnemyAI {
                    active: true,
                    ..default()
                },
                Transform::default(),
            ))
            .id();
        let normal = travel(&mut app, enemy, -100.0);
        activate(&mut app, player, AbilityType::WarpDisruptor);
        assert!((travel(&mut app, enemy, -100.0) / normal - 0.5).abs() < 0.01);
        assert!(
            (travel(&mut app, enemy, 200.0) / normal - 1.0).abs() < 0.01,
            "outside the field is unaffected"
        );
        tick(&mut app, 185);
        assert!(
            (travel(&mut app, enemy, -100.0) / normal - 1.0).abs() < 0.01,
            "expired field restores motion"
        );
        assert_eq!(app.world().get::<EnemyStats>(enemy).unwrap().speed, 60.0);
    }
}

#[test]
fn ability_duration_and_cooldown_freeze_while_paused() {
    let (mut app, player) = setup(GameState::Playing);
    activate(&mut app, player, AbilityType::Scorch);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Paused);
    tick(&mut app, 2);
    let a = app.world().get::<Ability>(player).unwrap();
    let saved = (a.effect_remaining, a.cooldown_remaining);
    tick(&mut app, 180);
    let a = app.world().get::<Ability>(player).unwrap();
    assert_eq!((a.effect_remaining, a.cooldown_remaining), saved);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    tick(&mut app, 2);
    assert!(app.world().get::<Ability>(player).unwrap().effect_remaining < saved.0);
}

#[test]
fn drone_abilities_spawn_attack_and_expire() {
    for (kind, count) in [(AbilityType::DeployDrone, 1), (AbilityType::DroneBay, 2)] {
        let (mut app, player) = setup(GameState::Playing);
        let target = app
            .world_mut()
            .spawn((
                Enemy,
                EnemyStats {
                    health: 10000.0,
                    max_health: 10000.0,
                    ..default()
                },
                Transform::from_xyz(0.0, -120.0, 0.0),
            ))
            .id();
        activate(&mut app, player, kind);
        tick(&mut app, 3);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Drone>>()
                .iter(app.world())
                .count(),
            count
        );
        tick(&mut app, 120);
        assert!(
            app.world().get::<EnemyStats>(target).unwrap().health < 10000.0,
            "autonomous drones must damage enemies"
        );
        tick(&mut app, (kind.duration() * 60.0) as usize + 5);
        assert_eq!(
            app.world_mut()
                .query_filtered::<Entity, With<Drone>>()
                .iter(app.world())
                .count(),
            0
        );
    }
}

#[test]
fn close_range_checks_each_chain_impact_instead_of_carrying_a_near_bonus_outward() {
    use rebellion::core::{ContactRaw, RawContactType};
    use rebellion::simulation::resolve_damage::{
        enrich_contacts, resolve_player_projectile_damage,
    };
    use rebellion::systems::collision::SpatialGrid;
    let (mut app, _) = setup(GameState::Playing);
    let near_pos = Vec2::new(0.0, 200.0);
    let far_pos = Vec2::new(20.0, 210.0);
    let near = app.world_mut().spawn((Enemy, EnemyStats::default())).id();
    let far = app.world_mut().spawn((Enemy, EnemyStats::default())).id();
    app.world_mut()
        .resource_mut::<SpatialGrid>()
        .insert_enemy(far, far_pos);
    let shot = app
        .world_mut()
        .spawn((
            PlayerProjectileBundle {
                damage: ProjectileDamage {
                    damage: 10.0,
                    crit_chance: 0.0,
                    ..default()
                },
                ..default()
            },
            CloseRangeBonus {
                origin: Vec2::ZERO,
                multiplier: 2.0,
            },
            ChainOnHit(1),
        ))
        .id();
    app.world_mut().send_event(ContactRaw {
        contact_type: RawContactType::PlayerProjectileEnemy {
            projectile: shot,
            enemy: near,
            projectile_pos: near_pos,
            enemy_pos: near_pos,
        },
    });
    app.world_mut().run_system_once(enrich_contacts).unwrap();
    app.world_mut()
        .run_system_once(resolve_player_projectile_damage)
        .unwrap();
    assert_eq!(
        app.world().get::<EnemyStats>(near).unwrap().health,
        10.0,
        "the exact 200-unit boundary receives double damage"
    );
    assert_eq!(
        app.world().get::<EnemyStats>(far).unwrap().health,
        21.0,
        "outside the boundary receives ordinary chain damage"
    );
}

#[test]
fn shield_boost_and_armor_repair_restore_actual_defenses() {
    let (mut app, player) = setup(GameState::BossFight);
    {
        let mut s = app.world_mut().get_mut::<ShipStats>(player).unwrap();
        s.shield = 0.0;
        s.shield_timer = 20.0;
        s.armor = 0.0;
    }
    activate(&mut app, player, AbilityType::ShieldBoost);
    let s = app.world().get::<ShipStats>(player).unwrap();
    assert!((s.shield - s.max_shield * 0.5).abs() < 0.01);
    activate(&mut app, player, AbilityType::ArmorRepair);
    tick(&mut app, 300);
    let healed = app.world().get::<ShipStats>(player).unwrap().armor;
    assert!(healed > 0.0 && healed < app.world().get::<ShipStats>(player).unwrap().max_armor);
    tick(&mut app, 120);
    assert_eq!(
        app.world().get::<ShipStats>(player).unwrap().armor,
        healed,
        "repair stops on expiry"
    );
}

#[test]
fn disruptor_slows_cg_boss_movement_without_slowing_its_attack_clock() {
    use rebellion::games::caldari_gallente::{campaign::CGBossType, cg_campaign::*};
    let (mut app, player) = setup(GameState::BossFight);
    app.add_systems(Update, update_cg_boss);
    let boss = app
        .world_mut()
        .spawn((
            CGBoss {
                boss_type: CGBossType::PatrolCommander,
                health: 1000.0,
                max_health: 1000.0,
                current_phase: 1,
                total_phases: 2,
            },
            CGBossMovement {
                timer: 0.0,
                speed: 50.0,
            },
            CGBossAttack {
                fire_timer: 0.0,
                fire_rate: 1000.0,
            },
            EnemyStats {
                health: 1000.0,
                max_health: 1000.0,
                ..default()
            },
            Transform::from_xyz(0.0, -100.0, 0.0),
        ))
        .id();
    tick(&mut app, 6);
    let normal = app.world().get::<Transform>(boss).unwrap().translation.x;
    activate(&mut app, player, AbilityType::WarpDisruptor);
    app.world_mut()
        .get_mut::<CGBossMovement>(boss)
        .unwrap()
        .timer = 0.0;
    app.world_mut()
        .get_mut::<CGBossAttack>(boss)
        .unwrap()
        .fire_timer = 0.0;
    app.world_mut()
        .get_mut::<Transform>(boss)
        .unwrap()
        .translation
        .x = 0.0;
    tick(&mut app, 6);
    let slow = app.world().get::<Transform>(boss).unwrap().translation.x;
    assert!((slow / normal - 0.5).abs() < 0.01);
    assert!((app.world().get::<CGBossAttack>(boss).unwrap().fire_timer - 0.1).abs() < 0.001);
    assert_eq!(app.world().get::<CGBossMovement>(boss).unwrap().speed, 50.0);
}

#[test]
fn fired_shots_keep_their_bonus_after_ability_expiry_and_ship_movement() {
    for (kind, family, distance, expected) in [
        (AbilityType::CloseRange, WeaponType::Railgun, 150.0, 20.0),
        (AbilityType::Scorch, WeaponType::Laser, 600.0, 10.0),
    ] {
        let (mut app, player) = setup(GameState::BossFight);
        {
            let mut w = app.world_mut().get_mut::<Weapon>(player).unwrap();
            w.weapon_type = family;
            w.damage = 10.0;
            w.cooldown = 0.0;
        }
        activate(&mut app, player, kind);
        app.world_mut()
            .get_mut::<Ability>(player)
            .unwrap()
            .effect_remaining = 0.1;
        let target = app
            .world_mut()
            .spawn((
                Enemy,
                EnemyStats {
                    health: 1000.0,
                    max_health: 1000.0,
                    ..default()
                },
                Transform::from_xyz(0.0, -300.0 + distance, 0.0),
            ))
            .id();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        tick(&mut app, 2);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        for mut d in app
            .world_mut()
            .query_filtered::<&mut ProjectileDamage, With<PlayerProjectile>>()
            .iter_mut(app.world_mut())
        {
            d.crit_chance = 0.0;
        }
        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation
            .x = 350.0;
        tick(&mut app, 75);
        assert!(!app.world().get::<Ability>(player).unwrap().is_active);
        assert_eq!(
            1000.0 - app.world().get::<EnemyStats>(target).unwrap().health,
            expected
        );
    }
}
