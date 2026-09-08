//! Native pickup/HUD review with an isolated muted save. No review code ships.
//! cargo run --offline --locked --example powerup_review
use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};
use rebellion::{
    app_builder::RebellionAppConfig,
    core::*,
    entities::{
        collectible::spawn_collectible, CollectiblePhysics, Player, PowerupEffects, ShipStats,
    },
    games::ActiveModule,
    PowerupIconCache,
};
use std::path::PathBuf;

#[derive(Resource)]
struct Review {
    output: PathBuf,
    phase: u8,
    seconds: f32,
}

#[derive(Component)]
struct ReviewPickup(Vec2);

fn main() {
    let output = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap()
                .join("build/playtest-review/classic-boosters-20260908")
        });
    std::fs::create_dir_all(&output).unwrap();
    let profile =
        std::env::temp_dir().join(format!("rebellion-powerup-review-{}", std::process::id()));
    std::fs::create_dir(&profile).unwrap();
    std::fs::write(profile.join("save.json"), r#"{"stage_progress":[],"unlocked_ships":[],"lifetime_credits":0,"high_scores":[],"settings":{"master_volume":0,"music_volume":0,"sfx_volume":0}}"#).unwrap();
    std::env::set_var("REBELLION_HOME", profile);
    let mut app = RebellionAppConfig::native().build();
    for mut window in app
        .world_mut()
        .query::<&mut Window>()
        .iter_mut(app.world_mut())
    {
        window.title = "Rebellion — isolated booster review".into();
        window.focused = false;
    }
    app.insert_resource(Review {
        output,
        phase: 0,
        seconds: 0.0,
    })
    .add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
    })
    .add_systems(PostUpdate, review)
    .run();
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn review(
    mut commands: Commands,
    mut review: ResMut<Review>,
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    mut active: ResMut<ActiveModule>,
    mut session: ResMut<GameSession>,
    mut dialogue: ResMut<rebellion::systems::dialogue::DialogueSystem>,
    cache: Res<PowerupIconCache>,
    images: Res<Assets<Image>>,
    mut collected: EventWriter<CollectiblePickedUpEvent>,
    mut player: Query<
        (&mut Transform, &PowerupEffects, &mut ShipStats),
        (With<Player>, Without<ReviewPickup>),
    >,
    mut pickups: Query<(&ReviewPickup, &mut Transform, &mut CollectiblePhysics), Without<Player>>,
    mut exit: EventWriter<AppExit>,
) {
    review.seconds += time.delta_secs();
    // Keep the labeled pickup review clear of the opening story panel.
    dialogue.clear();
    dialogue.queue.clear();
    if review.phase == 0 && *state.get() == GameState::MainMenu {
        active.set_module("elder_fleet");
        active.player_faction = Some("minmatar".into());
        *session = GameSession::new(Faction::Minmatar, Faction::Amarr);
        next.set(GameState::Playing);
        review.phase = 1;
        review.seconds = 0.0;
    }
    if review.phase == 1
        && *state.get() == GameState::Playing
        && review.seconds > 0.5
        && cache.icons.len() == 14
        && cache
            .icons
            .values()
            .all(|handle| images.get(handle).is_some())
    {
        let gallery = [
            (CollectibleType::ShieldBoost, "BLUE PILL\nShield +25"),
            (CollectibleType::ArmorRepair, "EXILE\nArmor +25"),
            (CollectibleType::CapacitorCharge, "MINDFLOOD\nCapacitor +50"),
            (
                CollectibleType::Invulnerability,
                "X-INSTINCT\nInvulnerability / 3s",
            ),
            (CollectibleType::Overdrive, "OVERCLOCKER\nSpeed / 5s"),
            (CollectibleType::DamageBoost, "PYROLANCEA\nDamage x2 / 10s"),
            (CollectibleType::HullRepair, "HARDSHELL I\nHull +25"),
            (CollectibleType::ExtraLife, "HARDSHELL\nFull repair"),
            (CollectibleType::Nanite, "SUNYATA\nHeat -50"),
        ];
        for (i, (kind, label)) in gallery.into_iter().enumerate() {
            let icon = images.get(&cache.get(&kind).unwrap()).unwrap();
            assert_eq!((icon.width(), icon.height()), (64, 64));
            assert!(icon.data.chunks_exact(4).any(|pixel| pixel[3] == 0));
            assert!(icon.data.chunks_exact(4).any(|pixel| pixel[3] == 255));
            let pos = Vec2::new(
                -285.0 + (i % 4) as f32 * 155.0,
                45.0 - (i / 4) as f32 * 105.0,
            );
            // Use the real collectible spawn path; pin motion only for inspection.
            spawn_collectible(&mut commands, pos, kind, Some(&cache));
            commands.queue(move |world: &mut World| {
                let entity = world
                    .query::<(Entity, &Transform, &CollectiblePhysics)>()
                    .iter(world)
                    .find(|(_, transform, _)| transform.translation.truncate() == pos)
                    .map(|(entity, _, _)| entity)
                    .expect("review pickup spawned");
                world.entity_mut(entity).insert(ReviewPickup(pos));
            });
            commands.spawn((
                Text2d::new(label),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                Transform::from_xyz(pos.x, pos.y - 34.0, 100.0),
            ));
        }
        for kind in [
            CollectibleType::Overdrive,
            CollectibleType::DamageBoost,
            CollectibleType::Invulnerability,
        ] {
            collected.send(CollectiblePickedUpEvent {
                collectible_type: kind,
                position: Vec2::new(0.0, -240.0),
                value: 1,
            });
        }
        review.phase = 2;
        review.seconds = 0.0;
    }
    for (point, mut transform, mut physics) in &mut pickups {
        transform.translation.x = point.0.x;
        transform.translation.y = point.0.y;
        physics.lifetime = 10.0;
    }
    if let Ok((mut transform, _, mut stats)) = player.get_single_mut() {
        transform.translation.x = 0.0;
        transform.translation.y = -240.0;
        // This stationary review pilot must outlive the real buff timers.
        stats.max_hull = 1_000_000.0;
        stats.hull = stats.max_hull;
    }
    if review.phase == 2 && review.seconds > 0.7 {
        let (_, effects, _) = player.single();
        assert!(
            effects.overdrive_timer > 0.0
                && effects.damage_boost_timer > 0.0
                && effects.invuln_timer > 0.0
        );
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(review.output.join("boosters.png")));
        review.phase = 3;
    }
    if review.phase == 3 && review.seconds > 4.0 {
        // Renew Overclocker after its bar has entered the low-time warning.
        collected.send(CollectiblePickedUpEvent {
            collectible_type: CollectibleType::Overdrive,
            position: Vec2::new(0.0, -240.0),
            value: 1,
        });
        review.phase = 4;
    }
    if review.phase == 4 && review.seconds > 4.4 {
        assert!(player.single().1.overdrive_timer > 3.5);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(review.output.join("refreshed.png")));
        review.phase = 5;
    }
    if review.phase == 5 && review.seconds > 11.0 {
        let (_, effects, _) = player.single();
        assert!(
            effects.overdrive_timer <= 0.0
                && effects.damage_boost_timer <= 0.0
                && effects.invuln_timer <= 0.0
        );
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(review.output.join("expired.png")));
        review.phase = 6;
    }
    if review.phase == 6 && review.seconds > 12.0 {
        std::fs::write(review.output.join("review-result.json"),
            r#"{"loaded_icons":14,"real_pickup_spawn_path":true,"activation_refresh_and_expiration_verified":true,"scripted_not_human_playtest":true}"#).unwrap();
        exit.send(AppExit::Success);
    } else if review.seconds > 30.0 {
        panic!("booster review timed out in phase {}", review.phase);
    }
}
