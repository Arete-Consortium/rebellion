//! Music System
//!
//! Procedural ambient music for Rebellion.
//! Generates atmospheric tracks for menu, gameplay, and boss fights.

#![allow(dead_code)]

use bevy::audio::{AudioSink, PlaybackMode, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::f32::consts::PI;

use crate::core::*;

/// Music plugin
pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicAssets>()
            .init_resource::<MusicState>();

        app.add_systems(Startup, (generate_music, load_music_file_overrides).chain())
            .add_systems(
                Update,
                (
                    manage_menu_music.run_if(in_state(GameState::MainMenu)),
                    manage_gameplay_music
                        .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
                    update_music_intensity
                        .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
                    handle_state_music_transitions,
                ),
            );
    }
}

/// Generated music assets
#[derive(Resource, Default)]
pub struct MusicAssets {
    // Procedural fallbacks — always populated at startup.
    pub menu_ambient: Option<Handle<AudioSource>>,
    pub gameplay_ambient: Option<Handle<AudioSource>>,
    pub boss_ambient: Option<Handle<AudioSource>>,
    pub victory_sting: Option<Handle<AudioSource>>,
    pub defeat_sting: Option<Handle<AudioSource>>,
    // Optional file-based overrides loaded from `assets/audio/music/`.
    // When the asset's load state is Loaded, the play system prefers
    // these over the procedural fallback. When the file is missing or
    // failed to load, procedural plays.
    pub menu_file: Option<Handle<AudioSource>>,
    pub gameplay_file: Option<Handle<AudioSource>>,
    pub boss_file: Option<Handle<AudioSource>>,
    pub victory_file: Option<Handle<AudioSource>>,
    pub defeat_file: Option<Handle<AudioSource>>,
}

impl MusicAssets {
    /// Pick the best handle for a given slot — file override if it has
    /// finished loading, else procedural fallback.
    pub fn effective(
        &self,
        slot: MusicType,
        asset_server: &AssetServer,
    ) -> Option<Handle<AudioSource>> {
        let (file, proc) = match slot {
            MusicType::Menu => (&self.menu_file, &self.menu_ambient),
            MusicType::Gameplay => (&self.gameplay_file, &self.gameplay_ambient),
            MusicType::Boss => (&self.boss_file, &self.boss_ambient),
            MusicType::None => return None,
        };
        if let Some(h) = file {
            if matches!(
                asset_server.get_load_state(h.id()),
                Some(bevy::asset::LoadState::Loaded)
            ) {
                return Some(h.clone());
            }
        }
        proc.clone()
    }
}

/// Current music state
#[derive(Resource)]
pub struct MusicState {
    pub current_track: Option<Entity>,
    pub current_type: MusicType,
    /// Current reactive volume scale (0.25 = idle, 1.0 = max intensity)
    pub volume: f32,
    pub fade_timer: f32,
    pub fading_out: bool,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            current_track: None,
            current_type: MusicType::None,
            volume: 0.25, // Start at idle baseline
            fade_timer: 0.0,
            fading_out: false,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MusicType {
    #[default]
    None,
    Menu,
    Gameplay,
    Boss,
}

/// Marker for music entities
#[derive(Component)]
pub struct MusicTrack {
    pub music_type: MusicType,
}

/// Generate all music tracks at startup
fn generate_music(mut music: ResMut<MusicAssets>, mut audio_sources: ResMut<Assets<AudioSource>>) {
    info!("Generating procedural music...");

    // Menu ambient - slow, mysterious, spacey
    if let Some(source) = generate_menu_ambient() {
        music.menu_ambient = Some(audio_sources.add(source));
    }

    // Gameplay ambient - tense, driving
    if let Some(source) = generate_gameplay_ambient() {
        music.gameplay_ambient = Some(audio_sources.add(source));
    }

    // Boss ambient - intense, urgent
    if let Some(source) = generate_boss_ambient() {
        music.boss_ambient = Some(audio_sources.add(source));
    }

    // Victory sting
    if let Some(source) = generate_victory_sting() {
        music.victory_sting = Some(audio_sources.add(source));
    }

    // Defeat sting
    if let Some(source) = generate_defeat_sting() {
        music.defeat_sting = Some(audio_sources.add(source));
    }

    info!("Music generation complete!");
}

/// Try to load file-based music overrides from `assets/audio/music/`.
/// Loading is async — handles start in the Loading state and either
/// resolve to Loaded (file present + valid) or Failed (404 or bad
/// format). The `effective()` helper on MusicAssets picks file vs.
/// procedural at play time based on load state, so missing files are
/// silent fallback to procedural (no error to the player).
///
/// Drop tracks at:
///   assets/audio/music/menu.ogg     (main menu)
///   assets/audio/music/gameplay.ogg (during stages)
///   assets/audio/music/boss.ogg     (boss fights)
///   assets/audio/music/victory.ogg  (stage complete sting)
///   assets/audio/music/defeat.ogg   (game over sting)
///
/// .ogg is preferred for size; .wav and .mp3 also work via Bevy's
/// default audio loaders.
fn load_music_file_overrides(mut music: ResMut<MusicAssets>, asset_server: Res<AssetServer>) {
    music.menu_file = Some(asset_server.load("audio/music/menu.ogg"));
    music.gameplay_file = Some(asset_server.load("audio/music/gameplay.ogg"));
    music.boss_file = Some(asset_server.load("audio/music/boss.ogg"));
    music.victory_file = Some(asset_server.load("audio/music/victory.ogg"));
    music.defeat_file = Some(asset_server.load("audio/music/defeat.ogg"));
    info!("Queued music file overrides — files at assets/audio/music/* will replace procedural tracks if present");
}

// =============================================================================
// MUSIC GENERATORS
// =============================================================================

/// Generate menu ambient - ethereal, space atmosphere (30 seconds loop)
fn generate_menu_ambient() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 30.0; // 30 second loop
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Deep drone pad (shifting between notes)
        let drone_freq = 55.0 + 10.0 * (0.1 * t).sin(); // A1 with slow drift
        let drone = (2.0 * PI * drone_freq * t).sin() * 0.15;

        // Fifth above (creates space feeling)
        let fifth_freq = drone_freq * 1.5;
        let fifth = (2.0 * PI * fifth_freq * t).sin() * 0.08;

        // Slow LFO modulated pad
        let lfo = (2.0 * PI * 0.05 * t).sin(); // Very slow modulation
        let pad_freq = 110.0 + lfo * 5.0;
        let pad =
            (2.0 * PI * pad_freq * t).sin() * 0.06 * (0.5 + 0.5 * (2.0 * PI * 0.03 * t).sin());

        // Ethereal shimmer (high frequencies)
        let shimmer_freq = 880.0 + 220.0 * (0.07 * t).sin();
        let shimmer =
            (2.0 * PI * shimmer_freq * t).sin() * 0.02 * (0.5 + 0.5 * (2.0 * PI * 0.02 * t).sin());

        // Occasional distant "star" twinkles
        let twinkle = if (t * 0.3).fract() < 0.01 {
            let tw_freq = 1200.0 + 400.0 * ((t * 7.0).sin());
            (2.0 * PI * tw_freq * t).sin() * 0.03 * (-(t * 0.3).fract() * 100.0).exp()
        } else {
            0.0
        };

        // Mix with subtle fade in/out for seamless loop
        let loop_env = if t < 2.0 {
            t / 2.0
        } else if t > duration - 2.0 {
            (duration - t) / 2.0
        } else {
            1.0
        };

        let sample = ((drone + fifth + pad + shimmer + twinkle) * loop_env * 0.8).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate gameplay ambient - tense, driving (20 seconds loop)
fn generate_gameplay_ambient() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 20.0;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let bpm = 120.0;
    let beat_duration = 60.0 / bpm;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let beat = (t / beat_duration).floor();
        let beat_phase = (t / beat_duration).fract();

        // Driving bass pulse (on the beat)
        let bass_freq = if beat as i32 % 4 == 0 { 55.0 } else { 41.25 }; // A1 / E1
        let bass_env = (-beat_phase * 8.0).exp();
        let bass = (2.0 * PI * bass_freq * t).sin() * bass_env * 0.2;

        // Sub-bass rumble
        let sub = (2.0 * PI * 30.0 * t).sin() * 0.1;

        // Pulsing synth (offbeat)
        let synth_freq = 110.0;
        let synth_env = if beat_phase > 0.5 {
            (-(beat_phase - 0.5) * 10.0).exp()
        } else {
            0.0
        };
        let synth = (2.0 * PI * synth_freq * t).sin() * synth_env * 0.08;

        // High tension string-like pad
        let tension_freq = 220.0 * (1.0 + 0.01 * (t * 0.5).sin()); // Slight detune for tension
        let tension = (2.0 * PI * tension_freq * t).sin() * 0.05;

        // Rhythmic hi-hat-like noise
        let hihat_phase = (t * 4.0 / beat_duration).fract();
        let hihat = if hihat_phase < 0.1 {
            (fastrand::f32() * 2.0 - 1.0) * (-hihat_phase * 50.0).exp() * 0.03
        } else {
            0.0
        };

        // Loop envelope
        let loop_env = if t < 1.0 {
            t
        } else if t > duration - 1.0 {
            duration - t
        } else {
            1.0
        };

        let sample = ((bass + sub + synth + tension + hihat) * loop_env * 0.9).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate boss ambient - intense, urgent (15 seconds loop)
fn generate_boss_ambient() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 15.0;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let bpm = 140.0; // Faster tempo
    let beat_duration = 60.0 / bpm;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let beat = (t / beat_duration).floor();
        let beat_phase = (t / beat_duration).fract();

        // Heavy bass hit every beat
        let bass_freq = 36.7; // D1 - ominous
        let bass_env = (-beat_phase * 12.0).exp();
        let bass = (2.0 * PI * bass_freq * t).sin() * bass_env * 0.25;

        // Distorted bass overtones
        let dist = (2.0 * PI * bass_freq * 2.0 * t).sin() * bass_env * 0.1;

        // Urgent alarm-like synth (tritone for tension)
        let alarm_freq = if beat as i32 % 2 == 0 { 293.66 } else { 415.3 }; // D4 / Ab4 tritone
        let alarm_env = (-beat_phase * 6.0).exp();
        let alarm = (2.0 * PI * alarm_freq * t).sin() * alarm_env * 0.06;

        // Rapid hi-hats
        let hh_phase = (t * 8.0 / beat_duration).fract();
        let hihat = (fastrand::f32() * 2.0 - 1.0) * (-hh_phase * 40.0).exp() * 0.04;

        // Tension riser (pitch goes up over time, resets at loop)
        let riser_freq = 200.0 + (t / duration) * 400.0;
        let riser = (2.0 * PI * riser_freq * t).sin() * 0.03;

        // Loop envelope
        let loop_env = if t < 0.5 {
            t * 2.0
        } else if t > duration - 0.5 {
            (duration - t) * 2.0
        } else {
            1.0
        };

        let sample = ((bass + dist + alarm + hihat + riser) * loop_env).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate victory sting - triumphant, short
fn generate_victory_sting() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 3.0;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Rising arpeggio: C - E - G - C (major chord)
        let note = if t < 0.3 {
            261.63 // C4
        } else if t < 0.6 {
            329.63 // E4
        } else if t < 0.9 {
            392.0 // G4
        } else {
            523.25 // C5 (hold through end)
        };

        let note_t = t % 0.3;
        let note_env = if t < 1.2 {
            (1.0 - note_t / 0.3).powf(0.3)
        } else {
            (-(t - 1.2) * 1.5).exp()
        };

        let melody = (2.0 * PI * note * t).sin() * note_env * 0.3;

        // Harmony pad
        let pad = (2.0 * PI * 130.81 * t).sin() * 0.1 // C3
            + (2.0 * PI * 164.81 * t).sin() * 0.08 // E3
            + (2.0 * PI * 196.0 * t).sin() * 0.08; // G3

        let pad_env = (-(t - 1.0).max(0.0) * 0.5).exp();

        // Shimmer
        let shimmer = (2.0 * PI * 1046.5 * t).sin() * 0.02 * (-(t - 0.5).max(0.0) * 2.0).exp();

        let sample = ((melody + pad * pad_env + shimmer) * 0.8).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate defeat sting - somber, short
fn generate_defeat_sting() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 2.5;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Descending minor: E - D - C - B (sad descent)
        let note = if t < 0.4 {
            329.63 // E4
        } else if t < 0.8 {
            293.66 // D4
        } else if t < 1.2 {
            261.63 // C4
        } else {
            246.94 // B3
        };

        let note_env = (-(t % 0.4) * 3.0).exp() * (-t * 0.8).exp();
        let melody = (2.0 * PI * note * t).sin() * note_env * 0.25;

        // Minor pad (Am)
        let pad = (2.0 * PI * 110.0 * t).sin() * 0.1 // A2
            + (2.0 * PI * 130.81 * t).sin() * 0.08 // C3
            + (2.0 * PI * 164.81 * t).sin() * 0.08; // E3

        let pad_env = (-t * 0.5).exp();

        // Low rumble
        let rumble = (2.0 * PI * 55.0 * t).sin() * 0.08 * (-t * 0.3).exp();

        let sample = ((melody + pad * pad_env + rumble) * 0.8).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

// =============================================================================
// MUSIC MANAGEMENT
// =============================================================================

/// Manage menu music
fn manage_menu_music(
    mut commands: Commands,
    music_assets: Res<MusicAssets>,
    asset_server: Res<AssetServer>,
    mut music_state: ResMut<MusicState>,
    settings: Res<crate::systems::audio::SoundSettings>,
) {
    // Only start music if not already playing menu music
    if music_state.current_type != MusicType::Menu {
        // Despawn old track
        if let Some(entity) = music_state.current_track {
            commands.entity(entity).despawn();
        }

        // Spawn menu music — file override if loaded, else procedural.
        if let Some(source) = music_assets.effective(MusicType::Menu, &asset_server) {
            if settings.enabled {
                let entity = commands
                    .spawn((
                        MusicTrack {
                            music_type: MusicType::Menu,
                        },
                        AudioPlayer(source),
                        PlaybackSettings {
                            mode: PlaybackMode::Loop,
                            volume: Volume::new(
                                settings.music_volume * settings.master_volume * 0.4,
                            ),
                            ..default()
                        },
                    ))
                    .id();

                music_state.current_track = Some(entity);
                music_state.current_type = MusicType::Menu;
            }
        }
    }
}

/// Manage gameplay music
fn manage_gameplay_music(
    mut commands: Commands,
    music_assets: Res<MusicAssets>,
    asset_server: Res<AssetServer>,
    mut music_state: ResMut<MusicState>,
    settings: Res<crate::systems::audio::SoundSettings>,
    boss_query: Query<&crate::entities::Boss>,
) {
    let has_boss = !boss_query.is_empty();
    let target_type = if has_boss {
        MusicType::Boss
    } else {
        MusicType::Gameplay
    };

    // Switch music if needed
    if music_state.current_type != target_type {
        // Despawn old track
        if let Some(entity) = music_state.current_track {
            commands.entity(entity).despawn();
        }

        // File override if loaded, else procedural fallback.
        let source = music_assets.effective(target_type, &asset_server);

        if let Some(source) = source {
            if settings.enabled {
                let entity = commands
                    .spawn((
                        MusicTrack {
                            music_type: target_type,
                        },
                        AudioPlayer(source),
                        PlaybackSettings {
                            mode: PlaybackMode::Loop,
                            volume: Volume::new(
                                settings.music_volume * settings.master_volume * 0.35,
                            ),
                            ..default()
                        },
                    ))
                    .id();

                music_state.current_track = Some(entity);
                music_state.current_type = target_type;
            }
        }
    }
}

/// Handle music transitions on state changes
fn handle_state_music_transitions(
    mut commands: Commands,
    music_assets: Res<MusicAssets>,
    mut music_state: ResMut<MusicState>,
    settings: Res<crate::systems::audio::SoundSettings>,
    game_state: Res<State<GameState>>,
) {
    // Play victory sting on victory
    if *game_state.get() == GameState::Victory && music_state.current_type != MusicType::None {
        // Stop current music
        if let Some(entity) = music_state.current_track {
            commands.entity(entity).despawn();
        }
        music_state.current_track = None;
        music_state.current_type = MusicType::None;

        // Play victory sting
        if let Some(source) = music_assets.victory_sting.clone() {
            if settings.enabled {
                commands.spawn((
                    AudioPlayer(source),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new(settings.music_volume * settings.master_volume * 0.5),
                        ..default()
                    },
                ));
            }
        }
    }

    // Play defeat sting on game over
    if *game_state.get() == GameState::GameOver && music_state.current_type != MusicType::None {
        // Stop current music
        if let Some(entity) = music_state.current_track {
            commands.entity(entity).despawn();
        }
        music_state.current_track = None;
        music_state.current_type = MusicType::None;

        // Play defeat sting
        if let Some(source) = music_assets.defeat_sting.clone() {
            if settings.enabled {
                commands.spawn((
                    AudioPlayer(source),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new(settings.music_volume * settings.master_volume * 0.5),
                        ..default()
                    },
                ));
            }
        }
    }
}

// =============================================================================
// MUSIC REACTIVITY
// =============================================================================

/// Adjust music volume based on gameplay intensity (combo multiplier).
///
/// Maps the ScoreSystem multiplier to a volume range:
/// - Multiplier 1.0 (idle): base volume (0.25)
/// - Multiplier 5.0+: full volume (1.0)
///
/// Volume changes are smoothed over time to avoid jarring transitions.
fn update_music_intensity(
    time: Res<Time>,
    score: Res<ScoreSystem>,
    mut music_state: ResMut<MusicState>,
    settings: Res<crate::systems::audio::SoundSettings>,
    music_query: Query<&AudioSink, With<MusicTrack>>,
) {
    // Map combo multiplier to a 0.0..1.0 intensity range
    // multiplier 1.0 -> 0.0, multiplier 5.0+ -> 1.0
    let intensity = ((score.multiplier - 1.0) / 4.0).clamp(0.0, 1.0);

    // Lerp between quiet ambient (0.25) and full presence (1.0)
    let target_scale = 0.25 + 0.75 * intensity;

    // Smooth toward target — ramp up faster than decay for punch
    let lerp_speed = if target_scale > music_state.volume {
        4.0 // quick ramp up on kills
    } else {
        1.5 // slower fade back to ambient
    };
    let dt = time.delta_secs();
    music_state.volume += (target_scale - music_state.volume) * (lerp_speed * dt).min(1.0);

    // Apply to the active music track via AudioSink
    let base_volume = settings.music_volume * settings.master_volume * 0.35;
    let final_volume = base_volume * music_state.volume;

    for sink in &music_query {
        sink.set_volume(final_volume);
    }
}

// =============================================================================
// AUDIO UTILS
// =============================================================================

/// Create AudioSource from samples — works on both native and WASM
fn create_audio_source(samples: &[f32], sample_rate: u32) -> Option<AudioSource> {
    super::wav_encoder::create_audio_source(samples, sample_rate)
}
