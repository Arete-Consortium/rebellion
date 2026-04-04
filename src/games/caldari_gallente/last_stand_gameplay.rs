//! Last Stand Gameplay & HUD Systems
//!
//! CNS Kairiola titan defense mode - spawning, HUD, input, and fighter AI.

use super::last_stand::{
    DoomsdayBeam, EcmBurst, LastStandAction, LastStandEvent, LastStandState, TitanFighter,
};
use crate::core::{GameState, LAYER_PLAYER_BULLETS};
use crate::systems::JoystickState;
use bevy::prelude::*;

/// Marker for Last Stand HUD elements
#[derive(Component)]
pub struct LastStandHud;

/// Marker for the titan entity
#[derive(Component)]
pub struct LastStandTitan;

/// HUD element types for Last Stand
#[derive(Component)]
pub enum LastStandHudElement {
    Heat,
    Evacuation,
    Shield,
    Armor,
    Hull,
    Fighters,
    Ability(LastStandAction),
    Message,
}

/// Spawn the Last Stand mode (titan + HUD)
pub fn spawn_last_stand(
    mut commands: Commands,
    last_stand: Res<LastStandState>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
) {
    if !last_stand.active {
        return;
    }

    info!("Spawning THE LAST STAND - CNS Kairiola titan defense");

    // Spawn the titan at bottom center (fixed position)
    // Type 3764 is Leviathan (Caldari titan)
    let titan_type_id = 3764u32;
    let sprite = sprite_cache.get(titan_type_id);

    // Titan size: ~18km long, frigates ~100m
    // So titan is ~180x frigate size, but we compress for gameplay
    // Frigates render at ~50px, titan at ~820px (~16x) for visual impact
    let titan_size = 820.0;

    let mut titan = commands.spawn((
        LastStandTitan,
        Transform::from_xyz(0.0, -350.0, 5.0), // No rotation needed - sprite faces up toward enemies
    ));

    // Sprite is 107x692 (aspect ratio ~0.15), scale proportionally
    let titan_width = titan_size * 0.15;

    if let Some(texture) = sprite {
        titan.insert(Sprite {
            image: texture,
            custom_size: Some(Vec2::new(titan_width, titan_size)),
            ..default()
        });
    } else {
        // Fallback to colored rectangle
        titan.insert(Sprite {
            color: Color::srgb(0.3, 0.5, 0.8), // Caldari blue
            custom_size: Some(Vec2::new(titan_width, titan_size)),
            ..default()
        });
    }

    // Spawn the HUD
    commands
        .spawn((
            LastStandHud,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Top bar - Evacuation progress
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(20.0),
                        left: Val::Px(20.0),
                        right: Val::Px(20.0),
                        height: Val::Px(40.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.05, 0.1, 0.8)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Text::new("EVACUATION: 0%"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.8, 1.0)),
                        LastStandHudElement::Evacuation,
                    ));
                });

            // Left panel - Ship status
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(80.0),
                        left: Val::Px(20.0),
                        width: Val::Px(200.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.05, 0.1, 0.8)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("CNS KAIRIOLA"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.8, 1.0)),
                    ));
                    panel.spawn((
                        Text::new("SHIELD: 100%"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.6, 1.0)),
                        LastStandHudElement::Shield,
                    ));
                    panel.spawn((
                        Text::new("ARMOR: 100%"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.6, 0.2)),
                        LastStandHudElement::Armor,
                    ));
                    panel.spawn((
                        Text::new("HULL: 100%"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.3, 0.3)),
                        LastStandHudElement::Hull,
                    ));
                });

            // Right panel - Heat and abilities
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(80.0),
                        right: Val::Px(20.0),
                        width: Val::Px(220.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.05, 0.1, 0.8)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("HEAT: 0%"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.8, 0.5)),
                        LastStandHudElement::Heat,
                    ));
                    panel.spawn((
                        Text::new("FIGHTERS: 6"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        LastStandHudElement::Fighters,
                    ));
                    // Ability hints
                    panel.spawn((
                        Text::new("[RT] Fighter Launch"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        LastStandHudElement::Ability(LastStandAction::FighterLaunch),
                    ));
                    panel.spawn((
                        Text::new("[LB] ECM Burst"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        LastStandHudElement::Ability(LastStandAction::EcmBurst),
                    ));
                    panel.spawn((
                        Text::new("[RB] Shield Booster"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        LastStandHudElement::Ability(LastStandAction::ShieldBooster),
                    ));
                    panel.spawn((
                        Text::new("[Y] DOOMSDAY"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.8, 0.2)),
                        LastStandHudElement::Ability(LastStandAction::Doomsday),
                    ));
                });

            // Center message area (for milestones)
            parent.spawn((
                Text::new("Hold the line. Evacuate the fleet."),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.8)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(150.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                LastStandHudElement::Message,
            ));
        });
}

/// Update Last Stand state
pub fn update_last_stand(
    time: Res<Time>,
    mut last_stand: ResMut<LastStandState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut dialogue_events: EventWriter<crate::systems::DialogueEvent>,
) {
    let dt = time.delta_secs();
    let event = last_stand.update(dt);

    match event {
        LastStandEvent::Milestone(_idx) => {
            if let Some(message) = last_stand.current_milestone_message() {
                // Show milestone dialogue
                dialogue_events.send(crate::systems::DialogueEvent {
                    trigger: crate::systems::DialogueTrigger::Custom(message.to_string()),
                    custom_text: None,
                    duration: 4.0,
                    priority: 2,
                });
                info!("EVACUATION MILESTONE: {}", message);
            }
        }
        LastStandEvent::EvacuationComplete => {
            // Show descent prompt
            dialogue_events.send(crate::systems::DialogueEvent {
                trigger: crate::systems::DialogueTrigger::Custom(
                    "Evacuation complete. Press [A] to confirm descent into Gallente Prime."
                        .to_string(),
                ),
                custom_text: None,
                duration: 10.0,
                priority: 3,
            });
            info!("EVACUATION COMPLETE - Awaiting descent confirmation");
        }
        LastStandEvent::DescentComplete => {
            // Victory! Show special victory screen
            info!("CNS Kairiola completes its final mission");
            next_state.set(GameState::Victory);
        }
        LastStandEvent::Destroyed => {
            // Death before evacuation complete
            info!(
                "CNS Kairiola destroyed - Evacuation failed at {}%",
                last_stand.evacuation_progress as u32
            );
            next_state.set(GameState::GameOver);
        }
        LastStandEvent::None => {}
    }
}

/// Handle Last Stand ability inputs
pub fn last_stand_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut last_stand: ResMut<LastStandState>,
    mut commands: Commands,
) {
    // Fighter Launch (RT / F)
    if (keyboard.just_pressed(KeyCode::KeyF) || joystick.right_trigger_pressed())
        && last_stand.perform(LastStandAction::FighterLaunch)
    {
        info!(
            "Launching fighter squadron! {} remaining",
            last_stand.fighters_remaining
        );
        // Spawn 3 fighters in a formation
        for i in 0..3 {
            let offset_x = (i as f32 - 1.0) * 40.0; // -40, 0, +40
            commands.spawn((
                TitanFighter {
                    lifetime: 10.0,
                    target: None,
                },
                Sprite {
                    color: Color::srgb(0.5, 0.7, 1.0), // Caldari blue
                    custom_size: Some(Vec2::new(16.0, 20.0)),
                    ..default()
                },
                Transform::from_xyz(offset_x, -200.0, LAYER_PLAYER_BULLETS),
            ));
        }
    }

    // ECM Burst (LB / E)
    if (keyboard.just_pressed(KeyCode::KeyE) || joystick.left_bumper())
        && last_stand.perform(LastStandAction::EcmBurst)
    {
        info!("ECM Burst activated!");
        // Spawn ECM burst entity
        commands.spawn((
            EcmBurst {
                radius: 50.0,
                speed: 400.0,
                lifetime: 1.0,
            },
            Sprite {
                color: Color::srgba(0.3, 0.6, 1.0, 0.5),
                custom_size: Some(Vec2::splat(100.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -250.0, 8.0),
        ));
    }

    // Shield Booster (RB / Q)
    if (keyboard.just_pressed(KeyCode::KeyQ) || joystick.right_bumper())
        && last_stand.perform(LastStandAction::ShieldBooster)
    {
        info!(
            "Emergency shield booster activated! Shield: {:.0}%",
            last_stand.shield
        );
    }

    // Doomsday (Y / R)
    if (keyboard.just_pressed(KeyCode::KeyR) || joystick.y_button())
        && last_stand.perform(LastStandAction::Doomsday)
    {
        info!("DOOMSDAY DEVICE ACTIVATED!");
        // Spawn doomsday beam
        commands.spawn((
            DoomsdayBeam {
                width: 80.0,
                damage_per_sec: 500.0,
                duration: 3.0,
            },
            Sprite {
                color: Color::srgb(1.0, 0.9, 0.5),
                custom_size: Some(Vec2::new(80.0, 800.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 200.0, 15.0),
        ));
    }

    // Confirm Descent (A / Space) - only in descent phase
    if (keyboard.just_pressed(KeyCode::Space) || joystick.confirm())
        && last_stand.in_descent
        && !last_stand.descent_confirmed
        && last_stand.perform(LastStandAction::ConfirmDescent)
    {
        info!("DESCENT CONFIRMED - CNS Kairiola begins final approach");
    }
}

/// Update Last Stand HUD elements
pub fn update_last_stand_hud(
    last_stand: Res<LastStandState>,
    mut hud_query: Query<(&mut Text, &mut TextColor, &LastStandHudElement)>,
) {
    for (mut text, mut color, element) in hud_query.iter_mut() {
        match element {
            LastStandHudElement::Heat => {
                let heat = last_stand.heat as u32;
                **text = format!("HEAT: {}%", heat);
                // Color based on heat level
                let heat_color = if heat > 80 {
                    Color::srgb(1.0, 0.2, 0.2) // Critical red
                } else if heat > 50 {
                    Color::srgb(1.0, 0.6, 0.2) // Warning orange
                } else {
                    Color::srgb(0.3, 0.8, 0.5) // Safe green
                };
                *color = TextColor(heat_color);
            }
            LastStandHudElement::Evacuation => {
                let evac = last_stand.evacuation_progress as u32;
                **text = format!("EVACUATION: {}%", evac);
            }
            LastStandHudElement::Shield => {
                **text = format!("SHIELD: {:.0}%", last_stand.shield);
            }
            LastStandHudElement::Armor => {
                **text = format!("ARMOR: {:.0}%", last_stand.armor);
            }
            LastStandHudElement::Hull => {
                **text = format!("HULL: {:.0}%", last_stand.hull);
                // Flash red if low
                if last_stand.hull < 25.0 {
                    *color = TextColor(Color::srgb(1.0, 0.2, 0.2));
                }
            }
            LastStandHudElement::Fighters => {
                **text = format!("FIGHTERS: {}", last_stand.fighters_remaining);
            }
            LastStandHudElement::Ability(action) => {
                // Dim abilities on cooldown or unavailable
                let available = last_stand.can_perform(*action);
                let alpha = if available { 1.0 } else { 0.4 };
                match action {
                    LastStandAction::FighterLaunch => {
                        if last_stand.fighter_cooldown > 0.0 {
                            **text = format!("[RT] Fighter ({:.1}s)", last_stand.fighter_cooldown);
                        } else if last_stand.fighters_remaining == 0 {
                            **text = "[RT] No Fighters".to_string();
                        } else {
                            **text = "[RT] Fighter Launch".to_string();
                        }
                    }
                    LastStandAction::EcmBurst => {
                        if last_stand.ecm_cooldown > 0.0 {
                            **text = format!("[LB] ECM ({:.1}s)", last_stand.ecm_cooldown);
                        } else {
                            **text = "[LB] ECM Burst".to_string();
                        }
                    }
                    LastStandAction::ShieldBooster => {
                        if last_stand.shield_cooldown > 0.0 {
                            **text = format!("[RB] Shield ({:.1}s)", last_stand.shield_cooldown);
                        } else {
                            **text = "[RB] Shield Booster".to_string();
                        }
                    }
                    LastStandAction::Doomsday => {
                        if !last_stand.doomsday_available {
                            **text = "[Y] DOOMSDAY (USED)".to_string();
                        } else {
                            **text = "[Y] DOOMSDAY".to_string();
                        }
                    }
                    _ => {}
                }
                *color = TextColor(Color::srgba(0.6, 0.6, 0.6, alpha));
            }
            LastStandHudElement::Message => {
                if last_stand.in_descent && !last_stand.descent_confirmed {
                    **text = "Press [A/SPACE] to confirm descent".to_string();
                    *color = TextColor(Color::srgb(1.0, 0.8, 0.2));
                } else if last_stand.descent_confirmed {
                    **text = "Final approach... The State will remember.".to_string();
                    *color = TextColor(Color::srgb(1.0, 0.4, 0.2));
                }
            }
        }
    }
}

/// Spawn enemies for Last Stand mode
pub fn spawn_last_stand_enemies(
    time: Res<Time>,
    last_stand: Res<LastStandState>,
    mut commands: Commands,
    enemy_query: Query<Entity, With<crate::entities::Enemy>>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    mut spawn_timer: Local<f32>,
) {
    if !last_stand.active || last_stand.in_descent {
        return;
    }

    // Spawn enemies periodically based on evacuation progress
    *spawn_timer -= time.delta_secs();
    if *spawn_timer > 0.0 {
        return;
    }

    // Spawn rate increases with evacuation progress
    let spawn_interval = 5.0 - (last_stand.evacuation_progress / 100.0) * 3.0;
    *spawn_timer = spawn_interval.max(1.5);

    // Don't spawn if too many enemies already
    let enemy_count = enemy_query.iter().count();
    if enemy_count >= 20 {
        return;
    }

    // Spawn 2-4 enemies
    let count = 2 + (last_stand.evacuation_progress / 50.0) as usize;

    use crate::entities::enemy::{spawn_enemy, EnemyBehavior};

    for _i in 0..count {
        let x = (fastrand::f32() - 0.5) * 600.0;
        let y = 350.0;
        let type_id = [608, 594, 593][fastrand::usize(0..3)]; // Gallente frigates
        let sprite = sprite_cache.get(type_id);

        spawn_enemy(
            &mut commands,
            type_id,
            Vec2::new(x, y),
            EnemyBehavior::Linear, // Simple downward movement
            sprite,
            None,
        );
    }
}

/// Despawn all Last Stand entities
pub fn despawn_last_stand(
    mut commands: Commands,
    mut last_stand: ResMut<LastStandState>,
    hud_query: Query<Entity, With<LastStandHud>>,
    titan_query: Query<Entity, With<LastStandTitan>>,
    ecm_query: Query<Entity, With<EcmBurst>>,
    beam_query: Query<Entity, With<DoomsdayBeam>>,
    fighter_query: Query<Entity, With<TitanFighter>>,
) {
    last_stand.end();

    for entity in hud_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in titan_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in ecm_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in beam_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in fighter_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Update titan fighters - movement, targeting, damage, lifetime
pub fn update_titan_fighters(
    time: Res<Time>,
    mut commands: Commands,
    mut fighter_query: Query<(Entity, &mut TitanFighter, &mut Transform)>,
    enemy_query: Query<
        (Entity, &Transform, &crate::entities::EnemyStats),
        (With<crate::entities::Enemy>, Without<TitanFighter>),
    >,
    mut last_stand: ResMut<LastStandState>,
    mut despawned_this_frame: Local<std::collections::HashSet<Entity>>,
) {
    let dt = time.delta_secs();
    const FIGHTER_SPEED: f32 = 350.0;
    const FIGHTER_HIT_RANGE: f32 = 30.0;
    const FIGHTER_ACQUIRE_RANGE: f32 = 400.0;

    // Clear the set at the start of each frame
    despawned_this_frame.clear();

    for (fighter_entity, mut fighter, mut transform) in fighter_query.iter_mut() {
        // Decrement lifetime
        fighter.lifetime -= dt;
        if fighter.lifetime <= 0.0 {
            commands.entity(fighter_entity).despawn();
            continue;
        }

        // Find or validate target
        let mut current_target_valid = false;
        let mut target_pos = None;

        if let Some(target_entity) = fighter.target {
            // Check if target still exists and wasn't despawned this frame
            if !despawned_this_frame.contains(&target_entity) {
                if let Ok((_, enemy_transform, _)) = enemy_query.get(target_entity) {
                    current_target_valid = true;
                    target_pos = Some(enemy_transform.translation.truncate());
                }
            }
        }

        // Acquire new target if needed
        if !current_target_valid {
            fighter.target = None;
            let fighter_pos = transform.translation.truncate();
            let mut closest_dist = FIGHTER_ACQUIRE_RANGE;

            for (enemy_entity, enemy_transform, _) in enemy_query.iter() {
                // Skip enemies already despawned this frame
                if despawned_this_frame.contains(&enemy_entity) {
                    continue;
                }
                let enemy_pos = enemy_transform.translation.truncate();
                let dist = fighter_pos.distance(enemy_pos);
                if dist < closest_dist {
                    closest_dist = dist;
                    fighter.target = Some(enemy_entity);
                    target_pos = Some(enemy_pos);
                }
            }
        }

        // Move toward target or fly upward if no target
        let fighter_pos = transform.translation.truncate();
        let direction = if let Some(target) = target_pos {
            (target - fighter_pos).normalize_or_zero()
        } else {
            Vec2::Y // Fly upward if no target
        };

        transform.translation.x += direction.x * FIGHTER_SPEED * dt;
        transform.translation.y += direction.y * FIGHTER_SPEED * dt;

        // Rotate to face movement direction
        if direction != Vec2::ZERO {
            let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;
            transform.rotation = Quat::from_rotation_z(angle);
        }

        // Check for collision with target enemy
        if let Some(target_entity) = fighter.target {
            // Skip if already despawned this frame by another fighter
            if despawned_this_frame.contains(&target_entity) {
                fighter.target = None;
            } else if let Ok((_, enemy_transform, _)) = enemy_query.get(target_entity) {
                let enemy_pos = enemy_transform.translation.truncate();
                if fighter_pos.distance(enemy_pos) < FIGHTER_HIT_RANGE {
                    // Mark as despawned before actually despawning
                    despawned_this_frame.insert(target_entity);
                    commands.entity(target_entity).despawn_recursive();
                    last_stand.kills += 1;
                    fighter.target = None;

                    // Fighter continues to next target (doesn't despawn on hit)
                }
            }
        }

        // Despawn if off-screen (too far up)
        if transform.translation.y > 400.0 {
            commands.entity(fighter_entity).despawn();
        }
    }
}
