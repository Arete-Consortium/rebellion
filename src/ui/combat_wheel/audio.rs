use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;

use super::events::CombatWheelEvent;

/// Manages HUD audio feedback with throttling to prevent spam.
///
/// Audio assets are not yet shipped (see `/assets/sounds/` TODO). Real sound
/// playback requires asset paths; this module is wired and functional but
/// produces no audible output until assets land. Console fallback matches the
/// prototype's behavior.
#[derive(Resource, Default)]
pub struct AudioManager {
    pub last_heat_warning: f32,
    pub last_integrity_critical: f32,
    pub last_insufficient_capacitor: f32,
}

/// Plays audio cues in response to CombatWheelEvents.
/// Uses placeholder console output when sound assets are not available.
pub fn update_audio_feedback(
    mut events: EventReader<CombatWheelEvent>,
    mut audio_manager: ResMut<AudioManager>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for event in events.read() {
        let now = time.elapsed_secs();

        let (sound_path, _cooldown) = match event {
            CombatWheelEvent::ModuleActivated { slot_id } => {
                println!("[AUDIO] Module activated: {:?}", slot_id);
                (Some("sounds/module_activate.ogg"), 0.0)
            }
            CombatWheelEvent::ModuleRejected { slot_id, reason } => {
                println!("[AUDIO] Module rejected: {:?} - {:?}", slot_id, reason);
                (Some("sounds/module_reject.ogg"), 0.0)
            }
            CombatWheelEvent::ModuleCooledDown { slot_id } => {
                println!("[AUDIO] Module ready: {:?}", slot_id);
                (Some("sounds/module_ready.ogg"), 0.0)
            }
            CombatWheelEvent::ShieldCollapsed => {
                println!("[AUDIO] Shield collapsed!");
                (Some("sounds/shield_collapse.ogg"), 0.0)
            }
            CombatWheelEvent::IntegrityCritical => {
                if now - audio_manager.last_integrity_critical < 3.0 {
                    continue;
                }
                audio_manager.last_integrity_critical = now;
                println!("[AUDIO] Integrity critical!");
                (Some("sounds/hull_critical.ogg"), 3.0)
            }
            CombatWheelEvent::HeatWarning => {
                if now - audio_manager.last_heat_warning < 2.0 {
                    continue;
                }
                audio_manager.last_heat_warning = now;
                println!("[AUDIO] Heat warning");
                (Some("sounds/heat_warning.ogg"), 2.0)
            }
            CombatWheelEvent::HeatCritical => {
                if now - audio_manager.last_heat_warning < 1.0 {
                    continue;
                }
                audio_manager.last_heat_warning = now;
                println!("[AUDIO] Heat critical!");
                (Some("sounds/heat_critical.ogg"), 1.0)
            }
            CombatWheelEvent::Overheated => {
                println!("[AUDIO] Overheated!");
                (Some("sounds/overheat.ogg"), 0.0)
            }
            CombatWheelEvent::CoolingComplete => {
                println!("[AUDIO] Cooling complete");
                (Some("sounds/cooling_done.ogg"), 0.0)
            }
            CombatWheelEvent::PowerupAcquired { effect } => {
                println!("[AUDIO] Powerup acquired: {:?}", effect);
                (Some("sounds/powerup_get.ogg"), 0.0)
            }
            CombatWheelEvent::PowerupExpiring { .. } => {
                println!("[AUDIO] Powerup expiring");
                (Some("sounds/powerup_expire.ogg"), 0.0)
            }
            CombatWheelEvent::SentryDeployed { count } => {
                println!("[AUDIO] Sentries deployed: {}", count);
                (Some("sounds/sentry_deploy.ogg"), 0.0)
            }
            CombatWheelEvent::SentryNetworkDegraded => {
                println!("[AUDIO] Sentry network degraded");
                (Some("sounds/sentry_disabled.ogg"), 0.0)
            }
            CombatWheelEvent::InsufficientCapacitor { .. } => {
                if now - audio_manager.last_insufficient_capacitor < 1.0 {
                    continue;
                }
                audio_manager.last_insufficient_capacitor = now;
                println!("[AUDIO] Insufficient capacitor");
                (Some("sounds/insufficient_cap.ogg"), 1.0)
            }
            _ => (None, 0.0),
        };

        if let Some(path) = sound_path {
            // Try to load and play; falls back to console output if asset missing.
            // Bevy 0.15 audio: AudioBundle was removed in favor of AudioPlayer
            // (component) + AudioSource. We attach both to a one-shot entity.
            let handle: Handle<AudioSource> = asset_server.load(path);
            commands.spawn((
                AudioPlayer::new(handle),
                PlaybackSettings::DESPAWN.with_volume(Volume::new(0.7)),
            ));
        }
    }
}
