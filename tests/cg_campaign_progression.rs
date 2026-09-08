//! Regressions for the production CG wave and boss-completion system ordering.

use std::time::Duration;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use rebellion::app_builder::build_headless_app;
use rebellion::core::{AmmoType, DamageType, Difficulty, EnemyDestroyedEvent, GameState};
use rebellion::entities::{Enemy, EnemyStats, PlayerProjectile, ProjectileDamage};
use rebellion::games::caldari_gallente::campaign::{CGCampaignState, CG_INTER_WAVE_DELAY};
use rebellion::games::caldari_gallente::cg_campaign::{
    check_cg_boss_defeated, check_cg_wave_complete, spawn_cg_boss, spawn_cg_wave, CGBoss,
    CGSpawnTelegraph,
};
use rebellion::simulation::detect_collisions::{
    detect_player_projectile_hits, update_spatial_grid,
};
use rebellion::simulation::resolve_damage::{enrich_contacts, resolve_player_projectile_damage};
use rebellion::simulation::resolve_deaths::resolve_enemy_deaths;

fn campaign_app(mission_index: usize) -> App {
    // Use the production headless builder for private asset caches and combat
    // resources. Run focused schedules below so wave timing remains explicit.
    let mut app = build_headless_app();
    let mut campaign = CGCampaignState {
        mission_index,
        ..default()
    };
    campaign.start_mission();
    app.insert_resource(campaign);
    app.insert_resource(Time::<()>::default());
    app.insert_resource(Difficulty::Newbro);
    app.insert_resource(NextState::<GameState>::Unchanged);
    app
}

fn wave_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    // Keep the same check-before-spawn order used by CaldariGallentePlugin.
    schedule.add_systems((check_cg_wave_complete, spawn_cg_wave).chain());
    schedule
}

fn tick(world: &mut World, schedule: &mut Schedule, seconds: f32) {
    world
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
    schedule.run(world);
}

fn count<T: Component>(world: &mut World) -> usize {
    world
        .query_filtered::<Entity, With<T>>()
        .iter(world)
        .count()
}

fn clear_enemies(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<Enemy>>()
        .iter(world)
        .collect();
    for entity in entities {
        world.entity_mut(entity).despawn_recursive();
    }
}

#[test]
fn clearing_first_wave_spawns_second_after_one_delay() {
    let mut app = campaign_app(0);
    let world = app.world_mut();
    let mut schedule = wave_schedule();
    tick(world, &mut schedule, 1.0 / 60.0);
    assert_eq!(count::<Enemy>(world), 4, "first wave is immediate");
    clear_enemies(world);

    // Most of the breather elapses without spawning enemies.
    tick(world, &mut schedule, CG_INTER_WAVE_DELAY - 0.1);
    assert_eq!(count::<Enemy>(world), 0);

    // Expiry must spawn in this same update, before the completion check
    // has an opportunity to re-arm the timer on the following frame.
    tick(world, &mut schedule, 0.2);
    assert_eq!(count::<Enemy>(world), 5, "wave two must spawn");
    assert_eq!(world.resource::<CGCampaignState>().current_wave, 3);
    assert_eq!(world.resource::<CGCampaignState>().wave_delay_timer, 0.0);
    assert_eq!(count::<CGSpawnTelegraph>(world), 0);
}

#[test]
fn wave_telegraphs_spawn_once_and_match_upcoming_enemy_count() {
    let mut app = campaign_app(0);
    let world = app.world_mut();
    let mut schedule = wave_schedule();
    tick(world, &mut schedule, 1.0 / 120.0);
    clear_enemies(world);

    // Several high-refresh frames used to duplicate the warning indicators.
    for _ in 0..5 {
        tick(world, &mut schedule, 1.0 / 120.0);
        assert_eq!(count::<Enemy>(world), 0);
        assert_eq!(count::<CGSpawnTelegraph>(world), 5);
    }
    tick(world, &mut schedule, CG_INTER_WAVE_DELAY);
    assert_eq!(count::<Enemy>(world), 5);
    assert_eq!(count::<CGSpawnTelegraph>(world), 0);
}

#[test]
fn all_regular_waves_reach_the_mission_completion_state() {
    for mission_index in [0, 1] {
        let mut app = campaign_app(mission_index);
        let world = app.world_mut();
        let mut schedule = wave_schedule();
        let mission = world
            .resource::<CGCampaignState>()
            .current_mission()
            .unwrap();
        for wave in 1..=mission.waves {
            tick(world, &mut schedule, CG_INTER_WAVE_DELAY + 0.1);
            assert!(count::<Enemy>(world) > 0, "missing wave {wave}");
            clear_enemies(world);
        }
        tick(world, &mut schedule, 1.0 / 60.0);
        let expected = if mission.boss.is_some() {
            GameState::BossIntro
        } else {
            GameState::StageComplete
        };
        assert!(matches!(
            world.resource::<NextState<GameState>>(),
            NextState::Pending(state) if *state == expected
        ));
    }
}

#[test]
fn lethal_projectile_completes_boss_mission_after_simulation_despawns_boss() {
    let mut app = campaign_app(1);
    let world = app.world_mut();
    world.run_system_once(spawn_cg_boss).expect("spawn CG boss");
    let (boss_entity, position, health) = world
        .query_filtered::<(Entity, &Transform, &EnemyStats), With<CGBoss>>()
        .iter(world)
        .map(|(entity, transform, stats)| (entity, transform.translation, stats.health))
        .next()
        .expect("mission two has a boss");
    world.spawn((
        PlayerProjectile,
        ProjectileDamage {
            damage: health * 10.0,
            damage_type: DamageType::EM,
            crit_chance: 0.0,
            crit_multiplier: 1.0,
            ammo_type: AmmoType::default(),
        },
        Transform::from_translation(position),
    ));

    // Run the real detection/damage/death pipeline and flush its commands,
    // exactly as FixedUpdate finishes before the campaign's Update system.
    let mut simulation = Schedule::default();
    simulation.add_systems(
        (
            update_spatial_grid,
            detect_player_projectile_hits,
            enrich_contacts,
            resolve_player_projectile_damage,
            resolve_enemy_deaths,
        )
            .chain(),
    );
    simulation.run(world);
    assert!(
        world.get_entity(boss_entity).is_err(),
        "simulation owns despawn"
    );
    assert_eq!(count::<CGBoss>(world), 0);
    let deaths = world.resource::<Events<EnemyDestroyedEvent>>();
    let mut reader = deaths.get_cursor();
    let death = reader
        .read(deaths)
        .next()
        .expect("simulation emits a death");
    assert_eq!(death.enemy, boss_entity);
    assert!(
        death.was_boss,
        "spawned CG boss must be classified as a boss"
    );
    assert_eq!(
        death.enemy_type,
        world
            .resource::<CGCampaignState>()
            .current_mission()
            .unwrap()
            .boss
            .unwrap()
            .name(),
        "death identity must match the current campaign boss"
    );

    let mut campaign_update = Schedule::default();
    campaign_update.add_systems(check_cg_boss_defeated);
    campaign_update.run(world);
    assert!(world.resource::<CGCampaignState>().boss_defeated);
    assert!(matches!(
        world.resource::<NextState<GameState>>(),
        NextState::Pending(GameState::StageComplete)
    ));

    world.insert_resource(NextState::<GameState>::Unchanged);
    campaign_update.run(world);
    assert!(
        matches!(
            world.resource::<NextState<GameState>>(),
            NextState::Unchanged
        ),
        "the defeat is handled once"
    );
}

#[test]
fn missing_boss_without_destruction_does_not_complete_mission() {
    let mut app = campaign_app(1);
    let world = app.world_mut();
    world.resource_mut::<CGCampaignState>().boss_spawned = true;
    world
        .run_system_once(check_cg_boss_defeated)
        .expect("check CG boss defeat");
    assert!(!world.resource::<CGCampaignState>().boss_defeated);
    assert!(matches!(
        world.resource::<NextState<GameState>>(),
        NextState::Unchanged
    ));
}
