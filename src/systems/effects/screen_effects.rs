//! Screen-level effects: shake, hit stop, screen flash, camera zoom

use crate::core::*;
use bevy::prelude::*;

// =============================================================================
// SCREEN SHAKE
// =============================================================================

/// Screen shake state
#[derive(Resource)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
    pub timer: f32,
    /// Global multiplier for shake intensity (0.0 = disabled, 1.0 = full)
    pub multiplier: f32,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            duration: 0.0,
            timer: 0.0,
            multiplier: 1.0, // Full intensity by default
        }
    }
}

impl ScreenShake {
    /// Trigger a screen shake
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        if intensity > self.intensity || self.timer <= 0.0 {
            self.intensity = intensity;
            self.duration = duration;
            self.timer = duration;
        }
    }

    /// Small shake (player hit)
    pub fn small(&mut self) {
        self.trigger(4.0, 0.12);
    }

    /// Medium shake (enemy explosion)
    pub fn medium(&mut self) {
        self.trigger(8.0, 0.2);
    }

    /// Large shake (boss phase change)
    pub fn large(&mut self) {
        self.trigger(15.0, 0.3);
    }

    /// Massive shake (boss defeat)
    pub fn massive(&mut self) {
        self.trigger(25.0, 0.5);
    }
}

/// Handle screen shake events
pub fn update_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut shake_events: EventReader<ScreenShakeEvent>,
) {
    // Process new shake events
    for event in shake_events.read() {
        if event.intensity > shake.intensity {
            shake.intensity = event.intensity;
            shake.duration = event.duration;
            shake.timer = event.duration;
        }
    }

    let dt = time.delta_secs();

    if shake.timer > 0.0 {
        shake.timer -= dt;

        let progress = shake.timer / shake.duration;
        // Apply global multiplier to shake intensity
        let current_intensity = shake.intensity * progress * shake.multiplier;

        if let Ok(mut transform) = camera_query.get_single_mut() {
            let offset_x = (fastrand::f32() - 0.5) * 2.0 * current_intensity;
            let offset_y = (fastrand::f32() - 0.5) * 2.0 * current_intensity;
            transform.translation.x = offset_x;
            transform.translation.y = offset_y;
        }
    } else {
        // Reset camera
        if let Ok(mut transform) = camera_query.get_single_mut() {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        }
    }
}

// =============================================================================
// HIT STOP (FREEZE FRAME)
// =============================================================================

/// Hit stop (freeze frame) resource for dramatic pauses
#[derive(Resource, Default)]
pub struct HitStop {
    /// Remaining freeze time
    pub timer: f32,
    /// Time scale during hit stop (0 = full freeze)
    pub time_scale: f32,
}

impl HitStop {
    /// Trigger a hit stop
    pub fn trigger(&mut self, duration: f32) {
        self.timer = duration.max(self.timer); // Don't override longer stops
        self.time_scale = 0.05; // Near-freeze
    }

    /// Small hit stop (minor impact)
    pub fn small(&mut self) {
        self.trigger(0.03);
    }

    /// Medium hit stop (significant hit)
    pub fn medium(&mut self) {
        self.trigger(0.06);
    }

    /// Large hit stop (boss phase, heavy damage)
    pub fn large(&mut self) {
        self.trigger(0.1);
    }

    /// Massive hit stop (boss defeat, player death)
    pub fn massive(&mut self) {
        self.trigger(0.15);
    }

    /// Check if hit stop is active
    pub fn is_active(&self) -> bool {
        self.timer > 0.0
    }

    /// Get current time multiplier (for game systems to use)
    pub fn time_mult(&self) -> f32 {
        if self.timer > 0.0 {
            self.time_scale
        } else {
            1.0
        }
    }

    /// Update hit stop timer
    pub fn update(&mut self, dt: f32) {
        if self.timer > 0.0 {
            // Use real time, not game time (so freeze actually works)
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.timer = 0.0;
                self.time_scale = 1.0;
            }
        }
    }
}

/// Update hit stop effect
pub fn update_hit_stop(time: Res<Time>, mut hit_stop: ResMut<HitStop>) {
    // Use delta_secs which is already real time
    hit_stop.update(time.delta_secs());
}

// =============================================================================
// SCREEN FLASH
// =============================================================================

/// Screen-wide flash effect for big explosions
#[derive(Resource, Default)]
pub struct ScreenFlash {
    /// Current flash intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Flash color
    pub color: Color,
    /// Fade speed
    pub fade_speed: f32,
}

impl ScreenFlash {
    /// Trigger a white screen flash
    pub fn white(&mut self, intensity: f32) {
        self.intensity = intensity.min(1.0);
        self.color = Color::WHITE;
        self.fade_speed = 4.0;
    }

    /// Trigger a colored screen flash
    pub fn colored(&mut self, color: Color, intensity: f32) {
        self.intensity = intensity.min(1.0);
        self.color = color;
        self.fade_speed = 4.0;
    }

    /// Trigger flash for massive explosion (boss kill)
    pub fn massive(&mut self) {
        self.white(0.8);
        self.fade_speed = 2.0; // Slower fade for dramatic effect
    }

    /// Trigger flash for large explosion
    pub fn large(&mut self) {
        self.white(0.5);
    }

    /// Subtle punch flash for crit hits — low intensity, fast fade.
    pub fn brief(&mut self) {
        self.white(0.10);
        self.fade_speed = 8.0;
    }

    /// Trigger red flash for salt miner activation
    pub fn salt_miner(&mut self) {
        self.colored(Color::srgb(1.0, 0.2, 0.2), 0.6);
        self.fade_speed = 3.0;
    }
}

/// Marker component for screen flash overlay sprite
#[derive(Component)]
pub struct ScreenFlashOverlay;

/// Update screen flash effect
pub fn update_screen_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash: ResMut<ScreenFlash>,
    mut overlay_query: Query<(Entity, &mut Sprite), With<ScreenFlashOverlay>>,
) {
    let dt = time.delta_secs();

    if flash.intensity > 0.0 {
        // Fade out
        flash.intensity = (flash.intensity - flash.fade_speed * dt).max(0.0);

        // Update or create overlay
        if let Ok((_, mut sprite)) = overlay_query.get_single_mut() {
            sprite.color = flash.color.with_alpha(flash.intensity);
        } else if flash.intensity > 0.01 {
            // Spawn overlay sprite covering screen
            commands.spawn((
                ScreenFlashOverlay,
                Sprite {
                    color: flash.color.with_alpha(flash.intensity),
                    custom_size: Some(Vec2::new(SCREEN_WIDTH + 100.0, SCREEN_HEIGHT + 100.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, LAYER_HUD + 10.0), // Above everything
            ));
        }
    } else {
        // Remove overlay when done
        for (entity, _) in overlay_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// CAMERA ZOOM PULSE
// =============================================================================

/// Camera zoom pulse for dramatic moments (boss kills)
#[derive(Resource)]
pub struct CameraZoom {
    /// Target scale (1.0 = normal, 1.1 = 10% zoom in)
    pub target_scale: f32,
    /// Current scale
    pub current_scale: f32,
    /// Return speed (how fast to return to normal)
    pub return_speed: f32,
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self {
            target_scale: 1.0,
            current_scale: 1.0,
            return_speed: 3.0,
        }
    }
}

impl CameraZoom {
    /// Trigger a zoom pulse (zoom in then out)
    pub fn pulse(&mut self, intensity: f32) {
        self.target_scale = 1.0 + intensity;
        self.return_speed = 3.0;
    }

    /// Quick dramatic zoom for boss kills
    pub fn boss_kill(&mut self) {
        self.pulse(0.08); // 8% zoom in
        self.return_speed = 2.0; // Slower return for drama
    }

    /// Small zoom for regular kills
    pub fn small(&mut self) {
        self.pulse(0.02);
        self.return_speed = 5.0;
    }
}

/// Update camera zoom effect
pub fn update_camera_zoom(
    time: Res<Time>,
    mut zoom: ResMut<CameraZoom>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
) {
    let dt = time.delta_secs();

    // Move current scale toward target
    if zoom.current_scale != zoom.target_scale {
        let diff = zoom.target_scale - zoom.current_scale;
        zoom.current_scale += diff * 8.0 * dt; // Fast zoom in

        // Apply to camera
        if let Ok(mut projection) = camera_query.get_single_mut() {
            projection.scale = zoom.current_scale;
        }
    }

    // Return target to 1.0 over time
    if zoom.target_scale > 1.0 {
        zoom.target_scale = (zoom.target_scale - zoom.return_speed * dt).max(1.0);
    }

    // Snap to 1.0 when close
    if (zoom.current_scale - 1.0).abs() < 0.001 && zoom.target_scale == 1.0 {
        zoom.current_scale = 1.0;
        if let Ok(mut projection) = camera_query.get_single_mut() {
            projection.scale = 1.0;
        }
    }
}
