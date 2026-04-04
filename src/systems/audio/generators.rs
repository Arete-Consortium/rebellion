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

/// Generate autocannon sound - deep industrial thump
pub fn generate_autocannon() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.12;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Deep bass thump
        let bass = (2.0 * PI * 80.0 * t).sin() * 0.5;

        // Mid punch
        let mid = (2.0 * PI * 200.0 * t).sin() * (-t * 50.0).exp() * 0.4;

        // High crack
        let crack = (2.0 * PI * 600.0 * t).sin() * (-t * 80.0).exp() * 0.3;

        // Noise burst
        let noise = (fastrand::f32() * 2.0 - 1.0) * (-t * 40.0).exp() * 0.2;

        // Envelope
        let env = (-t * 15.0).exp();

        let sample = ((bass + mid + crack + noise) * env * 0.8).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate laser sound - high-pitched zap
pub fn generate_laser() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.15;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Descending frequency
        let freq = 1200.0 - t * 3000.0;
        let wave = (2.0 * PI * freq * t).sin();

        // Add harmonics
        let harm = (2.0 * PI * freq * 2.0 * t).sin() * 0.3;

        // Envelope
        let env = (-t * 20.0).exp();

        let sample = ((wave + harm) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate missile launch sound - whooshing rocket
pub fn generate_missile() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.2;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Whoosh noise
        let noise = (fastrand::f32() * 2.0 - 1.0) * 0.5;

        // Rising frequency for ignition
        let freq = 150.0 + t * 400.0;
        let rumble = (2.0 * PI * freq * t).sin() * 0.4;

        // High hiss
        let hiss = (2.0 * PI * 2000.0 * t).sin() * 0.15 * (-t * 20.0).exp();

        let env = (1.0 - (-t * 30.0).exp()) * (-t * 8.0).exp();

        let sample = ((noise + rumble + hiss) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

// =============================================================================
// EXPLOSION / IMPACT SOUND GENERATORS
// =============================================================================

/// Generate explosion sound
pub fn generate_explosion(duration: f32, base_freq: f32) -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Low rumble
        let rumble = (2.0 * PI * base_freq * t).sin() * 0.5;

        // Noise
        let noise = (fastrand::f32() * 2.0 - 1.0) * 0.6;

        // Crackle (filtered noise bursts)
        let crackle = if fastrand::f32() < 0.1 {
            fastrand::f32() * 2.0 - 1.0
        } else {
            0.0
        } * (-t * 5.0).exp()
            * 0.3;

        // Envelope - quick attack, slow decay
        let env = (1.0 - (-t * 30.0).exp()) * (-t * 4.0).exp();

        let sample = ((rumble + noise + crackle) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
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

/// Generate shield hit sound - electric crackle
pub fn generate_shield_hit() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.08;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // High frequency buzz
        let buzz = (2.0 * PI * 800.0 * t).sin() * 0.4;

        // Electric crackle
        let crackle = (fastrand::f32() * 2.0 - 1.0) * 0.5;

        let env = (-t * 30.0).exp();

        let sample = ((buzz + crackle) * env * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate armor hit sound - metallic clang
pub fn generate_armor_hit() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.1;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Metallic frequencies
        let f1 = (2.0 * PI * 300.0 * t).sin() * 0.5;
        let f2 = (2.0 * PI * 450.0 * t).sin() * 0.3;
        let f3 = (2.0 * PI * 180.0 * t).sin() * 0.4;

        let env = (-t * 25.0).exp();

        let sample = ((f1 + f2 + f3) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate hull hit sound - deep crunch
pub fn generate_hull_hit() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.12;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Low crunch
        let crunch = (2.0 * PI * 100.0 * t).sin() * 0.6;

        // Noise
        let noise = (fastrand::f32() * 2.0 - 1.0) * 0.4;

        let env = (-t * 20.0).exp();

        let sample = ((crunch + noise) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

// =============================================================================
// WARNING SOUND GENERATORS
// =============================================================================

/// Generate shield warning - high-pitched triple beep
pub fn generate_shield_warning() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.6;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Three beeps
        let beep_duration = 0.12;
        let gap = 0.08;
        let cycle = beep_duration + gap;

        let beep_num = (t / cycle).floor() as i32;
        let beep_t = t - (beep_num as f32 * cycle);

        let sample = if beep_num < 3 && beep_t < beep_duration {
            let freq = 1200.0; // High pitched
            let wave = (2.0 * PI * freq * beep_t).sin();
            let env = (1.0 - (beep_t / beep_duration)).powf(0.5);
            wave * env * 0.6
        } else {
            0.0
        };

        samples.push(sample.clamp(-1.0, 1.0));
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate armor warning - mid-tone double beep with urgency
pub fn generate_armor_warning() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.5;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Two longer beeps
        let beep_duration = 0.15;
        let gap = 0.1;
        let cycle = beep_duration + gap;

        let beep_num = (t / cycle).floor() as i32;
        let beep_t = t - (beep_num as f32 * cycle);

        let sample = if beep_num < 2 && beep_t < beep_duration {
            let freq = 800.0; // Mid tone
            let wave = (2.0 * PI * freq * beep_t).sin();
            // Add slight harmonic for urgency
            let harm = (2.0 * PI * freq * 1.5 * beep_t).sin() * 0.3;
            let env = (1.0 - (beep_t / beep_duration)).powf(0.3);
            (wave + harm) * env * 0.7
        } else {
            0.0
        };

        samples.push(sample.clamp(-1.0, 1.0));
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate hull warning - low urgent alarm (critical warning)
pub fn generate_hull_warning() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.8;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Continuous warbling alarm
        let base_freq = 400.0;
        // Frequency modulation for urgency
        let mod_freq = 8.0; // 8 Hz wobble
        let freq = base_freq + 100.0 * (2.0 * PI * mod_freq * t).sin();

        let wave = (2.0 * PI * freq * t).sin();
        // Add harmonics for harshness
        let harm1 = (2.0 * PI * freq * 2.0 * t).sin() * 0.4;
        let harm2 = (2.0 * PI * freq * 3.0 * t).sin() * 0.2;

        // Envelope with attack
        let env = (1.0 - (-t * 20.0).exp()) * (1.0 - (t / duration).powf(2.0));

        let sample = (wave + harm1 + harm2) * env * 0.65;
        samples.push(sample.clamp(-1.0, 1.0));
    }

    create_audio_source(&samples, sample_rate)
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
