//! CG Campaign Systems
//!
//! Boss encounters, mission flow, and boss intro UI for the campaign.

use super::campaign::{CGBossType, CGCampaignState, CG_INTER_WAVE_DELAY, ShiigeruNightmare};
use crate::core::{DamageType, Difficulty, Faction, GameSession, GameState};
use crate::entities::enemy::EnemyBehavior;
use crate::entities::projectile::ProjectilePhysics;
use bevy::prelude::*;

/// Component for CG boss entities
#[derive(Component)]
pub struct CGBoss {
    pub boss_type: CGBossType,
    pub health: f32,
    pub max_health: f32,
    pub current_phase: u32,
    pub total_phases: u32,
}

/// Component for CG boss movement
#[derive(Component)]
pub struct CGBossMovement {
    pub timer: f32,
    pub speed: f32,
}

/// Component for CG boss attacks
#[derive(Component)]
pub struct CGBossAttack {
    pub fire_timer: f32,
    pub fire_rate: f32,
}

/// Visual telegraph shown at spawn positions before a wave appears.
#[derive(Component)]
pub struct CGSpawnTelegraph {
    pub timer: f32,
}

// ============================================================================
// CG Boss Intro UI Components
// ============================================================================

/// Root marker for CG boss intro overlay
#[derive(Component)]
pub struct CGBossIntroRoot;

/// Warning text that pulses
#[derive(Component)]
pub struct CGBossIntroWarning {
    timer: f32,
}

/// Boss name that fades in
#[derive(Component)]
pub struct CGBossIntroName {
    timer: f32,
}

/// Boss dialogue that types in
#[derive(Component)]
pub struct CGBossIntroDialogue {
    full_text: String,
    timer: f32,
}

/// Start a CG mission when entering Playing state
pub fn start_cg_mission(mut cg_campaign: ResMut<CGCampaignState>) {
    cg_campaign.start_mission();

    if let Some(mission) = cg_campaign.current_mission() {
        info!(
            "Starting CG Mission {}: {} - {}",
            cg_campaign.mission_number(),
            mission.name,
            mission.description
        );
    }
}

/// Update CG mission timer
pub fn update_cg_mission(
    _time: Res<Time>,
    cg_campaign: Res<CGCampaignState>,
    nightmare: Res<ShiigeruNightmare>,
) {
    // Don't update if nightmare mode is active
    if nightmare.active {
        return;
    }

    if cg_campaign.in_mission {
        // Timer tracking could be added here if needed
    }
}

/// Check if current wave is complete in CG campaign.
/// When a wave is cleared, starts the inter-wave delay timer so the next
/// wave doesn't spawn instantly (pacing breather for the player).
pub fn check_cg_wave_complete(
    mut cg_campaign: ResMut<CGCampaignState>,
    enemy_query: Query<Entity, With<crate::entities::Enemy>>,
    boss_query: Query<Entity, With<CGBoss>>,
) {
    // Don't check if we're in boss wave
    if cg_campaign.is_boss_wave() {
        return;
    }

    // Don't check if boss exists
    if boss_query.iter().count() > 0 {
        return;
    }

    // Wave complete when no enemies remain
    let enemy_count = enemy_query.iter().count();
    if enemy_count == 0 && cg_campaign.current_wave > 0 && cg_campaign.in_mission {
        if let Some(mission) = cg_campaign.current_mission() {
            if cg_campaign.current_wave <= mission.waves {
                info!("CG Wave {} complete!", cg_campaign.current_wave);
                // Start inter-wave delay if not already counting
                if cg_campaign.wave_delay_timer <= 0.0 {
                    cg_campaign.wave_delay_timer = CG_INTER_WAVE_DELAY;
                }
            }
        }
    }
}

/// Spawn next wave of enemies for CG campaign.
/// Respects `wave_delay_timer` for pacing between waves.
pub fn spawn_cg_wave(
    mut commands: Commands,
    time: Res<Time>,
    mut cg_campaign: ResMut<CGCampaignState>,
    session: Res<GameSession>,
    difficulty: Res<crate::core::Difficulty>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    enemy_query: Query<Entity, With<crate::entities::Enemy>>,
    boss_query: Query<Entity, With<CGBoss>>,
    telegraph_query: Query<Entity, With<CGSpawnTelegraph>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    use crate::entities::enemy::spawn_enemy;

    // Tick inter-wave delay
    if cg_campaign.wave_delay_timer > 0.0 {
        // On first frame of delay, spawn telegraphs at the upcoming wave positions
        if cg_campaign.wave_delay_timer >= CG_INTER_WAVE_DELAY - 0.05 {
            let wave = cg_campaign.current_wave;
            let base_count = 3 + wave as usize;
            let spawn_mult = difficulty.spawn_rate_mult();
            let count = (base_count as f32 * spawn_mult) as usize;
            spawn_cg_telegraphs(
                &mut commands,
                count,
                wave,
                session.enemy_faction,
            );
        }
        cg_campaign.wave_delay_timer -= time.delta_secs();
        return;
    }

    // Only spawn if no enemies remain
    if enemy_query.iter().count() > 0 || boss_query.iter().count() > 0 {
        return;
    }

    // Despawn any lingering telegraphs before spawning the actual wave
    despawn_cg_telegraphs(&mut commands, &telegraph_query);

    let Some(mission) = cg_campaign.current_mission() else {
        return;
    };

    // Check if it's boss time
    if cg_campaign.current_wave > mission.waves {
        if !cg_campaign.boss_spawned && mission.boss.is_some() {
            // Transition to boss intro
            next_state.set(GameState::BossIntro);
        } else if mission.boss.is_none() {
            // No boss mission - complete immediately
            next_state.set(GameState::StageComplete);
        }
        return;
    }

    // Spawn wave enemies
    let wave = cg_campaign.current_wave;
    let base_count = 3 + wave as usize;
    let spawn_mult = difficulty.spawn_rate_mult();
    let count = (base_count as f32 * spawn_mult) as usize;

    info!("CG: Spawning wave {} with {} enemies", wave, count);

    // Get enemy type IDs based on enemy faction
    let enemy_types: Vec<u32> = match session.enemy_faction {
        Faction::Caldari => vec![583, 602, 603], // Condor, Kestrel, Merlin
        Faction::Gallente => vec![608, 594, 593], // Atron, Incursus, Tristan
        Faction::Amarr => vec![597, 589, 591],   // Punisher, Executioner, Tormentor
        Faction::Minmatar => vec![587, 585, 598], // Rifter, Slasher, Breacher
    };

    for i in 0..count {
        let type_id = enemy_types[fastrand::usize(..enemy_types.len())];
        let sprite = sprite_cache.get(type_id);

        // Vary spawn positions by wave for visual interest and readability
        let (x, y) = cg_spawn_position(i, count, wave);

        let behavior = cg_behavior_for_mission(cg_campaign.mission_index);

        let entity = spawn_enemy(
            &mut commands,
            type_id,
            Vec2::new(x, y),
            behavior,
            sprite,
            None,
        );

        // Apply mission-specific scaling for vertical slice tuning
        apply_cg_mission_scaling(&mut commands, entity, type_id, cg_campaign.mission_index);
    }

    cg_campaign.current_wave += 1;
}

/// Compute a varied spawn position for CG wave enemies.
/// Wave 1: single line; Wave 2: V-formation; Wave 3+: staggered columns.
fn cg_spawn_position(i: usize, count: usize, wave: u32) -> (f32, f32) {
    let spread = (count as f32 * 70.0).min(500.0);
    let base_x = -spread / 2.0;

    match wave {
        1 => {
            // Line formation: flat across top
            let x = base_x + (i as f32 / count.max(1) as f32) * spread;
            let y = 320.0 + fastrand::f32() * 40.0;
            (x, y)
        }
        2 => {
            // V-formation: edges forward, center back
            let t = i as f32 / count.max(1) as f32;
            let x = base_x + t * spread;
            let y_offset = (t - 0.5).abs() * 120.0;
            let y = 300.0 + y_offset + fastrand::f32() * 20.0;
            (x, y)
        }
        _ => {
            // Staggered columns
            let col = i % 3;
            let row = i / 3;
            let x = base_x + (col as f32 / 3.0) * spread + fastrand::f32() * 30.0 - 15.0;
            let y = 280.0 + row as f32 * 50.0 + fastrand::f32() * 30.0;
            (x, y)
        }
    }
}

/// Spawn visual telegraphs (warning indicators) at the positions where the
/// next wave will appear. Gives the player ~2.5 s to read the pattern and
/// reposition before enemies actually spawn.
fn spawn_cg_telegraphs(
    commands: &mut Commands,
    count: usize,
    wave: u32,
    enemy_faction: Faction,
) {
    let color = match enemy_faction {
        Faction::Caldari => Color::srgba(0.4, 0.6, 0.9, 0.35),   // Caldari blue
        Faction::Gallente => Color::srgba(0.4, 0.9, 0.5, 0.35), // Gallente green
        Faction::Amarr => Color::srgba(0.9, 0.6, 0.2, 0.35),    // Amarr gold
        Faction::Minmatar => Color::srgba(0.9, 0.3, 0.2, 0.35),  // Minmatar red
    };

    for i in 0..count {
        let (x, y) = cg_spawn_position(i, count, wave);
        commands.spawn((
            CGSpawnTelegraph { timer: 0.0 },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 5.0),
        ));
    }
}

fn despawn_cg_telegraphs(commands: &mut Commands, query: &Query<Entity, With<CGSpawnTelegraph>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Animate telegraphs: fast pulse in opacity so the player notices them.
pub fn update_cg_telegraphs(
    time: Res<Time>,
    mut query: Query<(&mut Sprite,
        &mut CGSpawnTelegraph,
    )>,
) {
    let dt = time.delta_secs();
    for (mut sprite, mut telegraph) in query.iter_mut() {
        telegraph.timer += dt * 8.0; // fast pulse
        let pulse = (telegraph.timer.sin() * 0.5 + 0.5).clamp(0.2, 1.0);
        sprite.color.set_alpha(pulse * 0.5);
    }
}

/// Pick an enemy behavior weighted by mission index.
/// Mission 1 (tutorial): mostly Linear (easy to hit, predictable).
/// Mission 2: more Zigzag and Homing (requires leading shots).
/// Mission 3+: emphasis on Weaver and Homing (harassment, spread pressure).
fn cg_behavior_for_mission(mission_index: usize) -> EnemyBehavior {
    match mission_index {
        0 => {
            // Tutorial: 70% linear, 30% zigzag — easy patterns
            match fastrand::u32(0..10) {
                0..=6 => EnemyBehavior::Linear,
                _ => EnemyBehavior::Zigzag,
            }
        }
        1 => {
            // Mission 2: 40% zigzag, 30% homing, 20% linear, 10% weaver
            match fastrand::u32(0..10) {
                0..=3 => EnemyBehavior::Zigzag,
                4..=6 => EnemyBehavior::Homing,
                7..=8 => EnemyBehavior::Linear,
                _ => EnemyBehavior::Weaver,
            }
        }
        _ => {
            // Mission 3+: 35% weaver, 30% homing, 25% zigzag, 10% linear
            match fastrand::u32(0..20) {
                0..=6 => EnemyBehavior::Weaver,
                7..=12 => EnemyBehavior::Homing,
                13..=17 => EnemyBehavior::Zigzag,
                _ => EnemyBehavior::Linear,
            }
        }
    }
}

/// Overwrite enemy stats for Caldari/Gallente mission scaling.
/// Call immediately after spawn_enemy() in spawn_cg_wave.
///
/// Mission 1 (tutorial): 2x HP, 60% damage — forgiving first contact.
/// Mission 2: 1.5x HP, 80% damage — gentle ramp.
/// Mission 3+: baseline stats.
fn apply_cg_mission_scaling(
    commands: &mut Commands,
    entity: Entity,
    type_id: u32,
    mission_index: usize,
) {
    let hp_mult = match mission_index {
        0 => 2.0,
        1 => 1.5,
        _ => 1.0,
    };
    let dmg_mult = match mission_index {
        0 => 0.6,
        1 => 0.8,
        _ => 1.0,
    };

    let (name, base_hp, speed, score) = match type_id {
        // NOTE: These base HP values MUST stay in sync with spawn.rs.
        // When buffing base stats, update BOTH tables.
        583 => ("Condor", 35.0, 130.0, 75),
        602 => ("Kestrel", 40.0, 100.0, 90),
        603 => ("Merlin", 55.0, 70.0, 100),
        593 => ("Tristan", 40.0, 90.0, 100),
        594 => ("Incursus", 50.0, 85.0, 95),
        608 => ("Atron", 35.0, 130.0, 75),
        _ => return,
    };

    let scaled_hp = base_hp * hp_mult;
    commands.entity(entity).insert(crate::entities::EnemyStats {
        type_id,
        name: name.to_string(),
        health: scaled_hp,
        max_health: scaled_hp,
        speed,
        score_value: score,
        is_boss: false,
        liberation_value: 1,
    });

    let (weapon_type, fire_rate, base_dmg, bullet_speed) = match type_id {
        603 => (crate::core::WeaponType::Railgun, 0.6, 18.0, 350.0),
        602 | 583 => (crate::core::WeaponType::MissileLauncher, 0.5, 20.0, 180.0),
        593 | 594 | 608 => (crate::core::WeaponType::Drone, 1.2, 8.0, 200.0),
        _ => return,
    };

    let scaled_dmg = base_dmg * dmg_mult;
    commands.entity(entity).insert(crate::entities::EnemyWeapon {
        weapon_type,
        fire_rate,
        damage: scaled_dmg,
        bullet_speed,
        cooldown: 0.5 + fastrand::f32() * 1.0,
        pattern: crate::entities::FiringPattern::Single,
    });
}

/// Spawn CG boss for current mission
pub fn spawn_cg_boss(
    mut commands: Commands,
    mut cg_campaign: ResMut<CGCampaignState>,
    session: Res<GameSession>,
    difficulty: Res<Difficulty>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
) {
    let Some(mission) = cg_campaign.current_mission() else {
        return;
    };

    let Some(boss_type) = mission.boss else {
        return;
    };

    info!(
        "Spawning CG Boss: {} (difficulty: {:?})",
        boss_type.name(),
        *difficulty
    );

    // Scale health by difficulty
    let base_health = boss_type.health();
    let health = base_health * difficulty.enemy_health_mult();
    let phases = boss_type.phases();

    // Scale fire rate by difficulty (lower = faster attacks)
    let fire_rate = 1.2 / difficulty.enemy_fire_rate_mult();

    // Get boss type_id based on enemy faction
    let type_id = boss_type.type_id(session.enemy_faction);
    let size = 100.0; // Boss size

    // Get sprite from cache or fallback to colored square
    let sprite = if type_id > 0 {
        if let Some(texture) = sprite_cache.get(type_id) {
            Sprite {
                image: texture,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        } else {
            // Fallback color based on enemy faction
            let boss_color = match session.enemy_faction {
                Faction::Caldari => Color::srgb(0.4, 0.6, 0.9),
                Faction::Gallente => Color::srgb(0.4, 0.9, 0.5),
                _ => Color::srgb(1.0, 0.5, 0.5),
            };
            Sprite {
                color: boss_color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        }
    } else {
        let boss_color = match session.enemy_faction {
            Faction::Caldari => Color::srgb(0.4, 0.6, 0.9),
            Faction::Gallente => Color::srgb(0.4, 0.9, 0.5),
            _ => Color::srgb(1.0, 0.5, 0.5),
        };
        Sprite {
            color: boss_color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        }
    };

    // Spawn the boss entity with Enemy + EnemyStats for collision system compatibility
    commands.spawn((
        crate::entities::Enemy,
        crate::entities::EnemyStats {
            type_id,
            name: boss_type.name().to_string(),
            health,
            max_health: health,
            speed: 80.0,
            score_value: (health as u64) * 10,
            is_boss: true,
            liberation_value: 50,
        },
        CGBoss {
            boss_type,
            health,
            max_health: health,
            current_phase: 1,
            total_phases: phases,
        },
        CGBossMovement {
            timer: 0.0,
            speed: 80.0,
        },
        CGBossAttack {
            fire_timer: 0.0,
            fire_rate, // Scaled by difficulty
        },
        sprite,
        // Rotate 180° to face down (ships face up by default)
        Transform::from_xyz(0.0, 400.0, 10.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::PI)),
    ));

    cg_campaign.boss_spawned = true;
}

/// CG Boss intro sequence
pub fn cg_boss_intro(
    time: Res<Time>,
    mut boss_query: Query<(&mut Transform, &CGBoss)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();

    for (mut transform, boss) in boss_query.iter_mut() {
        // Descend boss to battle position
        let target_y = 200.0;
        if transform.translation.y > target_y {
            transform.translation.y -= 100.0 * time.delta_secs();
        }

        // After 2 seconds, start fight
        if *timer > 2.0 {
            *timer = 0.0;
            next_state.set(GameState::BossFight);
            info!("CG Boss battle started: {}", boss.boss_type.name());
        }
    }
}

/// Spawn CG boss intro UI overlay
pub fn spawn_cg_boss_intro(mut commands: Commands, cg_campaign: Res<CGCampaignState>) {
    let Some(mission) = cg_campaign.current_mission() else {
        return;
    };

    let Some(boss_type) = mission.boss else {
        return;
    };

    let boss_name = boss_type.name();
    let boss_title = boss_type.title();
    let dialogue = boss_type.dialogue_intro();
    let phases = boss_type.phases();

    // Phase difficulty indicator
    let phase_text = match phases {
        1 => "Single Phase",
        2 => "Two Phases",
        3 => "Three Phases • Challenging",
        4 => "Four Phases • Dangerous",
        _ => "Multi-Phase",
    };

    commands
        .spawn((
            CGBossIntroRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        ))
        .with_children(|parent| {
            // Warning text (pulses)
            parent.spawn((
                Text::new("⚠ WARNING ⚠"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                CGBossIntroWarning { timer: 0.0 },
            ));

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Boss name (fades in) - Caldari blue instead of Amarr gold
            parent.spawn((
                Text::new(boss_name),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgba(0.2, 0.6, 1.0, 0.0)), // Start transparent, Caldari blue
                CGBossIntroName { timer: 0.0 },
            ));

            // Boss title
            parent.spawn((
                Text::new(boss_title),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.7, 0.9)), // Lighter blue
            ));

            // Phase indicator
            parent.spawn((
                Text::new(phase_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if phases >= 4 {
                    Color::srgb(1.0, 0.4, 0.4) // Red for dangerous
                } else if phases >= 3 {
                    Color::srgb(1.0, 0.7, 0.3) // Orange for challenging
                } else {
                    Color::srgb(0.6, 0.6, 0.6) // Gray for normal
                }),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Boss dialogue (types in)
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                CGBossIntroDialogue {
                    full_text: format!("\"{}\"", dialogue),
                    timer: 0.0,
                },
            ));
        });
}

/// Update CG boss intro animations
pub fn cg_boss_intro_update(
    time: Res<Time>,
    mut warning_query: Query<(&mut TextColor, &mut CGBossIntroWarning)>,
    mut name_query: Query<(&mut TextColor, &mut CGBossIntroName), Without<CGBossIntroWarning>>,
    mut dialogue_query: Query<(&mut Text, &mut CGBossIntroDialogue)>,
) {
    let dt = time.delta_secs();

    // Pulse warning text
    for (mut color, mut warning) in warning_query.iter_mut() {
        warning.timer += dt * 4.0;
        let pulse = (warning.timer.sin() * 0.3 + 0.7).clamp(0.4, 1.0);
        *color = TextColor(Color::srgb(1.0, 0.2 * pulse, 0.2 * pulse));
    }

    // Fade in boss name (Caldari blue)
    for (mut color, mut name) in name_query.iter_mut() {
        name.timer += dt * 2.0;
        let alpha = (name.timer - 0.3).clamp(0.0, 1.0); // Delay 0.3s then fade in
        *color = TextColor(Color::srgba(0.2, 0.6, 1.0, alpha));
    }

    // Type in dialogue
    for (mut text, mut dialogue) in dialogue_query.iter_mut() {
        dialogue.timer += dt;
        let chars_to_show = ((dialogue.timer - 0.5) * 30.0) as usize; // 30 chars/sec, 0.5s delay
        let chars_to_show = chars_to_show.min(dialogue.full_text.len());
        if chars_to_show > 0 {
            **text = dialogue.full_text[..chars_to_show].to_string();
        }
    }
}

/// Despawn CG boss intro UI
pub fn despawn_cg_boss_intro(mut commands: Commands, query: Query<Entity, With<CGBossIntroRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Update CG boss behavior during fight
pub fn update_cg_boss(
    time: Res<Time>,
    mut boss_query: Query<(
        &mut Transform,
        &mut CGBoss,
        &mut CGBossMovement,
        &mut CGBossAttack,
        &crate::entities::EnemyStats,
    )>,
    player_query: Query<&Transform, (With<crate::entities::Player>, Without<CGBoss>)>,
    mut commands: Commands,
    difficulty: Res<Difficulty>,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, mut boss, mut movement, mut attack, enemy_stats) in boss_query.iter_mut() {
        let pos = transform.translation.truncate();
        let dt = time.delta_secs();

        // Sync health from EnemyStats (collision system updates this)
        boss.health = enemy_stats.health;

        // Movement - sweep pattern
        movement.timer += dt;
        let offset = (movement.timer * 0.5).sin() * 200.0;
        transform.translation.x = offset;

        // Phase transitions
        let health_percent = boss.health / boss.max_health;
        let phase_threshold = 1.0 - (boss.current_phase as f32 / boss.total_phases as f32);

        if health_percent <= phase_threshold && boss.current_phase < boss.total_phases {
            boss.current_phase += 1;
            movement.speed *= 1.2;
            attack.fire_rate *= 0.8;
            info!("CG Boss entering phase {}!", boss.current_phase);
        }

        // Attack
        attack.fire_timer += dt;
        if attack.fire_timer >= attack.fire_rate {
            attack.fire_timer = 0.0;

            let dir = (player_pos - pos).normalize_or_zero();
            let projectile_speed = 250.0 + (boss.current_phase as f32 * 50.0);

            // Scale damage by difficulty
            let base_damage = 20.0 + (boss.current_phase as f32 * 5.0);
            let scaled_damage = base_damage * difficulty.enemy_damage_mult();

            commands.spawn((
                crate::entities::EnemyProjectile,
                crate::entities::ProjectileDamage {
                    damage: scaled_damage,
                    damage_type: DamageType::EM,
                    crit_chance: 0.08,
                    crit_multiplier: 1.5,
                    ammo_type: crate::core::AmmoType::default(),
                },
                ProjectilePhysics {
                    velocity: dir * projectile_speed,
                    lifetime: 4.0,
                },
                Sprite {
                    color: Color::srgb(1.0, 0.8, 0.2),
                    custom_size: Some(Vec2::new(8.0, 16.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y - 30.0, 9.0),
            ));
        }
    }
}

/// Check if CG boss is defeated
pub fn check_cg_boss_defeated(
    mut commands: Commands,
    mut cg_campaign: ResMut<CGCampaignState>,
    boss_query: Query<(Entity, &CGBoss, &crate::entities::EnemyStats)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, boss, enemy_stats) in boss_query.iter() {
        // Check EnemyStats health (collision system updates this)
        if enemy_stats.health <= 0.0 {
            info!("CG Boss defeated: {}", boss.boss_type.name());

            // Mark boss defeated
            cg_campaign.boss_defeated = true;

            // Despawn boss
            commands.entity(entity).despawn_recursive();

            // Go to stage complete (mission advancement happens when player confirms)
            next_state.set(GameState::StageComplete);
        }
    }
}
