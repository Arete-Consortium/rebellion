//! Procedural sound generation functions
//!
//! All `generate_*` functions that synthesize audio waveforms.

use bevy::prelude::*;
use std::f32::consts::PI;

/// Create AudioSource from f32 samples — works on both native and WASM
pub fn create_audio_source(samples: &[f32], sample_rate: u32) -> Option<AudioSource> {
    crate::systems::wav_encoder::create_audio_source(samples, sample_rate)
}

// =============================================================================
// WEAPON SOUND GENERATORS
// =============================================================================

/// Original layered synthesis: mechanical transients, capacitor resonance and
/// filtered engine noise. These cues do not contain sampled game recordings.
#[derive(Clone, Copy)]
enum IndustrialCue {
    Autocannon,
    Laser,
    Missile,
    Railgun,
    Drone,
    CarrierWarp,
}

fn industrial_cue(cue: IndustrialCue) -> Option<AudioSource> {
    let rate = 44_100u32;
    let duration = match cue {
        IndustrialCue::Autocannon => 0.22,
        IndustrialCue::Laser => 0.34,
        IndustrialCue::Missile => 0.5,
        IndustrialCue::Railgun => 0.28,
        IndustrialCue::Drone => 0.18,
        IndustrialCue::CarrierWarp => 2.2,
    };
    let mut rng = fastrand::Rng::with_seed(0x524542454c + cue as u64);
    let mut low_noise = 0.0;
    let num_samples = (rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f32 / rate as f32;
        let noise = rng.f32() * 2.0 - 1.0;
        low_noise += 0.06 * (noise - low_noise);
        let tone = |frequency: f32| (2.0 * PI * frequency * t).sin();
        let chirp = |start: f32, slope: f32| (2.0 * PI * (start * t + 0.5 * slope * t * t)).sin();
        let body = match cue {
            IndustrialCue::Autocannon => {
                0.52 * chirp(150.0, -330.0) * (-18.0 * t).exp()
                    + 0.25 * (tone(1260.0) + 0.4 * tone(2810.0)) * (-65.0 * t).exp()
                    + 0.45 * noise * (-90.0 * t).exp()
            }
            IndustrialCue::Laser => {
                (0.36 * chirp(780.0, -1150.0) + 0.14 * tone(1565.0) * (tone(37.0) * 0.35 + 0.65))
                    * (-11.0 * t).exp()
                    + 0.12 * noise * (-95.0 * t).exp()
            }
            IndustrialCue::Missile => {
                (1.5 * low_noise + 0.23 * chirp(75.0, 110.0))
                    * (1.0 - (-90.0 * t).exp())
                    * (-6.0 * t).exp()
                    + 0.32 * noise * (-95.0 * t).exp()
            }
            IndustrialCue::Railgun => {
                0.48 * noise * (-110.0 * t).exp()
                    + 0.4 * chirp(260.0, -420.0) * (-22.0 * t).exp()
                    + (0.13 * tone(2300.0) + 0.07 * tone(3317.0)) * (-28.0 * t).exp()
            }
            IndustrialCue::Drone => {
                (0.33 * tone(390.0) + 0.16 * tone(803.0) * (0.5 + 0.5 * tone(56.0)))
                    * (-28.0 * t).exp()
            }
            IndustrialCue::CarrierWarp => {
                let charge = (t / 1.9).clamp(0.0, 1.0);
                let collapse = (-(t - 1.95).max(0.0) * 18.0).exp();
                (0.15 * tone(48.0) + 0.22 * chirp(95.0, 430.0) + low_noise * 0.7)
                    * charge
                    * collapse
                    + if t >= 1.95 {
                        0.42 * (2.0 * PI * 62.0 * (t - 1.95)).sin() * (-(t - 1.95) * 13.0).exp()
                    } else {
                        0.0
                    }
            }
        };
        // Short edge ramps eliminate clicks, with headroom for simultaneous voices.
        let edge = (t / 0.002).min(1.0) * ((duration - t) / 0.012).clamp(0.0, 1.0);
        samples.push((body * edge * 0.85).tanh() * 0.85);
    }
    create_audio_source(&samples, rate)
}

pub fn generate_autocannon() -> Option<AudioSource> {
    industrial_cue(IndustrialCue::Autocannon)
}
pub fn generate_laser() -> Option<AudioSource> {
    industrial_cue(IndustrialCue::Laser)
}
pub fn generate_missile() -> Option<AudioSource> {
    industrial_cue(IndustrialCue::Missile)
}
pub fn generate_railgun() -> Option<AudioSource> {
    industrial_cue(IndustrialCue::Railgun)
}
pub fn generate_drone() -> Option<AudioSource> {
    industrial_cue(IndustrialCue::Drone)
}
pub fn generate_carrier_warp() -> Option<AudioSource> {
    industrial_cue(IndustrialCue::CarrierWarp)
}

// =============================================================================
// EXPLOSION / IMPACT SOUND GENERATORS
// =============================================================================

/// Generate explosion sound
pub fn generate_explosion(duration: f32, base_freq: f32) -> Option<AudioSource> {
    let rate = 44_100u32;
    let mut rng = fastrand::Rng::with_seed(base_freq.to_bits() as u64);
    let mut low = 0.0;
    let samples: Vec<f32> = (0..(rate as f32 * duration) as usize)
        .map(|i| {
            let t = i as f32 / rate as f32;
            let noise = rng.f32() * 2.0 - 1.0;
            low += 0.045 * (noise - low);
            let decay = (-6.0 * t / duration).exp();
            let rumble = (2.0 * PI * (base_freq * t - 0.2 * base_freq * t * t / duration)).sin();
            let debris = ((2.0 * PI * 713.0 * t).sin() + (2.0 * PI * 1193.0 * t).sin())
                * 0.06
                * (-22.0 * t).exp();
            let edge = (t / 0.003).min(1.0) * ((duration - t) / 0.015).clamp(0.0, 1.0);
            ((0.5 * rumble * decay + 1.6 * low * decay + 0.38 * noise * (-75.0 * t).exp() + debris)
                * edge)
                .tanh()
                * 0.82
        })
        .collect();
    create_audio_source(&samples, rate)
}

/// Generate pickup sound - happy blip
pub fn generate_pickup() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.1;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Rising frequency
        let freq = 400.0 + t * 2000.0;
        let wave = (2.0 * PI * freq * t).sin();

        // Envelope
        let env = (1.0 - t / duration) * (1.0 - (-t * 50.0).exp());

        let sample = (wave * env * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Layered impact identities: shield resonance, armor ring and structural crunch.
fn generate_impact(layer: u32) -> Option<AudioSource> {
    let rate = 44_100u32;
    let duration = [0.18, 0.24, 0.32][layer as usize];
    let mut random = fastrand::Rng::with_seed(0x494d50414354 + layer as u64);
    let mut low = 0.0;
    let samples: Vec<f32> = (0..(rate as f32 * duration) as usize)
        .map(|i| {
            let t = i as f32 / rate as f32;
            let noise = random.f32() * 2.0 - 1.0;
            low += 0.09 * (noise - low);
            let tone = |hz: f32| (2.0 * PI * hz * t).sin();
            let body = match layer {
                0 => {
                    (0.28 * tone(970.0) + 0.12 * tone(1943.0)) * (-27.0 * t).exp()
                        + 0.22 * (noise - low) * (-80.0 * t).exp()
                }
                1 => {
                    (0.3 * tone(270.0) + 0.14 * tone(731.0) + 0.08 * tone(1267.0))
                        * (-22.0 * t).exp()
                        + 0.35 * noise * (-110.0 * t).exp()
                }
                _ => {
                    (0.48 * tone(82.0) + 0.2 * tone(173.0) + low) * (-17.0 * t).exp()
                        + 0.28 * noise * (-85.0 * t).exp()
                }
            };
            let edge = (t / 0.002).min(1.0) * ((duration - t) / 0.012).clamp(0.0, 1.0);
            (body * edge).tanh() * 0.8
        })
        .collect();
    create_audio_source(&samples, rate)
}

pub fn generate_shield_hit() -> Option<AudioSource> {
    generate_impact(0)
}
pub fn generate_armor_hit() -> Option<AudioSource> {
    generate_impact(1)
}
pub fn generate_hull_hit() -> Option<AudioSource> {
    generate_impact(2)
}

/// Distinct pulse cadence and a rougher resonance as damage becomes critical.
fn generate_alarm(level: usize) -> Option<AudioSource> {
    let rate = 44_100u32;
    let (duration, frequency, pulse, gap, count) = match level {
        0 => (0.6, 930.0, 0.11, 0.07, 3),
        1 => (0.7, 640.0, 0.19, 0.10, 2),
        _ => (0.9, 430.0, 0.19, 0.07, 3),
    };
    let mut phase = 0.0;
    let samples: Vec<f32> = (0..(rate as f32 * duration) as usize)
        .map(|i| {
            let t = i as f32 / rate as f32;
            let cycle = pulse + gap;
            let local = t % cycle;
            let warble = if level == 2 {
                32.0 * (2.0 * PI * 7.0 * t).sin()
            } else {
                0.0
            };
            phase += 2.0 * PI * (frequency + warble) / rate as f32;
            if (t / cycle) as usize >= count || local >= pulse {
                return 0.0;
            }
            let edge = (local / 0.004).min(1.0) * ((pulse - local) / 0.018).clamp(0.0, 1.0);
            (phase.sin() * 0.38 + (phase * 1.49).sin() * 0.12) * edge
        })
        .collect();
    create_audio_source(&samples, rate)
}

pub fn generate_shield_warning() -> Option<AudioSource> {
    generate_alarm(0)
}
pub fn generate_armor_warning() -> Option<AudioSource> {
    generate_alarm(1)
}
pub fn generate_hull_warning() -> Option<AudioSource> {
    generate_alarm(2)
}

// =============================================================================
// GAME EVENT SOUND GENERATORS
// =============================================================================

/// Generate wave complete sound - triumphant ascending chime
pub fn generate_wave_complete() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.5;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Three ascending notes
        let note1 = if t < 0.15 {
            (2.0 * PI * 523.25 * t).sin() * (1.0 - t / 0.15).powf(0.5) // C5
        } else {
            0.0
        };

        let note2 = if (0.12..0.3).contains(&t) {
            let nt = t - 0.12;
            (2.0 * PI * 659.25 * t).sin() * (1.0 - nt / 0.18).powf(0.5) // E5
        } else {
            0.0
        };

        let note3 = if t >= 0.25 {
            let nt = t - 0.25;
            (2.0 * PI * 783.99 * t).sin() * (-nt * 6.0).exp() // G5
        } else {
            0.0
        };

        let sample = ((note1 + note2 + note3) * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate boss spawn sound - dramatic low impact
pub fn generate_boss_spawn() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.8;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Deep impact
        let bass = (2.0 * PI * 60.0 * t).sin() * 0.6;

        // Ominous drone
        let drone = (2.0 * PI * 100.0 * t).sin() * 0.3;
        let drone2 = (2.0 * PI * 150.0 * t).sin() * 0.2;

        // Metallic ring
        let ring = (2.0 * PI * 300.0 * t).sin() * (-t * 4.0).exp() * 0.3;

        // Rumble
        let rumble = (fastrand::f32() * 2.0 - 1.0) * 0.2 * (-t * 3.0).exp();

        let env = (1.0 - (-t * 10.0).exp()) * (-t * 2.5).exp();

        let sample = ((bass + drone + drone2 + ring + rumble) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

// =============================================================================
// POWERUP SOUND GENERATORS
// =============================================================================

/// Generate overdrive powerup sound - engine rev
pub fn generate_powerup_overdrive() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.3;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Rising engine freq
        let freq = 200.0 + t * 600.0;
        let engine = (2.0 * PI * freq * t).sin() * 0.5;

        // Turbo whoosh
        let whoosh = (fastrand::f32() * 2.0 - 1.0) * 0.3 * (t * 4.0).min(1.0);

        let env = (1.0 - (-t * 20.0).exp()) * (1.0 - (t / duration).powf(2.0));

        let sample = ((engine + whoosh) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate damage boost powerup sound - power surge
pub fn generate_powerup_damage() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.25;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Power charge
        let charge = (2.0 * PI * (400.0 + t * 800.0) * t).sin() * 0.5;

        // Electric crackle
        let crackle = if fastrand::f32() < 0.15 {
            (fastrand::f32() * 2.0 - 1.0) * 0.4
        } else {
            0.0
        };

        let env = (1.0 - (-t * 30.0).exp()) * (-t * 6.0).exp();

        let sample = ((charge + crackle) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate invulnerability powerup sound - shield activation
pub fn generate_powerup_invuln() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.35;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Shield hum
        let hum = (2.0 * PI * 300.0 * t).sin() * 0.3;

        // Shimmer
        let shimmer = (2.0 * PI * 1200.0 * t).sin() * 0.2 * (t * 8.0).sin().abs();

        // Bass impact
        let bass = (2.0 * PI * 80.0 * t).sin() * 0.4 * (-t * 15.0).exp();

        let env = (1.0 - (-t * 20.0).exp()) * (1.0 - (t / duration).powf(3.0));

        let sample = ((hum + shimmer + bass) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate health restore powerup sound - healing chime
pub fn generate_powerup_health() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.2;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Gentle ascending tone
        let freq = 600.0 + t * 400.0;
        let tone = (2.0 * PI * freq * t).sin() * 0.4;

        // Soft shimmer
        let shimmer = (2.0 * PI * freq * 2.0 * t).sin() * 0.15;

        let env = (1.0 - (-t * 30.0).exp()) * (-t * 8.0).exp();

        let sample = ((tone + shimmer) * env * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

// =============================================================================
// MENU SOUND GENERATORS
// =============================================================================

/// Generate menu navigation sound - soft blip
pub fn generate_menu_select() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.05;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        let wave = (2.0 * PI * 800.0 * t).sin();
        let env = (-t * 60.0).exp();

        let sample = (wave * env * 0.4).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate menu confirm sound - satisfying click
pub fn generate_menu_confirm() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.1;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        let wave1 = (2.0 * PI * 600.0 * t).sin() * 0.4;
        let wave2 = (2.0 * PI * 900.0 * t).sin() * 0.3;

        let env = (-t * 30.0).exp();

        let sample = ((wave1 + wave2) * env * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

// =============================================================================
// ABILITY SOUND GENERATORS
// =============================================================================

/// Generate speed ability sound - engine boost whoosh
pub fn generate_ability_speed() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.4;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Rising engine frequency
        let freq = 150.0 + t * 800.0;
        let engine = (2.0 * PI * freq * t).sin() * 0.4;

        // Turbo whoosh (filtered noise)
        let whoosh = (fastrand::f32() * 2.0 - 1.0) * 0.35 * (t * 5.0).min(1.0);

        // High overtone
        let high = (2.0 * PI * (freq * 2.5) * t).sin() * 0.15 * (t * 8.0).min(1.0);

        let env = (1.0 - (-t * 15.0).exp()) * (1.0 - (t / duration).powf(1.5));

        let sample = ((engine + whoosh + high) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate shield ability sound - energy bubble activation
pub fn generate_ability_shield() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.35;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Shield activation sweep
        let freq = 800.0 - t * 400.0;
        let sweep = (2.0 * PI * freq * t).sin() * 0.4;

        // Shimmer
        let shimmer = (2.0 * PI * 2400.0 * t).sin() * 0.2 * (1.0 + (PI * 20.0 * t).sin() * 0.5);

        // Bubble pop at start
        let pop = (2.0 * PI * 300.0 * t).sin() * (-t * 60.0).exp() * 0.3;

        let env = (1.0 - (-t * 25.0).exp()) * (-t * 4.0).exp();

        let sample = ((sweep + shimmer + pop) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate armor ability sound - metallic clang/hardening
pub fn generate_ability_armor() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.3;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Metallic clang
        let clang = (2.0 * PI * 400.0 * t).sin() * 0.3 * (-t * 20.0).exp();

        // Harmonic overtones (metallic)
        let harm1 = (2.0 * PI * 800.0 * t).sin() * 0.2 * (-t * 25.0).exp();
        let harm2 = (2.0 * PI * 1200.0 * t).sin() * 0.15 * (-t * 30.0).exp();

        // Low rumble for weight
        let rumble = (2.0 * PI * 80.0 * t).sin() * 0.25 * (-t * 10.0).exp();

        let sample = ((clang + harm1 + harm2 + rumble) * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate weapon ability sound - charging burst
pub fn generate_ability_weapon() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.25;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Rapid charge up
        let freq = 200.0 + t * 1200.0;
        let charge = (2.0 * PI * freq * t).sin() * 0.4;

        // Burst
        let burst = if t > 0.15 {
            (2.0 * PI * 500.0 * t).sin() * 0.5 * (-(t - 0.15) * 40.0).exp()
        } else {
            0.0
        };

        // Crackle
        let crackle = if fastrand::f32() < 0.1 {
            (fastrand::f32() * 2.0 - 1.0) * 0.3
        } else {
            0.0
        };

        let env = 1.0 - (-t * 40.0).exp();

        let sample = ((charge + burst + crackle) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate drone ability sound - mechanical launch
pub fn generate_ability_drone() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.4;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Mechanical hum
        let hum = (2.0 * PI * 120.0 * t).sin() * 0.3;

        // Drone whine (rising)
        let freq = 400.0 + t * 300.0;
        let whine = (2.0 * PI * freq * t).sin() * 0.25;

        // Launch click
        let click = (2.0 * PI * 1000.0 * t).sin() * (-t * 100.0).exp() * 0.4;

        // Propeller flutter
        let flutter = (2.0 * PI * 60.0 * t).sin() * 0.15 * (t * 4.0).min(1.0);

        let env = (1.0 - (-t * 20.0).exp()) * (1.0 - (t / duration).powf(2.0));

        let sample = ((hum + whine + click + flutter) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate debuff ability sound - disrupting pulse
pub fn generate_ability_debuff() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.35;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Warping frequency
        let warp = (2.0 * PI * (300.0 + 200.0 * (PI * 15.0 * t).sin()) * t).sin() * 0.4;

        // Disruptor pulse
        let pulse = (2.0 * PI * 100.0 * t).sin() * 0.3 * (1.0 + (PI * 8.0 * t).sin() * 0.5);

        // Static
        let static_noise = (fastrand::f32() * 2.0 - 1.0) * 0.15;

        let env = (1.0 - (-t * 20.0).exp()) * (-t * 5.0).exp();

        let sample = ((warp + pulse + static_noise) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate damage ability sound - power surge
pub fn generate_ability_damage() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.3;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Power charge
        let charge = (2.0 * PI * (500.0 + t * 600.0) * t).sin() * 0.4;

        // Impact hit
        let impact = (2.0 * PI * 150.0 * t).sin() * (-t * 30.0).exp() * 0.5;

        // Crackle
        let crackle = if fastrand::f32() < 0.12 {
            (fastrand::f32() * 2.0 - 1.0) * 0.35
        } else {
            0.0
        };

        let env = (1.0 - (-t * 35.0).exp()) * (-t * 6.0).exp();

        let sample = ((charge + impact + crackle) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

#[cfg(test)]
mod sound_tests {
    use super::*;
    #[test]
    fn combat_cues_have_headroom_distinct_timbres_and_clean_edges() {
        let cues = [
            ("autocannon", generate_autocannon().unwrap()),
            ("laser", generate_laser().unwrap()),
            ("railgun", generate_railgun().unwrap()),
            ("missile", generate_missile().unwrap()),
            ("drone", generate_drone().unwrap()),
            ("carrier-warp", generate_carrier_warp().unwrap()),
            ("explosion-heavy", generate_explosion(1.4, 50.0).unwrap()),
            ("impact-shield", generate_shield_hit().unwrap()),
            ("impact-armor", generate_armor_hit().unwrap()),
            ("impact-hull", generate_hull_hit().unwrap()),
            ("warning-shield", generate_shield_warning().unwrap()),
            ("warning-armor", generate_armor_warning().unwrap()),
            ("warning-hull", generate_hull_warning().unwrap()),
        ];
        let mut preview = Vec::new();
        let output = std::env::var_os("REBELLION_AUDIO_PREVIEW_DIR").map(std::path::PathBuf::from);
        if let Some(path) = &output {
            std::fs::create_dir_all(path).unwrap();
        }
        for (name, source) in &cues {
            assert_eq!(&source.bytes[..4], b"RIFF");
            let pcm: Vec<f32> = source.bytes[44..]
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32767.0)
                .collect();
            assert!(pcm.len() > 4000);
            let peak = pcm.iter().map(|s| s.abs()).fold(0.0, f32::max);
            assert!(
                peak > 0.1 && peak < 0.9,
                "{name}: unsafe or silent peak {peak}"
            );
            assert!(
                pcm.first().unwrap().abs() < 0.001 && pcm.last().unwrap().abs() < 0.005,
                "{name}: edge click"
            );
            preview.extend(pcm);
            preview.extend(std::iter::repeat_n(0.0, 11_025));
            if let Some(path) = &output {
                std::fs::write(path.join(format!("{name}.wav")), &source.bytes).unwrap();
            }
        }
        assert_ne!(
            cues[1].1.bytes, cues[2].1.bytes,
            "railguns must not reuse lasers"
        );
        if let Some(path) = &output {
            std::fs::write(
                path.join("preview.wav"),
                create_audio_source(&preview, 44_100).unwrap().bytes,
            )
            .unwrap();
        }
    }
}
