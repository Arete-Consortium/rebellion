//! HUD Wheel
//!
//! Capacitor-style HUD display:
//! - Three concentric semicircular health arcs (Shield/Armor/Structure)
//! - Capacitor ring inside the wheel center (fills as capacitor recharges)
//! - Capacitor percentage text in the center
//! - Pulsing red border when capacitor < 15%
//! - Mobile / desktop position variants

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::f32::consts::PI;

use crate::core::*;
use crate::entities::{Player, ShipStats};

/// Capacitor wheel plugin
pub struct CapacitorWheelPlugin;

impl Plugin for CapacitorWheelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CapacitorAnimation>().add_systems(
            Update,
            (update_capacitor_animation, draw_capacitor_wheel)
                .chain()
                .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight)))
                .after(bevy_egui::EguiSet::ProcessInput),
        );
    }
}

/// Animation state for capacitor effects
#[derive(Resource)]
pub struct CapacitorAnimation {
    pub rotation: f32,
    pub pulse: f32,
    pub pulse_direction: f32,
}

impl Default for CapacitorAnimation {
    fn default() -> Self {
        Self {
            rotation: 0.0,
            pulse: 1.0,
            pulse_direction: 1.0,
        }
    }
}

/// Update capacitor animation
fn update_capacitor_animation(time: Res<Time>, mut anim: ResMut<CapacitorAnimation>) {
    let dt = time.delta_secs();

    // Very slow rotation for capacitor glow effect
    anim.rotation += dt * 0.15;
    if anim.rotation > PI * 2.0 {
        anim.rotation -= PI * 2.0;
    }

    // Subtle pulsing
    anim.pulse += anim.pulse_direction * dt * 0.5;
    if anim.pulse > 1.1 {
        anim.pulse = 1.1;
        anim.pulse_direction = -1.0;
    } else if anim.pulse < 0.9 {
        anim.pulse = 0.9;
        anim.pulse_direction = 1.0;
    }
}

/// Draw capacitor wheel using egui
fn draw_capacitor_wheel(
    mut egui_ctx: EguiContexts,
    player_query: Query<&ShipStats, With<Player>>,
    anim: Res<CapacitorAnimation>,
    windows: Query<&Window>,
    mobile: Res<crate::systems::touch_joystick::MobileMode>,
) {
    let Ok(stats) = player_query.get_single() else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(ctx) = egui_ctx.try_ctx_mut() else {
        return;
    };

    // Position: desktop = bottom-right corner; mobile = centered between
    // the two virtual joysticks, lifted ~50px so it sits above them
    // rather than on top of either stick.
    let wheel_radius = 38.0;
    let (center_x, center_y) = if mobile.active {
        // Sit in the horizontal gap between the two joysticks, at the
        // same vertical level as the stick centers (96px from bottom).
        // Wheel radius is 38px, joysticks anchor 96px in from each side
        // so there's ~200-280px of clear horizontal space in the middle.
        (window.width() * 0.5, window.height() - 96.0)
    } else {
        // center_y = height - 80 keeps the outer sensor ring's bottom
        // edge at height - 19 (19px breathing room after the area
        // padding, see fixed_pos / response.rect.center math).
        (window.width() - 70.0, window.height() - 80.0)
    };

    // Calculate percentages
    let shield_pct = (stats.shield / stats.max_shield).clamp(0.0, 1.0);
    let armor_pct = (stats.armor / stats.max_armor).clamp(0.0, 1.0);
    let hull_pct = (stats.hull / stats.max_hull).clamp(0.0, 1.0);
    let cap_pct = (stats.capacitor / stats.max_capacitor).clamp(0.0, 1.0);

    // Draw using egui Area
    egui::Area::new(egui::Id::new("capacitor_wheel"))
        .fixed_pos(egui::pos2(
            center_x - wheel_radius - 45.0,
            center_y - wheel_radius - 25.0,
        ))
        .show(ctx, |ui| {
            let size = egui::vec2((wheel_radius + 50.0) * 2.0, (wheel_radius + 40.0) * 2.0);
            let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
            let center = response.rect.center();

            // === OUTER SENSOR OVERLAY RING ===
            painter.circle_stroke(
                center,
                wheel_radius + 8.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 40, 50)),
            );

            // === MAIN DARK BACKGROUND ===
            painter.circle_filled(
                center,
                wheel_radius + 3.0,
                egui::Color32::from_rgba_unmultiplied(12, 15, 22, 250),
            );
            painter.circle_stroke(
                center,
                wheel_radius + 3.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(40, 45, 55)),
            );

            // === HEALTH ARCS (top semicircle) ===
            let arc_width = 5.0; // Scaled down for smaller wheel
            let arc_gap = 1.5;
            let arc_start = -PI; // Left
            let arc_end = 0.0; // Right (top semicircle)

            // Shield arc (outermost) - grayish white
            let shield_radius = wheel_radius - 2.0;
            draw_health_arc(
                &painter,
                center,
                shield_radius,
                arc_width,
                shield_pct,
                arc_start,
                arc_end,
                egui::Color32::from_rgb(210, 215, 225), // Filled - bright white-gray
                egui::Color32::from_rgb(35, 40, 50),    // Empty - dark
                24,
            );

            // Armor arc (middle)
            let armor_radius = shield_radius - arc_width - arc_gap;
            draw_health_arc(
                &painter,
                center,
                armor_radius,
                arc_width,
                armor_pct,
                arc_start,
                arc_end,
                egui::Color32::from_rgb(195, 200, 210),
                egui::Color32::from_rgb(32, 37, 47),
                20,
            );

            // Structure/Hull arc (innermost)
            let structure_radius = armor_radius - arc_width - arc_gap;
            draw_health_arc(
                &painter,
                center,
                structure_radius,
                arc_width,
                hull_pct,
                arc_start,
                arc_end,
                egui::Color32::from_rgb(180, 185, 195),
                egui::Color32::from_rgb(28, 33, 43),
                16,
            );

            // === CAPACITOR RINGS (concentric circles of dashes) ===
            // Inner / outer radii are hard-coded rather than derived
            // from the structure arc — at the current wheel size
            // (wheel_radius=38) the structure arc's inner edge sits
            // at 23px, leaving no room for a positive-width ring.
            // 10..22 = 12px band, well clear of the structure arc.
            let cap_inner_radius = 10.0;
            let cap_outer_radius = 22.0;
            draw_capacitor_rings(
                &painter,
                center,
                cap_inner_radius,
                cap_outer_radius,
                cap_pct,
                anim.pulse,
            );

            // === INNER SPEED DISPLAY CIRCLE ===
            painter.circle_filled(
                center,
                cap_inner_radius,
                egui::Color32::from_rgba_unmultiplied(20, 28, 40, 240),
            );
            painter.circle_stroke(
                center,
                cap_inner_radius,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 60, 75)),
            );

            // === CAPACITOR PERCENTAGE (center of wheel) ===
            let cap_color = capacitor_text_color(cap_pct);
            painter.text(
                egui::pos2(center.x, center.y - 2.0),
                egui::Align2::CENTER_CENTER,
                format!("{:.0}%", cap_pct * 100.0),
                egui::FontId::monospace(10.0),
                cap_color,
            );

            // === LOW CAPACITOR WARNING FLASH ===
            if cap_pct < 0.15 {
                // Pulsing red/orange border around the wheel
                let flash_alpha = ((anim.pulse - 0.9) * 10.0).sin().abs();
                let flash_color =
                    egui::Color32::from_rgba_unmultiplied(255, 100, 40, (flash_alpha * 80.0) as u8);
                painter.circle_stroke(
                    center,
                    wheel_radius + 4.0,
                    egui::Stroke::new(2.0, flash_color),
                );
            }
        });
}

/// Get text color based on capacitor percentage
fn capacitor_text_color(pct: f32) -> egui::Color32 {
    if pct < 0.15 {
        egui::Color32::from_rgb(200, 80, 80) // Critical - reddish
    } else if pct < 0.30 {
        egui::Color32::from_rgb(200, 150, 80) // Low - orange-ish
    } else {
        egui::Color32::from_rgb(255, 220, 80) // Normal - bright yellow
    }
}

/// Draw health arc (semicircular, segmented)
fn draw_health_arc(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    width: f32,
    fill_pct: f32,
    arc_start: f32,
    arc_end: f32,
    fill_color: egui::Color32,
    empty_color: egui::Color32,
    num_segments: u32,
) {
    let segment_gap = 0.02;
    let total_arc = arc_end - arc_start;
    let segment_arc = (total_arc / num_segments as f32) - segment_gap;

    // Fills segments from edges toward center
    let filled_segments = (fill_pct * num_segments as f32).ceil() as u32;

    for i in 0..num_segments {
        let angle_start = arc_start + (i as f32) * (total_arc / num_segments as f32);

        // Fill from both edges toward middle
        let half = num_segments / 2;
        let is_filled = if i < half {
            i < filled_segments / 2
        } else {
            (num_segments - i - 1) < filled_segments.div_ceil(2)
        };

        let color = if is_filled || fill_pct >= 1.0 {
            fill_color
        } else {
            empty_color
        };

        draw_arc_segment(
            painter,
            center,
            radius,
            width,
            angle_start,
            segment_arc,
            color,
        );
    }
}

/// Draw a single arc segment
fn draw_arc_segment(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    width: f32,
    start_angle: f32,
    arc_span: f32,
    color: egui::Color32,
) {
    let steps = 8;
    let inner_r = radius - width / 2.0;
    let outer_r = radius + width / 2.0;

    let mut points = Vec::with_capacity((steps + 1) * 2);

    // Outer arc
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = start_angle + arc_span * t;
        points.push(egui::pos2(
            center.x + outer_r * angle.cos(),
            center.y + outer_r * angle.sin(),
        ));
    }

    // Inner arc (reversed)
    for i in (0..=steps).rev() {
        let t = i as f32 / steps as f32;
        let angle = start_angle + arc_span * t;
        points.push(egui::pos2(
            center.x + inner_r * angle.cos(),
            center.y + inner_r * angle.sin(),
        ));
    }

    if points.len() >= 3 {
        painter.add(egui::Shape::convex_polygon(
            points,
            color,
            egui::Stroke::NONE,
        ));
    }
}

/// Draw CAPACITOR display
/// Bright yellow dashes when full, grey when depleted
fn draw_capacitor_rings(
    painter: &egui::Painter,
    center: egui::Pos2,
    inner_radius: f32,
    outer_radius: f32,
    cap_pct: f32,
    pulse: f32,
) {
    // Ring of dashes/cells
    let num_cells = 18;

    // Cells filled based on capacitor percentage
    let filled_cells = (cap_pct * num_cells as f32).round() as u32;

    // Ring dimensions
    let ring_width = outer_radius - inner_radius - 2.0;
    let cell_inner = inner_radius + 1.0;
    let cell_outer = cell_inner + ring_width;

    // Draw cells in a full circle
    for i in 0..num_cells {
        // Start from top, go clockwise
        let angle = -PI / 2.0 + (i as f32 / num_cells as f32) * PI * 2.0;
        let cell_arc = (PI * 2.0 / num_cells as f32) * 0.65; // 65% fill, 35% gap

        // Fill from index 0 up
        let is_filled = i < filled_cells;

        // Bright yellow when full, grey when empty
        let filled_color = {
            let pulse_mod = 0.92 + 0.08 * pulse;
            egui::Color32::from_rgb(
                (255.0 * pulse_mod) as u8, // Bright yellow
                (220.0 * pulse_mod) as u8,
                (50.0 * pulse_mod) as u8,
            )
        };
        let empty_color = egui::Color32::from_rgb(45, 48, 55); // Grey

        let fill_color = if is_filled { filled_color } else { empty_color };

        // Draw the dash/cell
        draw_cap_cell(
            painter,
            center,
            cell_inner,
            cell_outer,
            angle,
            cell_arc,
            fill_color,
            egui::Color32::from_rgb(25, 28, 35), // Dark border
            is_filled,
        );
    }

    // Inner dark circle
    painter.circle_filled(center, inner_radius, egui::Color32::from_rgb(12, 14, 20));

    // Subtle yellow glow in center when cap is high
    if cap_pct > 0.5 {
        let glow_alpha = ((cap_pct - 0.5) * 0.6 * 255.0 * pulse) as u8;
        painter.circle_filled(
            center,
            inner_radius * 0.6,
            egui::Color32::from_rgba_unmultiplied(255, 220, 80, glow_alpha),
        );
    }
}

/// Draw a single capacitor cell
fn draw_cap_cell(
    painter: &egui::Painter,
    center: egui::Pos2,
    inner_radius: f32,
    outer_radius: f32,
    center_angle: f32,
    arc_width: f32,
    fill_color: egui::Color32,
    border_color: egui::Color32,
    is_filled: bool,
) {
    let half_arc = arc_width / 2.0;
    let start_angle = center_angle - half_arc;
    let end_angle = center_angle + half_arc;

    // Create trapezoid shape
    let points = vec![
        egui::pos2(
            center.x + inner_radius * start_angle.cos(),
            center.y + inner_radius * start_angle.sin(),
        ),
        egui::pos2(
            center.x + outer_radius * start_angle.cos(),
            center.y + outer_radius * start_angle.sin(),
        ),
        egui::pos2(
            center.x + outer_radius * end_angle.cos(),
            center.y + outer_radius * end_angle.sin(),
        ),
        egui::pos2(
            center.x + inner_radius * end_angle.cos(),
            center.y + inner_radius * end_angle.sin(),
        ),
    ];

    // Fill
    painter.add(egui::Shape::convex_polygon(
        points.clone(),
        fill_color,
        egui::Stroke::NONE,
    ));

    // Border
    painter.add(egui::Shape::closed_line(
        points.clone(),
        egui::Stroke::new(0.5, border_color),
    ));

    // Add highlight on filled cells (top edge glow)
    if is_filled {
        let highlight_points = vec![
            egui::pos2(
                center.x + (outer_radius - 1.0) * start_angle.cos(),
                center.y + (outer_radius - 1.0) * start_angle.sin(),
            ),
            egui::pos2(
                center.x + outer_radius * start_angle.cos(),
                center.y + outer_radius * start_angle.sin(),
            ),
            egui::pos2(
                center.x + outer_radius * end_angle.cos(),
                center.y + outer_radius * end_angle.sin(),
            ),
            egui::pos2(
                center.x + (outer_radius - 1.0) * end_angle.cos(),
                center.y + (outer_radius - 1.0) * end_angle.sin(),
            ),
        ];
        painter.add(egui::Shape::convex_polygon(
            highlight_points,
            egui::Color32::from_rgba_unmultiplied(255, 255, 200, 40),
            egui::Stroke::NONE,
        ));
    }
}
