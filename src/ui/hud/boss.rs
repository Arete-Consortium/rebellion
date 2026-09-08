//! Boss display reads the same health component that projectile damage mutates.
use super::common::*;
use crate::entities::{Boss, BossData, BossState, EnemyStats};
use crate::games::caldari_gallente::cg_campaign::CGBoss;
use bevy::prelude::*;

pub fn update_boss_health_bar(
    bosses: Query<
        (
            Option<&BossData>,
            Option<&BossState>,
            Option<&CGBoss>,
            Option<&EnemyStats>,
        ),
        Or<(With<Boss>, With<CGBoss>)>,
    >,
    mut containers: Query<&mut Node, With<BossHealthContainer>>,
    mut fills: Query<&mut Node, (With<BossHealthFill>, Without<BossHealthContainer>)>,
    mut names: Query<&mut Text, With<BossNameText>>,
) {
    let boss = bosses.get_single().ok();
    for mut node in &mut containers {
        node.display = if boss.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
    let Some((legacy, state, cg, stats)) = boss else {
        return;
    };
    let (health, max_health) = if let Some(stats) = stats {
        (stats.health, stats.max_health)
    } else if let Some(data) = legacy {
        (data.health, data.max_health)
    } else if let Some(data) = cg {
        (data.health, data.max_health)
    } else {
        return;
    };
    let fraction = (health / max_health.max(1.0)).clamp(0.0, 1.0);
    for mut node in &mut fills {
        node.width = Val::Percent(fraction * 100.0);
    }
    let (name, phase, total) = if let Some(data) = legacy {
        (data.name.as_str(), data.current_phase, data.total_phases)
    } else if let Some(data) = cg {
        (data.boss_type.name(), data.current_phase, data.total_phases)
    } else {
        return;
    };
    for mut text in &mut names {
        **text = if matches!(state, Some(BossState::Defeated)) {
            format!("{name} DEFEATED")
        } else {
            format!(
                "{name} | Phase {phase}/{total} | {:.0}/{:.0}",
                health.max(0.0),
                max_health
            )
        };
    }
}

#[cfg(test)]
mod damage_tests {
    use super::*;
    use crate::core::*;
    use crate::entities::{Enemy, PlayerProjectile, ProjectileDamage};
    use crate::games::elder_fleet::{
        check_ef_boss_defeated, spawn_ef_boss, ElderFleetCampaignState,
    };
    use crate::simulation::detect_collisions::{
        detect_player_projectile_hits, update_spatial_grid,
    };
    use crate::simulation::resolve_damage::{enrich_contacts, resolve_player_projectile_damage};
    use crate::simulation::resolve_deaths::resolve_enemy_deaths;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn destroyer_commander_takes_projectile_damage_bar_depletes_and_mission_advances() {
        for (side, expected) in [("minmatar", 16236), ("amarr", 16242)] {
            let mut app = crate::app_builder::build_headless_app();
            app.add_plugins(crate::games::GameModulesPlugin);
            let world = app.world_mut();
            world
                .resource_mut::<crate::games::ActiveModule>()
                .set_module("elder_fleet");
            world
                .resource_mut::<crate::games::ActiveModule>()
                .set_faction(side, "enemy");
            world.resource_mut::<ElderFleetCampaignState>().boss_spawned = true;
            world.run_system_once(spawn_ef_boss).unwrap();
            let (boss, pos, max) = world
                .query_filtered::<(Entity, &Transform, &EnemyStats), With<Boss>>()
                .iter(world)
                .map(|(e, t, s)| {
                    assert_eq!(s.type_id, expected);
                    (e, t.translation, s.max_health)
                })
                .next()
                .unwrap();
            assert!(
                world.get::<Enemy>(boss).is_some(),
                "commander must be in normal collision pipeline"
            );
            let fill = world.spawn((BossHealthFill, Node::default())).id();
            world.spawn((BossHealthContainer, Node::default()));
            world.spawn((BossNameText, Text::new("")));
            let mut damage = Schedule::default();
            damage.add_systems(
                (
                    update_spatial_grid,
                    detect_player_projectile_hits,
                    enrich_contacts,
                    resolve_player_projectile_damage,
                    resolve_enemy_deaths,
                )
                    .chain(),
            );
            for amount in [max * 0.25, max * 10.0] {
                world.spawn((
                    PlayerProjectile,
                    ProjectileDamage {
                        damage: amount,
                        damage_type: DamageType::EM,
                        crit_chance: 0.0,
                        crit_multiplier: 1.0,
                        ammo_type: AmmoType::default(),
                    },
                    Transform::from_translation(pos),
                ));
                damage.run(world);
                world.run_system_once(update_boss_health_bar).unwrap();
                if amount < max {
                    let remaining = world.get::<EnemyStats>(boss).unwrap().health;
                    assert!(remaining > 0.0 && remaining < max);
                    assert_eq!(
                        world.get::<Node>(fill).unwrap().width,
                        Val::Percent(remaining / max * 100.0)
                    );
                }
            }
            assert!(world.get_entity(boss).is_err());
            world.run_system_once(check_ef_boss_defeated).unwrap();
            assert_eq!(
                world.resource::<ElderFleetCampaignState>().current_mission,
                1
            );
        }
    }
}
