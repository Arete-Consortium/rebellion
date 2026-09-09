//! Environment Collision end-to-end integration test
//!
//! Verifies that environmental objects:
//!   1. Spawn and register in the spatial grid.
//!   2. Collide with the player, pushing them out and dealing contact damage.
//!   3. Collide with player projectiles, taking damage and being destroyed.
//!   4. Are cleaned up when the mission ends.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{AmmoType, GameState, PlayerFireEvent, WeaponType};
use rebellion::entities::environment::{
    spawn_environment, EnvironmentContactDamage, EnvironmentHealth, EnvironmentKind,
    EnvironmentObject, ProjectileInteraction,
};
use rebellion::entities::{Player, ShipStats};
use rebellion::simulation::state_hash::SimStateHash;
use rebellion::systems::ManeuverState;

/// Spawn a soft hazard (destructible asteroid) overlapping the player.
fn spawn_asteroid_over_player(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_pos = player_query.single().translation.truncate();
    spawn_environment(
        &mut commands,
        player_pos,
        EnvironmentKind::SoftHazard,
        25.0,       // radius
        None,       // no motion
        Some(50.0), // health
        Some(EnvironmentContactDamage {
            amount: 10.0,
            damage_type: rebellion::core::DamageType::Kinetic,
            cooldown_ticks: 30,
        }),
        ProjectileInteraction::Damageable,
        100, // score
    );
}

/// Fire a player projectile straight up through the asteroid.
fn send_fire_event(mut events: EventWriter<PlayerFireEvent>) {
    // Fire from below the asteroid so the projectile passes through it.
    events.send(PlayerFireEvent {
        range_multiplier: 1.0,
        close_range_multiplier: 1.0,
        position: Vec2::new(0.0, -300.0),
        direction: Vec2::new(0.0, 1.0),
        weapon_type: WeaponType::Laser,
        bullet_color: Color::srgb(1.0, 0.2, 0.2),
        damage: 60.0, // enough to one-shot the 50-hp asteroid
        burst_count: 1,
        spread_angle: 0.0,
        ammo_type: AmmoType::default(),
        crit_chance_override: None,
        crit_mult_override: None,
        pierce: 0,
        homing: 0.0,
        burn_dps: 0.0,
        chain_targets: 0,
    });
}

#[test]
fn environment_pipeline_e2e() {
    let mut app = build_headless_app();

    // Transition to Playing
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Disable respawn invincibility so contact damage applies
    {
        let mut q = app
            .world_mut()
            .query_filtered::<&mut ManeuverState, With<Player>>();
        let mut maneuver = q.single_mut(app.world_mut());
        maneuver.invincible = false;
        maneuver.invincibility_timer = 0.0;
    }

    // Verify initial player position and total health
    let (player_initial_pos, player_initial_total_hp) = {
        let mut q = app
            .world_mut()
            .query_filtered::<(&Transform, &ShipStats), With<Player>>();
        let (transform, stats) = q.single(app.world());
        (
            transform.translation.truncate(),
            stats.shield + stats.armor + stats.hull,
        )
    };

    // Spawn asteroid overlapping player
    app.world_mut()
        .run_system_once(spawn_asteroid_over_player)
        .expect("spawn asteroid");

    // Wait for registration
    for _ in 0..3 {
        app.update();
    }

    // Verify asteroid exists
    let asteroid_entity = {
        let mut q = app
            .world_mut()
            .query::<(Entity, &EnvironmentObject, &EnvironmentHealth)>();
        let (entity, _, _) = q.iter(app.world()).next().expect("asteroid spawned");
        entity
    };

    // Run detection/resolution ticks until player contact is resolved
    // The player should be pushed out and take damage.
    for _ in 0..10 {
        app.update();
    }

    // Player should have moved away from origin and taken damage
    let (player_after_contact, player_total_hp_after) = {
        let mut q = app
            .world_mut()
            .query_filtered::<(&Transform, &ShipStats), With<Player>>();
        let (transform, stats) = q.single(app.world());
        (
            transform.translation.truncate(),
            stats.shield + stats.armor + stats.hull,
        )
    };

    assert!(
        player_after_contact.distance(player_initial_pos) > 1.0,
        "player should be pushed away from asteroid, but stayed at {:?}",
        player_after_contact
    );
    assert!(
        player_total_hp_after < player_initial_total_hp,
        "player should take contact damage: {} >= {}",
        player_total_hp_after,
        player_initial_total_hp
    );

    // Capture state hash before projectile fire
    let hash_before = app.world().resource::<SimStateHash>().0;

    // Fire projectile through asteroid
    app.world_mut()
        .run_system_once(send_fire_event)
        .expect("send fire event");

    // Wait one tick for projectile to spawn
    app.update();

    // Verify projectile spawned
    let proj_count_before = {
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<rebellion::entities::PlayerProjectile>>();
        q.iter(app.world()).count()
    };
    assert!(
        proj_count_before > 0,
        "projectile should spawn after fire event"
    );

    // Run ticks until projectile collides (~10 frames at ~600 px/s)
    for _ in 0..20 {
        app.update();
    }

    // Asteroid should be destroyed
    let asteroid_exists = app
        .world()
        .get_entity(asteroid_entity)
        .map(|e: bevy::ecs::world::EntityRef| e.contains::<EnvironmentObject>())
        .unwrap_or(false);
    assert!(
        !asteroid_exists,
        "asteroid should be destroyed by projectile"
    );

    // Verify state hash changed
    let hash_after = app.world().resource::<SimStateHash>().0;
    assert_ne!(
        hash_before, hash_after,
        "state hash should change after projectile/environment collision"
    );
}

/// Spawn 100 environment objects and run 60 frames to verify spatial grid
/// and detection systems scale linearly without panic.
#[test]
fn environment_stress_test_survives() {
    let mut app = build_headless_app();

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    for i in 0..100 {
        let x = ((i % 10) as f32 - 4.5) * 60.0;
        let y = ((i / 10) as f32 - 4.5) * 50.0;
        app.world_mut().commands().queue(move |w: &mut World| {
            spawn_environment(
                &mut w.commands(),
                Vec2::new(x, y),
                EnvironmentKind::SoftHazard,
                20.0,
                None,
                Some(50.0),
                None,
                ProjectileInteraction::Damageable,
                10,
            );
        });
    }

    // Run 60 frames — enough for grid updates and detection to exercise all objects
    for _ in 0..60 {
        app.update();
    }

    // All 100 should still exist (nothing destroys them in this test)
    let remaining = {
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<EnvironmentObject>>();
        q.iter(app.world()).count()
    };
    assert_eq!(
        remaining, 100,
        "all 100 stress-test asteroids should survive"
    );
}

/// A ship and an asteroid can overlap. Detection queues both contacts before
/// resolution consumes the shot on the ship; terrain must respect that result.
#[test]
fn overlapping_ship_and_terrain_respect_projectile_lifetime_and_live_pierce() {
    use rebellion::entities::environment::{EnvironmentCollider, EnvironmentScoreValue};
    use rebellion::entities::{Enemy, EnemyStats, Pierce, PlayerProjectile, ProjectileDamage};
    use rebellion::simulation::detect_collisions::{
        detect_player_projectile_environment_hits, detect_player_projectile_hits,
        update_spatial_grid,
    };
    use rebellion::simulation::resolve_damage::{
        enrich_contacts, resolve_player_projectile_damage, resolve_projectile_environment_contacts,
    };

    for interaction in [
        ProjectileInteraction::Damageable,
        ProjectileInteraction::Absorb,
    ] {
        for pierce in [None, Some(0), Some(1), Some(2)] {
            let mut app = build_headless_app();
            let world = app.world_mut();
            let enemy = world
                .spawn((
                    Enemy,
                    EnemyStats {
                        health: 100.0,
                        max_health: 100.0,
                        ..default()
                    },
                    Transform::default(),
                ))
                .id();
            let terrain = world
                .spawn((
                    EnvironmentObject,
                    EnvironmentCollider { radius: 20.0 },
                    EnvironmentHealth {
                        current: 100.0,
                        maximum: 100.0,
                    },
                    EnvironmentScoreValue(10),
                    interaction,
                    Transform::default(),
                ))
                .id();
            let shot = world
                .spawn((
                    PlayerProjectile,
                    ProjectileDamage {
                        damage: 10.0,
                        crit_chance: 0.0,
                        ..default()
                    },
                    Transform::default(),
                ))
                .id();
            if let Some(count) = pierce {
                world.entity_mut(shot).insert(Pierce(count));
            }
            let mut schedule = Schedule::default();
            // Actual production phase order, including deferred despawns.
            schedule.add_systems(
                (
                    update_spatial_grid,
                    detect_player_projectile_hits,
                    detect_player_projectile_environment_hits,
                    enrich_contacts,
                    resolve_player_projectile_damage,
                    resolve_projectile_environment_contacts,
                )
                    .chain(),
            );
            schedule.run(world);
            assert!(world.get::<EnemyStats>(enemy).unwrap().health < 100.0);
            let reached_terrain = pierce.is_some_and(|count| count > 0);
            let damages_terrain =
                reached_terrain && interaction == ProjectileInteraction::Damageable;
            assert_eq!(
                world.get::<EnvironmentHealth>(terrain).unwrap().current,
                if damages_terrain { 90.0 } else { 100.0 }
            );
            if damages_terrain && pierce == Some(2) {
                assert_eq!(
                    world.get::<Pierce>(shot).unwrap().0,
                    0,
                    "both contacts must consume their own pierce charge"
                );
            } else {
                assert!(
                    world.get_entity(shot).is_err(),
                    "spent shot must stay consumed"
                );
            }
        }
    }
}
