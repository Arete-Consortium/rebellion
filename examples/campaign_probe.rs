//! Bounded, headless combat probe through the real chapter/faction/hull menus.
//! cargo run --offline --locked --example campaign_probe -- [output-directory] [--controller]
//! A scripted pilot uses ordinary movement/aim input. It never changes health,
//! weapon stats, drops, enemy lifetimes or mission completion. This measures a
//! bot run, not human difficulty, visual quality, frame rate or device support.
use bevy::prelude::*;
use rebellion::{
    app_builder::{build_headless_app, ControllerOnlyPlugin},
    core::*,
    entities::*,
    games::{caldari_gallente::CGCampaignState, GameModulesPlugin},
    systems::JoystickState,
    ui::{
        menu::{common::MenuSelection, MenuPlugin},
        TransitionPlugin,
    },
};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

#[derive(Default, Serialize)]
struct Mission {
    seconds: f32,
    boss_seconds: f32,
    completed: bool,
    kills: u32,
    bosses_defeated: u32,
    damage_taken: f32,
    last_defenses: [f32; 3],
    drops_spawned: BTreeMap<String, u32>,
    pickups: BTreeMap<String, u32>,
    boss_health_samples: Vec<f32>,
    #[serde(skip)]
    enemies_seen: HashSet<Entity>,
}

#[derive(Resource, Default)]
struct Probe {
    missions: [Mission; 3],
}

#[derive(Resource)]
struct ProbeController(Entity);

fn press_controller(app: &mut App, button: GamepadButton) {
    let entity = app.world().resource::<ProbeController>().0;
    app.world_mut()
        .get_mut::<Gamepad>(entity)
        .unwrap()
        .digital_mut()
        .press(button);
    app.update();
    app.world_mut()
        .get_mut::<Gamepad>(entity)
        .unwrap()
        .digital_mut()
        .reset_all();
    tick(app, 12);
}

fn tick(app: &mut App, frames: usize) {
    for _ in 0..frames {
        app.update();
    }
}

fn state(app: &App) -> GameState {
    *app.world().resource::<State<GameState>>().get()
}

fn confirm(app: &mut App) {
    if app.world().contains_resource::<ProbeController>() {
        press_controller(app, GamepadButton::South);
        tick(app, 65);
        return;
    }
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Enter);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .reset_all();
    tick(app, 65);
}

fn choose(app: &mut App, index: usize, expected: GameState) {
    if app.world().contains_resource::<ProbeController>() {
        for _ in 0..12 {
            if app.world().resource::<MenuSelection>().index == index {
                break;
            }
            press_controller(app, GamepadButton::DPadRight);
        }
        assert_eq!(app.world().resource::<MenuSelection>().index, index);
    } else {
        app.world_mut().resource_mut::<MenuSelection>().index = index;
    }
    confirm(app);
    assert_eq!(state(app), expected);
}

fn main() {
    let controller_only = std::env::args().any(|arg| arg == "--controller");
    let output = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "build/playtest-review/booster-combat-20260908".into());
    std::fs::create_dir_all(&output).unwrap();
    for (side, faction) in [(0, "caldari"), (1, "gallente")] {
        let profile = std::env::temp_dir().join(format!(
            "rebellion-combat-probe-{}-{faction}",
            std::process::id()
        ));
        std::fs::create_dir(&profile).unwrap();
        std::env::set_var("REBELLION_HOME", &profile);
        let mut app = build_headless_app();
        app.add_plugins((GameModulesPlugin, MenuPlugin, TransitionPlugin));
        app.add_event::<AppExit>();
        app.init_resource::<rebellion::systems::audio::SoundAssets>();
        app.init_resource::<rebellion::systems::touch_joystick::MobileMode>();
        app.init_resource::<Probe>();
        // Controller fixtures pass through real Gamepad polling and the player
        // connection gate. Legacy probes retain their direct input helper.
        app.add_systems(First, pilot);
        app.add_systems(FixedPostUpdate, observe_drops);
        app.add_systems(PostUpdate, observe);
        if controller_only {
            app.add_plugins(ControllerOnlyPlugin);
            let entity = app.world_mut().spawn(Gamepad::default()).id();
            app.insert_resource(ProbeController(entity));
            tick(&mut app, 2);
            confirm(&mut app); // Connect, acknowledge and release A.
        }
        tick(&mut app, 125);
        assert_eq!(state(&app), GameState::MainMenu);
        confirm(&mut app);
        assert_eq!(state(&app), GameState::ModuleSelect);
        choose(&mut app, 1, GameState::FactionSelect);
        choose(&mut app, side, GameState::DifficultySelect);
        choose(&mut app, 1, GameState::ShipSelect); // Newbro (normal)
        choose(&mut app, 0, GameState::MissionBriefing); // starter hull
        let hull = app
            .world()
            .resource::<GameSession>()
            .selected_ship()
            .name
            .to_string();
        confirm(&mut app);
        // At most 20 minutes of simulated time per faction. No retry/immortality.
        for _ in 0..72_000 {
            match state(&app) {
                GameState::StageComplete => {
                    let index = app.world().resource::<CGCampaignState>().mission_index;
                    app.world_mut().resource_mut::<Probe>().missions[index].completed = true;
                    confirm(&mut app);
                }
                GameState::SliceComplete | GameState::GameOver => break,
                _ => app.update(),
            }
        }
        let probe = app.world().resource::<Probe>();
        let report = serde_json::json!({
            "faction": faction, "hull": hull, "difficulty": "Newbro",
            "simulation_seed": rebellion::simulation::DEFAULT_MISSION_SEED,
            "content_rng": "unseeded thread-local RNG; runs vary",
            "outcome": format!("{:?}", state(&app)),
            "input_mode": if controller_only { "controller-only via Bevy Gamepad components" } else { "legacy headless input fixture" },
            "method": "scripted ordinary movement and aim; unmodified combat stats; default production executor; headless; no retries",
            "missions": probe.missions,
            "enemies_observed": probe.missions.iter().map(|m| m.enemies_seen.len()).collect::<Vec<_>>(),
        });
        let encoded = serde_json::to_string_pretty(&report).unwrap();
        std::fs::write(output.join(format!("{faction}.json")), &encoded).unwrap();
        println!("{encoded}");
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn pilot(
    state: Res<State<GameState>>,
    time: Res<Time>,
    mut joystick: ResMut<JoystickState>,
    controller: Option<Res<ProbeController>>,
    mut gamepads: Query<&mut Gamepad>,
    player: Query<(&Transform, &Movement), With<Player>>,
    enemies: Query<(&Transform, &EnemyStats), With<Enemy>>,
    bullets: Query<(&Transform, &ProjectilePhysics), With<EnemyProjectile>>,
    pickups: Query<&Transform, With<Collectible>>,
) {
    if controller.is_none() {
        *joystick = JoystickState::default();
    }
    let mut intent = JoystickState::default();
    if !matches!(*state.get(), GameState::Playing | GameState::BossFight) {
        return;
    }
    let Ok((transform, movement)) = player.get_single() else {
        return;
    };
    let position = transform.translation.truncate();
    let target = enemies
        .iter()
        .filter(|(_, s)| s.health > 0.0)
        .min_by(|(a, _), (b, _)| {
            a.translation
                .truncate()
                .distance_squared(position)
                .total_cmp(&b.translation.truncate().distance_squared(position))
        });
    if let Some((target, _)) = target {
        let aim = (target.translation.truncate() - position).normalize_or_zero();
        intent.right_x = aim.x;
        intent.right_y = aim.y; // Bevy: up is positive, as in game space.
        if controller.is_some() {
            intent.right_trigger = 1.0;
        }
    }
    let mut goal = Vec2::new((time.elapsed_secs() * 0.6).sin() * 220.0, -160.0);
    if let Some(pickup) = pickups
        .iter()
        .filter(|p| p.translation.y < 100.0)
        .min_by(|a, b| {
            a.translation
                .truncate()
                .distance_squared(position)
                .total_cmp(&b.translation.truncate().distance_squared(position))
        })
    {
        if pickup.translation.truncate().distance(position) < 240.0 {
            goal = pickup.translation.truncate();
        }
    }
    // Choose the lowest-risk short trajectory toward pickups/the patrol path.
    let mut best = (f32::INFINITY, Vec2::ZERO);
    for x in -1..=1 {
        for y in -1..=1 {
            let direction = Vec2::new(x as f32, y as f32).normalize_or_zero();
            let future = position + movement.velocity * 0.1 + direction * 65.0;
            if future.x.abs() > 345.0 || future.y < -290.0 || future.y > 120.0 {
                continue;
            }
            let mut cost = future.distance(goal);
            for (bullet, physics) in &bullets {
                for seconds in [0.15, 0.35, 0.6] {
                    let bullet_pos = bullet.translation.truncate() + physics.velocity * seconds;
                    let pilot_pos = position.lerp(future, (seconds / 0.6_f32).min(1.0));
                    let separation = bullet_pos.distance(pilot_pos);
                    if separation < 70.0 {
                        cost += (70.0 - separation).powi(2) * 0.6;
                    }
                }
            }
            if cost < best.0 {
                best = (cost, direction);
            }
        }
    }
    intent.left_x = best.1.x;
    intent.left_y = best.1.y;
    if let Some(controller) = controller {
        let mut pad = gamepads.get_mut(controller.0).unwrap();
        for (axis, value) in [
            (GamepadAxis::LeftStickX, intent.left_x),
            (GamepadAxis::LeftStickY, intent.left_y),
            (GamepadAxis::RightStickX, intent.right_x),
            (GamepadAxis::RightStickY, intent.right_y),
            (GamepadAxis::RightZ, intent.right_trigger),
        ] {
            pad.analog_mut().set(axis, value);
        }
    } else {
        *joystick = intent;
    }
}

#[allow(clippy::too_many_arguments)]
fn observe(
    time: Res<Time>,
    state: Res<State<GameState>>,
    campaign: Res<CGCampaignState>,
    mut probe: ResMut<Probe>,
    enemies: Query<(Entity, &EnemyStats), With<Enemy>>,
    player: Query<&ShipStats, With<Player>>,
    mut deaths: EventReader<EnemyDestroyedEvent>,
    mut damage: EventReader<PlayerDamagedEvent>,
    mut pickups: EventReader<CollectiblePickedUpEvent>,
    mut last_state: Local<Option<(GameState, usize)>>,
) {
    let current = (*state.get(), campaign.mission_index);
    if *last_state != Some(current) {
        println!(
            "probe: mission {} / {:?}",
            campaign.mission_index + 1,
            state.get()
        );
        *last_state = Some(current);
    }
    let Some(mission) = probe.missions.get_mut(campaign.mission_index) else {
        return;
    };
    if matches!(*state.get(), GameState::Playing | GameState::BossFight) {
        mission.seconds += time.delta_secs();
        if *state.get() == GameState::BossFight {
            mission.boss_seconds += time.delta_secs();
        }
        if let Ok(player) = player.get_single() {
            mission.last_defenses = [player.shield, player.armor, player.hull];
        }
        for (entity, stats) in &enemies {
            mission.enemies_seen.insert(entity);
            if stats.is_boss {
                let fraction = stats.health / stats.max_health;
                if mission
                    .boss_health_samples
                    .last()
                    .is_none_or(|last| last - fraction >= 0.1)
                {
                    mission.boss_health_samples.push(fraction);
                }
            }
        }
    }
    for event in deaths.read() {
        mission.kills += 1;
        if event.was_boss {
            mission.bosses_defeated += 1;
        }
    }
    for event in damage.read() {
        mission.damage_taken += event.shield_damage + event.armor_damage + event.hull_damage;
    }
    for event in pickups.read() {
        *mission
            .pickups
            .entry(format!("{:?}", event.collectible_type))
            .or_default() += 1;
    }
}

// Death outcomes spawn normal loot in FixedUpdate. Count it before Update can
// collect or expire it, including pickups grabbed immediately on a close kill.
fn observe_drops(
    campaign: Res<CGCampaignState>,
    mut probe: ResMut<Probe>,
    drops: Query<&CollectibleData, Added<CollectibleData>>,
) {
    let Some(mission) = probe.missions.get_mut(campaign.mission_index) else {
        return;
    };
    for drop in &drops {
        *mission
            .drops_spawned
            .entry(format!("{:?}", drop.collectible_type))
            .or_default() += 1;
    }
}
