//! Mission 3 (Fleet Interdiction) stress test
//!
//! Validates that Mission 3 — the heaviest scenario in the vertical slice —
//! does not spawn an unbounded number of entities. Acts as a proxy for
//! frame-time stability: if entity counts explode, frame times will follow.

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;

use rebellion::app_builder::build_headless_app;
use rebellion::core::{Faction, GameSession, GameState};
use rebellion::entities::Enemy;
use rebellion::games::caldari_gallente::campaign::CGCampaignState;
use rebellion::games::caldari_gallente::{CaldariGallenteShips, VerticalSliceMode};
use rebellion::games::ActiveModule;

fn setup_cg_mission3(app: &mut App) {
    app.init_resource::<CGCampaignState>();
    app.init_resource::<CaldariGallenteShips>();
    app.init_resource::<VerticalSliceMode>();

    {
        let mut active = app.world_mut().resource_mut::<ActiveModule>();
        active.set_module("caldari_gallente");
        active.set_faction("gallente", "caldari");
    }
    {
        let mut session = app.world_mut().resource_mut::<GameSession>();
        session.enemy_faction = Faction::Caldari;
    }
    {
        let mut cg = app.world_mut().resource_mut::<CGCampaignState>();
        *cg = CGCampaignState::default();
        cg.mission_index = 2; // Mission 3
        cg.in_mission = true;
        cg.current_wave = 1;
    }
}

/// Despawn all enemies so the next wave can spawn.
fn clear_all_enemies(app: &mut App) {
    let to_despawn: Vec<Entity> = {
        let mut q = app.world_mut().query::<(Entity, &Enemy)>();
        q.iter(app.world()).map(|(e, _)| e).collect()
    };
    for e in to_despawn {
        app.world_mut().despawn(e);
    }
    app.update(); // flush despawn commands
}

#[test]
fn mission3_all_waves_spawn_bounded_enemies() {
    let mut app = build_headless_app();
    setup_cg_mission3(&mut app);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // start_cg_mission runs

    // Mission 3 has 5 waves. Spawn each and assert bounded counts.
    for wave in 1..=5 {
        app.world_mut()
            .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_wave)
            .expect("spawn_cg_wave should run");
        app.update(); // flush spawn commands

        let enemy_count = app.world_mut().query::<&Enemy>().iter(app.world()).count();

        assert!(
            enemy_count > 0,
            "Mission 3 wave {wave} should spawn enemies"
        );
        assert!(
            enemy_count <= 12,
            "Mission 3 wave {wave} spawned too many enemies ({enemy_count}), expected ≤12"
        );

        clear_all_enemies(&mut app);
    }
}

#[test]
fn mission3_boss_spawn_does_not_leak() {
    let mut app = build_headless_app();
    setup_cg_mission3(&mut app);

    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Spawn all 5 waves and clear them so boss intro can trigger
    for _ in 1..=5 {
        app.world_mut()
            .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_wave)
            .expect("spawn_cg_wave should run");
        app.update();
        clear_all_enemies(&mut app);
    }

    // Advance wave counter past final wave so boss spawns
    {
        let mut cg = app.world_mut().resource_mut::<CGCampaignState>();
        cg.current_wave = 6; // past wave 5
    }

    // Trigger boss spawn via state transition (BossIntro -> BossFight)
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::BossIntro);
    app.update();

    // In BossIntro, the boss should be spawned by OnEnter(BossIntro) or
    // by spawn_cg_boss. Run spawn_cg_boss explicitly since GameModulesPlugin
    // isn't present in headless tests.
    app.world_mut()
        .run_system_once(rebellion::games::caldari_gallente::cg_campaign::spawn_cg_boss)
        .expect("spawn_cg_boss should run");
    app.update(); // flush commands

    let total_entities = app.world().entities().total_count();
    assert!(
        total_entities < 40,
        "Mission 3 boss spawn should not leak entities (got {total_entities} total)"
    );

    // Run a few simulation ticks to ensure no runaway spawning
    for _ in 0..10 {
        app.update();
    }

    let total_after_ticks = app.world().entities().total_count();
    assert!(
        total_after_ticks < 40,
        "Entity count should stay bounded after ticks (got {total_after_ticks})"
    );
}
