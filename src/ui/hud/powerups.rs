//! Powerup indicator and buff expiration warning systems

use super::common::*;
use crate::entities::{Player, PowerupEffects};
use bevy::prelude::*;

/// Update powerup effect indicators - show/hide boxes and update timer bars
pub fn update_powerup_indicators(
    time: Res<Time>,
    player_query: Query<&PowerupEffects, With<Player>>,
    mut status_box_query: Query<(&PowerupStatusBox, &mut Node, &mut BackgroundColor)>,
    mut timer_bar_query: Query<
        (&PowerupTimerBar, &mut Node, &mut BackgroundColor),
        Without<PowerupStatusBox>,
    >,
    mut countdown_query: Query<
        (&PowerupCountdown, &mut Text, &mut Node, &mut TextColor),
        (Without<PowerupStatusBox>, Without<PowerupTimerBar>),
    >,
) {
    let Ok(effects) = player_query.get_single() else {
        return;
    };

    // Max durations for each powerup type
    const OVERDRIVE_MAX: f32 = 5.0;
    const DAMAGE_BOOST_MAX: f32 = 10.0;
    const INVULN_MAX: f32 = 3.0;

    // Get current timer values
    let get_timer = |powerup_type: PowerupType| -> (f32, f32) {
        match powerup_type {
            PowerupType::Overdrive => (effects.overdrive_timer, OVERDRIVE_MAX),
            PowerupType::DamageBoost => (effects.damage_boost_timer, DAMAGE_BOOST_MAX),
            PowerupType::Invulnerability => (effects.invuln_timer, INVULN_MAX),
        }
    };

    let elapsed = time.elapsed_secs();

    // Update status box visibility and pulsing
    for (status_box, mut node, mut bg_color) in status_box_query.iter_mut() {
        let (timer, _max) = get_timer(status_box.powerup_type);

        if timer > 0.0 {
            node.display = Display::Flex;

            // Enhanced pulsing when timer is low
            if timer < BUFF_WARNING_THRESHOLD {
                // Faster pulse as timer gets lower
                let urgency = 1.0 - (timer / BUFF_WARNING_THRESHOLD);
                let pulse_speed = 8.0 + urgency * 12.0; // 8-20 Hz
                let pulse = (elapsed * pulse_speed).sin() * 0.5 + 0.5;

                // More dramatic red flash
                let red = 0.4 + pulse * 0.4;
                let alpha = 0.9 + pulse * 0.1;
                bg_color.0 = Color::srgba(red, 0.1, 0.1, alpha);
            } else {
                bg_color.0 = Color::srgba(0.1, 0.1, 0.15, 0.9);
            }
        } else {
            node.display = Display::None;
        }
    }

    // Update timer bar widths
    for (timer_bar, mut node, mut bg_color) in timer_bar_query.iter_mut() {
        let (timer, max) = get_timer(timer_bar.powerup_type);

        if timer > 0.0 {
            let percent = (timer / max * 100.0).clamp(0.0, 100.0);
            node.width = Val::Percent(percent);

            // Enhanced color pulsing when timer is low
            if timer < BUFF_WARNING_THRESHOLD {
                let urgency = 1.0 - (timer / BUFF_WARNING_THRESHOLD);
                let pulse_speed = 8.0 + urgency * 12.0;
                let pulse = (elapsed * pulse_speed).sin() * 0.5 + 0.5;

                // Pulse between orange and bright red
                bg_color.0 = Color::srgb(1.0, 0.2 + pulse * 0.4, 0.1);
            }
        }
    }

    // Update countdown text
    for (countdown, mut text, mut node, mut text_color) in countdown_query.iter_mut() {
        let (timer, _max) = get_timer(countdown.powerup_type);

        if timer > 0.0 && timer < BUFF_WARNING_THRESHOLD {
            node.display = Display::Flex;

            // Show remaining time with one decimal
            **text = format!("{:.1}", timer);

            // Pulse the countdown text color
            let urgency = 1.0 - (timer / BUFF_WARNING_THRESHOLD);
            let pulse_speed = 10.0 + urgency * 15.0;
            let pulse = (elapsed * pulse_speed).sin() * 0.5 + 0.5;

            // Flash between red and white
            let r = 1.0;
            let g = 0.3 + pulse * 0.7;
            let b = 0.3 + pulse * 0.7;
            text_color.0 = Color::srgb(r, g, b);
        } else {
            node.display = Display::None;
            **text = String::new();
        }
    }
}

/// Update screen edge warning overlays when buffs are expiring
pub fn update_buff_expiration_warnings(
    time: Res<Time>,
    player_query: Query<&PowerupEffects, With<Player>>,
    mut warning_query: Query<(&BuffExpirationWarning, &mut BackgroundColor)>,
) {
    let Ok(effects) = player_query.get_single() else {
        // Hide all warnings if no player
        for (_, mut bg) in warning_query.iter_mut() {
            bg.0 = Color::NONE;
        }
        return;
    };

    // Find the most urgent expiring buff
    let mut most_urgent: Option<(f32, Color)> = None;

    // Check each buff timer
    let buffs = [
        (effects.overdrive_timer, Color::srgb(0.3, 0.9, 1.0)), // Cyan
        (effects.damage_boost_timer, Color::srgb(1.0, 0.4, 0.2)), // Orange/red
        (effects.invuln_timer, Color::srgb(1.0, 0.9, 0.4)),    // Gold
    ];

    for (timer, color) in buffs {
        if timer > 0.0 && timer < BUFF_WARNING_THRESHOLD {
            match &most_urgent {
                Some((urgency, _)) if timer < *urgency => {
                    most_urgent = Some((timer, color));
                }
                None => {
                    most_urgent = Some((timer, color));
                }
                _ => {}
            }
        }
    }

    // Update edge colors based on most urgent buff
    if let Some((timer, color)) = most_urgent {
        let elapsed = time.elapsed_secs();

        // Calculate urgency (0 = just started warning, 1 = about to expire)
        let urgency = 1.0 - (timer / BUFF_WARNING_THRESHOLD);

        // Pulse speed increases with urgency (6-16 Hz)
        let pulse_speed = 6.0 + urgency * 10.0;
        let pulse = (elapsed * pulse_speed).sin() * 0.5 + 0.5;

        // Alpha increases with urgency and pulse
        let base_alpha = 0.3 + urgency * 0.4; // 0.3 to 0.7
        let alpha = base_alpha * (0.5 + pulse * 0.5);

        for (_, mut bg) in warning_query.iter_mut() {
            bg.0 = color.with_alpha(alpha);
        }
    } else {
        // No expiring buffs - hide warnings
        for (_, mut bg) in warning_query.iter_mut() {
            bg.0 = Color::NONE;
        }
    }
}
