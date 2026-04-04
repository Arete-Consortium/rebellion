//! Nightmare Mode Systems
//!
//! Endless wave survival with mini-boss encounters.

use super::campaign::{NightmareBoss, ShiigeruNightmare, NightmareEvent};
use super::faction_select::COLOR_CALDARI_ACCENT;
use crate::core::{Faction, GameSession};
use bevy::prelude::*;

/// Component to mark nightmare HUD elements
#[derive(Component)]
pub struct NightmareHud;

/// Component to mark nightmare mini-boss
#[derive(Component)]
pub struct NightmareMiniBoss {
    pub boss_type: NightmareBoss,
}

/// Root marker for wave announcement overlay
#[derive(Component)]
pub struct NightmareWaveAnnouncement {
    timer: f32,
    max_time: f32,
}

/// Root marker for mini-boss intro overlay
#[derive(Component)]
pub struct NightmareMiniBossIntro {
    timer: f32,
    max_time: f32,
    boss_type: NightmareBoss,
    spawned: bool,
}

/// Pulse animation for warning text
#[derive(Component)]
pub struct NightmareWarningPulse {
    timer: f32,
}

/// Typewriter effect for dialogue
#[derive(Component)]
pub struct NightmareDialogue {
    full_text: String,
    timer: f32,
}

/// Marker for spawn requests
#[derive(Component)]
pub enum NightmareSpawnRequest {
    Wave,
    Boss(NightmareBoss),
}

/// HUD element types for nightmare mode
#[derive(Component)]
pub enum NightmareHudElement {
    Wave,
    Time,
    Kills,
    Hull,
}

/// Spawn wave announcement overlay
fn spawn_wave_announcement(commands: &mut Commands, wave: u32) {
    // Only show announcement every 5th wave or wave 1
    if wave != 1 && !wave.is_multiple_of(5) {
        return;
    }

    commands
        .spawn((
            NightmareWaveAnnouncement {
                timer: 0.0,
                max_time: 1.5,
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("WAVE {}", wave)),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                NightmareWarningPulse { timer: 0.0 },
            ));

            if wave >= 20 {
                parent.spawn((
                    Text::new("EXTREME DANGER"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                ));
            } else if wave >= 10 {
                parent.spawn((
                    Text::new("DANGER INCREASING"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.6, 0.3)),
                ));
            }
        });
}

/// Spawn mini-boss intro overlay
fn spawn_miniboss_intro(commands: &mut Commands, boss: NightmareBoss) {
    commands
        .spawn((
            NightmareMiniBossIntro {
                timer: 0.0,
                max_time: 2.5,
                boss_type: boss,
                spawned: false,
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // Warning
            parent.spawn((
                Text::new("⚠ MINI-BOSS ⚠"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                NightmareWarningPulse { timer: 0.0 },
            ));

            parent.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });

            // Boss name
            parent.spawn((
                Text::new(boss.name()),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.4, 0.2)), // Orange-red
            ));

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Dialogue (typewriter)
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                NightmareDialogue {
                    full_text: format!("\"{}\"", boss.dialogue()),
                    timer: 0.0,
                },
            ));
        });
}

/// Update wave announcements (fade out and despawn)
pub fn update_wave_announcements(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut NightmareWaveAnnouncement, &mut BackgroundColor)>,
    mut text_query: Query<(&mut TextColor, &mut NightmareWarningPulse)>,
) {
    let dt = time.delta_secs();

    for (entity, mut announcement, mut bg) in query.iter_mut() {
        announcement.timer += dt;

        // Fade in/out background
        let progress = announcement.timer / announcement.max_time;
        let alpha = if progress < 0.2 {
            progress / 0.2 * 0.3
        } else if progress > 0.7 {
            (1.0 - progress) / 0.3 * 0.3
        } else {
            0.3
        };
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));

        if announcement.timer >= announcement.max_time {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Pulse warning text
    for (mut color, mut pulse) in text_query.iter_mut() {
        pulse.timer += dt * 6.0;
        let intensity = (pulse.timer.sin() * 0.3 + 0.7).clamp(0.4, 1.0);
        *color = TextColor(Color::srgb(1.0, 0.2 * intensity, 0.2 * intensity));
    }
}

/// Update mini-boss intros (typewriter, spawn boss, despawn)
pub fn update_miniboss_intros(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut NightmareMiniBossIntro)>,
    mut dialogue_query: Query<(&mut Text, &mut NightmareDialogue)>,
) {
    let dt = time.delta_secs();

    for (entity, mut intro) in query.iter_mut() {
        intro.timer += dt;

        // Spawn boss after 1.5 seconds
        if intro.timer >= 1.5 && !intro.spawned {
            intro.spawned = true;
            commands.spawn(NightmareSpawnRequest::Boss(intro.boss_type));
        }

        // Despawn overlay after max time
        if intro.timer >= intro.max_time {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Typewriter effect for dialogue
    for (mut text, mut dialogue) in dialogue_query.iter_mut() {
        dialogue.timer += dt;
        let chars_to_show = ((dialogue.timer - 0.3) * 35.0) as usize; // 35 chars/sec
        let chars_to_show = chars_to_show.min(dialogue.full_text.len());
        if chars_to_show > 0 {
            **text = dialogue.full_text[..chars_to_show].to_string();
        }
    }
}

/// Update nightmare state timers and spawn events
pub fn update_nightmare_mode(
    time: Res<Time>,
    mut nightmare: ResMut<ShiigeruNightmare>,
    mut commands: Commands,
) {
    let event = nightmare.update(time.delta_secs());

    match event {
        NightmareEvent::SpawnWave(wave) => {
            info!(
                "NIGHTMARE Wave {} - {} enemies incoming!",
                wave,
                nightmare.enemies_per_wave()
            );
            // Spawn wave announcement overlay (shows every 5th wave and wave 1)
            spawn_wave_announcement(&mut commands, wave);
            // Spawn the wave immediately
            commands.spawn(NightmareSpawnRequest::Wave);
        }
        NightmareEvent::SpawnBoss(boss) => {
            info!("NIGHTMARE BOSS: {} - \"{}\"", boss.name(), boss.dialogue());
            // Spawn mini-boss intro overlay (will spawn boss after delay)
            spawn_miniboss_intro(&mut commands, boss);
            // Note: Boss spawn request is now created by update_miniboss_intros after delay
        }
        NightmareEvent::None => {}
    }
}

/// Spawn enemies based on nightmare mode state
pub fn spawn_nightmare_enemies(
    mut commands: Commands,
    nightmare: Res<ShiigeruNightmare>,
    session: Res<GameSession>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    spawn_requests: Query<(Entity, &NightmareSpawnRequest)>,
) {
    use crate::entities::enemy::{spawn_enemy, EnemyBehavior};

    // Get enemy type IDs based on faction
    let enemy_types: Vec<u32> = match session.enemy_faction {
        Faction::Caldari => vec![583, 602, 603], // Condor, Kestrel, Merlin
        Faction::Gallente => vec![608, 594, 593], // Atron, Incursus, Tristan
        Faction::Amarr => vec![597, 589, 591],   // Punisher, Executioner, Tormentor
        Faction::Minmatar => vec![587, 585, 598], // Rifter, Slasher, Breacher
    };

    for (entity, request) in spawn_requests.iter() {
        // Despawn the request marker
        commands.entity(entity).despawn();

        match request {
            NightmareSpawnRequest::Wave => {
                // Spawn wave enemies
                let count = nightmare.enemies_per_wave();

                for i in 0..count {
                    // Spread spawn positions across top of screen
                    let x = -300.0 + (i as f32 * 600.0 / count.max(1) as f32);
                    let y = 300.0 + fastrand::f32() * 50.0;

                    // Random enemy type and behavior
                    let type_id = enemy_types[fastrand::usize(..enemy_types.len())];
                    let sprite = sprite_cache.get(type_id);
                    let behavior = match fastrand::u32(0..4) {
                        0 => EnemyBehavior::Linear,
                        1 => EnemyBehavior::Zigzag,
                        2 => EnemyBehavior::Homing,
                        _ => EnemyBehavior::Weaver,
                    };

                    spawn_enemy(
                        &mut commands,
                        type_id,
                        Vec2::new(x, y),
                        behavior,
                        sprite,
                        None,
                    );
                }
            }
            NightmareSpawnRequest::Boss(boss_type) => {
                // Spawn mini-boss at top center
                let type_id = enemy_types[0]; // Use first type as "elite"
                let sprite = sprite_cache.get(type_id);

                spawn_enemy(
                    &mut commands,
                    type_id,
                    Vec2::new(0.0, 320.0),
                    EnemyBehavior::Homing, // Bosses track player
                    sprite,
                    None,
                );

                info!("Mini-boss {} spawned!", boss_type.name());
            }
        }
    }
}

/// Update nightmare HUD elements
pub fn update_nightmare_hud(
    nightmare: Res<ShiigeruNightmare>,
    mut hud_query: Query<(&mut Text, &NightmareHudElement)>,
) {
    for (mut text, element) in hud_query.iter_mut() {
        match element {
            NightmareHudElement::Wave => {
                **text = format!("WAVE {}", nightmare.wave);
            }
            NightmareHudElement::Time => {
                let mins = (nightmare.time_survived / 60.0) as u32;
                let secs = (nightmare.time_survived % 60.0) as u32;
                **text = format!("{:02}:{:02}", mins, secs);
            }
            NightmareHudElement::Kills => {
                **text = format!("KILLS: {}", nightmare.kills);
            }
            NightmareHudElement::Hull => {
                **text = format!("HULL: {:.0}%", nightmare.hull_integrity);
            }
        }
    }
}

/// Spawn the nightmare mode HUD
pub fn spawn_nightmare_hud(mut commands: Commands) {
    info!("Spawning nightmare mode HUD");

    // HUD container at top-left
    commands
        .spawn((
            NightmareHud,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("⚠ SHIIGERU NIGHTMARE ⚠"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));

            // Wave counter
            parent.spawn((
                NightmareHudElement::Wave,
                Text::new("WAVE 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Time survived
            parent.spawn((
                NightmareHudElement::Time,
                Text::new("00:00"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(COLOR_CALDARI_ACCENT),
            ));

            // Kills
            parent.spawn((
                NightmareHudElement::Kills,
                Text::new("KILLS: 0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Hull integrity
            parent.spawn((
                NightmareHudElement::Hull,
                Text::new("HULL: 100%"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.4, 0.4)),
            ));
        });
}
