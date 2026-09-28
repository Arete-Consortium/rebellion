//! Exercise real spawn -> attack -> damage paths for the complete enemy roster.
use super::*;
use bevy::ecs::{system::RunSystemOnce, world::CommandQueue};
use crate::core::{ContactRaw, ContactDetected, DamageLayerEvent, PlayerDamagedEvent};
use crate::entities::{Player, ShipStats, Hitbox, PowerupEffects, EnemyProjectile};
use crate::systems::ManeuverState;
use crate::simulation::{detect_collisions::detect_enemy_projectile_hits, resolve_damage::{enrich_contacts, resolve_enemy_projectile_damage}, enemy_contacts::*};

fn world() -> (World, Entity) {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));
    world.insert_resource(time);
    world.insert_resource(PlayerTracker::default());
    world.insert_resource(NextState::<GameState>::default());
    world.insert_resource(Events::<ContactRaw>::default());
    world.insert_resource(Events::<ContactDetected>::default());
    world.insert_resource(Events::<PlayerDamagedEvent>::default());
    world.insert_resource(Events::<DamageLayerEvent>::default());
    world.insert_resource(Events::<EnemyShipContact>::default());
    world.insert_resource(ShipContactGrace::default());
    let player = world.spawn((Player, Transform::default(), ShipStats::default(),
        Hitbox::default(), PowerupEffects::default(), ManeuverState::default())).id();
    (world, player)
}

fn spawn(world: &mut World, hull: u32, variant: Option<EnemyVariant>, pos: Vec2) -> Entity {
    let mut queue = CommandQueue::default();
    let mut commands = Commands::new(&mut queue, world);
    let entity = if let Some(variant) = variant {
        spawn_variant(&mut commands, variant, pos, None, None)
    } else { spawn_enemy(&mut commands, hull, pos, EnemyBehavior::Linear, None, None) };
    queue.apply(world);
    entity
}

fn assert_attack_hurts(hull: u32, variant: Option<EnemyVariant>) {
    let (mut world, player) = world();
    let enemy = spawn(&mut world, hull, variant, Vec2::new(0.0, 100.0));
    let before = world.get::<ShipStats>(player).unwrap().shield;
    if world.get::<DisintegratorRamp>(enemy).is_some() {
        assert!(world.get::<EnemyWeapon>(enemy).is_none(), "beam hull must not retain a zero-damage gun");
        world.run_system_once(systems::disintegrator_update).unwrap();
        assert_eq!(world.get::<ShipStats>(player).unwrap().shield, before, "lock-on grace");
        for _ in 0..8 { world.run_system_once(systems::disintegrator_update).unwrap(); }
    } else {
        world.get_mut::<EnemyWeapon>(enemy).expect("armed hull").cooldown = 0.0;
        world.run_system_once(systems::enemy_shooting).unwrap();
        let mut bullets = world.query_filtered::<&mut Transform, With<EnemyProjectile>>();
        let mut count = 0;
        for mut transform in bullets.iter_mut(&mut world) {
            // Place the emitted shot at impact, then use real collision/enrichment/resolution.
            transform.translation = Vec3::ZERO;
            count += 1;
        }
        assert!(count > 0, "hull {hull} must emit an attack");
        world.run_system_once(detect_enemy_projectile_hits).unwrap();
        world.run_system_once(enrich_contacts).unwrap();
        world.run_system_once(resolve_enemy_projectile_damage).unwrap();
    }
    assert!(world.get::<ShipStats>(player).unwrap().shield < before, "hull {hull} attack must damage shields");
    assert!(!world.resource::<Events<PlayerDamagedEvent>>().is_empty(), "damage must reach feedback and game-over consumers");
}

#[test]
fn every_generic_hull_has_a_working_damage_path() {
    for hull in [597,589,591,16236,24696,603,602,583,16238,24698,593,594,608,
        16240,24700,624,622,587,585,598,47269,47270,47271,49710,49711,52250,
        52252,52254,54731,54732,54733,999999] {
        assert_attack_hurts(hull, None);
    }
}

#[test]
fn every_specialized_variant_has_a_working_damage_path() {
    for variant in [EnemyVariant::Kamikaze, EnemyVariant::Weaver, EnemyVariant::Sniper,
        EnemyVariant::Spawner, EnemyVariant::Tank, EnemyVariant::Damavik,
        EnemyVariant::StarvingDamavik, EnemyVariant::Vedmak, EnemyVariant::BlindingVedmak,
        EnemyVariant::Kikimora, EnemyVariant::Leshak, EnemyVariant::DrekavacBoss,
        EnemyVariant::ExecutionerElite, EnemyVariant::PunisherTank, EnemyVariant::RifterBerserker] {
        assert_attack_hurts(variant.config().type_id, Some(variant));
    }
}

#[test]
fn rushing_ship_impacts_once_and_dies() {
    let (mut world, player) = world();
    let enemy = spawn(&mut world, 589, Some(EnemyVariant::Kamikaze), Vec2::ZERO);
    let before = world.get::<ShipStats>(player).unwrap().shield;
    world.run_system_once(detect_enemy_ship_contacts).unwrap();
    world.run_system_once(resolve_enemy_ship_contacts).unwrap();
    let after = world.get::<ShipStats>(player).unwrap().shield;
    assert!(after < before);
    assert_eq!(world.get::<EnemyStats>(enemy).unwrap().health, 0.0);
    assert!(!world.resource::<Events<DamageLayerEvent>>().is_empty());
    world.run_system_once(detect_enemy_ship_contacts).unwrap();
    world.run_system_once(resolve_enemy_ship_contacts).unwrap();
    assert_eq!(world.get::<ShipStats>(player).unwrap().shield, after);
}

#[test]
fn contact_respects_dodge_and_swarm_grace() {
    let (mut world, player) = world();
    spawn(&mut world, 597, None, Vec2::ZERO);
    spawn(&mut world, 603, None, Vec2::ZERO);
    let before = world.get::<ShipStats>(player).unwrap().shield;
    world.get_mut::<ManeuverState>(player).unwrap().invincible = true;
    world.run_system_once(detect_enemy_ship_contacts).unwrap();
    world.run_system_once(resolve_enemy_ship_contacts).unwrap();
    assert_eq!(world.get::<ShipStats>(player).unwrap().shield, before);
    world.get_mut::<ManeuverState>(player).unwrap().invincible = false;
    world.resource_mut::<Events<EnemyShipContact>>().clear();
    world.run_system_once(detect_enemy_ship_contacts).unwrap();
    world.run_system_once(resolve_enemy_ship_contacts).unwrap();
    let after = world.get::<ShipStats>(player).unwrap().shield;
    assert!(after < before);
    assert_eq!(world.resource::<Events<PlayerDamagedEvent>>().len(), 1, "swarm shares one impact grace period");
    world.run_system_once(resolve_enemy_ship_contacts).unwrap();
    assert_eq!(world.get::<ShipStats>(player).unwrap().shield, after);
}

#[test]
fn beam_breaks_lock_out_of_range_and_respects_invulnerability() {
    let (mut world, player) = world();
    let enemy = spawn(&mut world, 47269, None, Vec2::new(0.0, 100.0));
    let before = world.get::<ShipStats>(player).unwrap().shield;
    world.get_mut::<PowerupEffects>(player).unwrap().invuln_timer = 1.0;
    for _ in 0..10 { world.run_system_once(systems::disintegrator_update).unwrap(); }
    assert_eq!(world.get::<ShipStats>(player).unwrap().shield, before);
    world.get_mut::<Transform>(player).unwrap().translation.x = 1000.0;
    world.run_system_once(systems::disintegrator_update).unwrap();
    assert_eq!(world.get::<DisintegratorRamp>(enemy).unwrap().time_on_target, 0.0);
    world.get_mut::<Transform>(player).unwrap().translation.x = 0.0;
    world.get_mut::<PowerupEffects>(player).unwrap().invuln_timer = 0.0;
    world.run_system_once(systems::disintegrator_update).unwrap();
    assert_eq!(world.get::<ShipStats>(player).unwrap().shield, before, "returning to range must charge again");
}
