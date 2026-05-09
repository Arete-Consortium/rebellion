//! On-screen virtual joysticks for mobile / touch devices.
//!
//! Two anchored sticks at the bottom corners of the screen — left =
//! ship movement, right = weapon aim + fire. Touch input is converted
//! into the existing `JoystickState` resource so every consumer system
//! (movement, shooting, abilities) keeps working unchanged.
//!
//! Mobile mode is auto-detected at startup based on viewport size +
//! touch capability (WASM only). Native builds never enable it.
//!
//! Architecture: floating-anchor sticks with fixed visual base. Each
//! corner has a base ring + knob always visible while mobile mode is
//! active. The first touch in each half-screen anchors that side's
//! stick at the touch position; subsequent drag is read as offset from
//! anchor, clamped to the stick radius. The visual knob mirrors the
//! offset relative to the (fixed) base center so the player still sees
//! a familiar stick visual.

#![allow(dead_code)]

use bevy::prelude::*;

use super::joystick::JoystickState;
use crate::core::GameState;

/// Auto-detected at startup. False on native builds, false on desktop
/// browsers, true on touch-capable browsers with small viewports.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct MobileMode {
    pub active: bool,
}

#[derive(Default, Debug, Clone, Copy)]
struct VirtualStick {
    /// Touch ID currently driving this stick (None if released).
    touch_id: Option<u64>,
    /// Anchor in screen pixels — set on touch-down; subsequent drag
    /// reads offset from this point.
    anchor: Vec2,
    /// Latest displacement (pixels) clamped to STICK_RADIUS_PX.
    offset: Vec2,
}

#[derive(Default, Debug, Clone, Copy)]
struct TapCandidate {
    touch_id: u64,
    start_pos: Vec2,
    start_time: f32,
}

#[derive(Resource, Default, Debug)]
pub struct TouchJoystickState {
    left: VirtualStick,
    right: VirtualStick,
    /// In-flight tap candidates — added on touch-down outside stick
    /// zones, removed when released or when displacement exceeds the
    /// tap threshold. Released candidates within thresholds emit a
    /// confirm-button press for one frame.
    pending_taps: Vec<TapCandidate>,
}

/// Marker components for the joystick UI nodes.
#[derive(Component)]
struct LeftStickBase;
#[derive(Component)]
struct LeftStickKnob;
#[derive(Component)]
struct RightStickBase;
#[derive(Component)]
struct RightStickKnob;

/// Stick visual radius in pixels — also the maximum knob displacement.
const STICK_RADIUS_PX: f32 = 64.0;
/// Knob diameter as a fraction of base diameter.
const KNOB_DIAMETER_FRAC: f32 = 0.55;
/// Minimum offset (pixels) before the stick reads as engaged.
const STICK_DEAD_PX: f32 = 6.0;
/// Distance from the bottom-corner of the screen to the base center.
const STICK_INSET_PX: f32 = 96.0;
/// Threshold below which a viewport is treated as mobile.
const MOBILE_VIEWPORT_PX: f32 = 1024.0;
/// Touches that land at or above this fraction of the viewport height
/// belong to the stick zones; touches above it are tap candidates.
const STICK_ZONE_TOP_FRAC: f32 = 0.55;
/// Maximum displacement for a tap to count as a tap (not a drag), px.
const TAP_MAX_TRAVEL_PX: f32 = 28.0;
/// Maximum duration for a tap (seconds).
const TAP_MAX_DURATION_S: f32 = 0.32;

pub struct TouchJoystickPlugin;

impl Plugin for TouchJoystickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MobileMode>()
            .init_resource::<TouchJoystickState>()
            .add_systems(Startup, (detect_mobile_mode, spawn_joystick_ui).chain())
            // Touch → JoystickState runs in PreUpdate AFTER gamepad
            // polling so (a) touch wins on mobile, and (b) prev_buttons
            // is snapshotted before we synthesize a confirm press —
            // otherwise just_pressed() never fires.
            .add_systems(
                PreUpdate,
                update_touch_joystick.after(super::joystick::poll_gamepad),
            )
            .add_systems(Update, (update_stick_visibility, update_knob_positions));
    }
}

#[cfg(target_arch = "wasm32")]
fn detect_mobile_mode(mut mobile: ResMut<MobileMode>) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let touch_pts = window.navigator().max_touch_points();
    let inner_w = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(1280.0);
    mobile.active = touch_pts > 0 && inner_w < MOBILE_VIEWPORT_PX as f64;
    if mobile.active {
        info!(
            "Mobile mode enabled (touch_points={}, viewport={:.0})",
            touch_pts, inner_w
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn detect_mobile_mode(mut _mobile: ResMut<MobileMode>) {
    // Native builds never show on-screen sticks.
}

fn spawn_joystick_ui(mut commands: Commands, mobile: Res<MobileMode>) {
    if !mobile.active {
        return;
    }

    let knob_d = STICK_RADIUS_PX * 2.0 * KNOB_DIAMETER_FRAC;
    let knob_inset = STICK_RADIUS_PX - knob_d * 0.5;

    // Left stick — bottom-left corner
    commands
        .spawn((
            LeftStickBase,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(STICK_INSET_PX - STICK_RADIUS_PX),
                left: Val::Px(STICK_INSET_PX - STICK_RADIUS_PX),
                width: Val::Px(STICK_RADIUS_PX * 2.0),
                height: Val::Px(STICK_RADIUS_PX * 2.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderRadius::all(Val::Percent(50.0)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.28)),
            BorderColor(Color::srgba(0.7, 0.9, 1.0, 0.45)),
            ZIndex(1000),
        ))
        .with_children(|parent| {
            parent.spawn((
                LeftStickKnob,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(knob_d),
                    height: Val::Px(knob_d),
                    left: Val::Px(knob_inset),
                    top: Val::Px(knob_inset),
                    ..default()
                },
                BorderRadius::all(Val::Percent(50.0)),
                BackgroundColor(Color::srgba(0.7, 0.9, 1.0, 0.55)),
            ));
        });

    // Right stick — bottom-right corner. Tinted amber to signal "weapon".
    commands
        .spawn((
            RightStickBase,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(STICK_INSET_PX - STICK_RADIUS_PX),
                right: Val::Px(STICK_INSET_PX - STICK_RADIUS_PX),
                width: Val::Px(STICK_RADIUS_PX * 2.0),
                height: Val::Px(STICK_RADIUS_PX * 2.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderRadius::all(Val::Percent(50.0)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.28)),
            BorderColor(Color::srgba(1.0, 0.65, 0.35, 0.55)),
            ZIndex(1000),
        ))
        .with_children(|parent| {
            parent.spawn((
                RightStickKnob,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(knob_d),
                    height: Val::Px(knob_d),
                    left: Val::Px(knob_inset),
                    top: Val::Px(knob_inset),
                    ..default()
                },
                BorderRadius::all(Val::Percent(50.0)),
                BackgroundColor(Color::srgba(1.0, 0.65, 0.35, 0.65)),
            ));
        });
}

/// Read touch events and convert to JoystickState.
fn update_touch_joystick(
    mobile: Res<MobileMode>,
    touches: Res<Touches>,
    windows: Query<&Window>,
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut sticks: ResMut<TouchJoystickState>,
    mut joystick: ResMut<JoystickState>,
    mut last_state: Local<Option<GameState>>,
) {
    // Always reset confirm-button-state every frame so a stale press
    // can't linger when no gamepad is filling JoystickState.
    joystick.buttons[0] = false;

    if !mobile.active {
        return;
    }

    // On state change, drop pending tap candidates and reset stick
    // claims. Otherwise a touch held down during the previous screen
    // can fire confirm on the new screen the instant it's released.
    let current = *state.get();
    let state_changed = match *last_state {
        Some(prev) => prev != current,
        None => true,
    };
    *last_state = Some(current);
    if state_changed {
        sticks.pending_taps.clear();
        sticks.left = VirtualStick::default();
        sticks.right = VirtualStick::default();
    }
    let Ok(window) = windows.get_single() else {
        return;
    };
    // Sticks only claim screen real estate during active gameplay. In
    // menus, the entire viewport is a tap zone — no stick logic, no
    // y-coordinate gating, and the LEFT/RIGHT stick axes get zeroed.
    let gameplay = matches!(state.get(), GameState::Playing | GameState::BossFight);
    let mid_x = window.width() * 0.5;
    let stick_zone_top = if gameplay {
        window.height() * STICK_ZONE_TOP_FRAC
    } else {
        f32::INFINITY // no stick zone in menus
    };
    let now = time.elapsed_secs();

    if !gameplay {
        // Drop any stale stick claims so a finger held from gameplay
        // doesn't keep writing axes after a state transition.
        sticks.left = VirtualStick::default();
        sticks.right = VirtualStick::default();
        joystick.left_x = 0.0;
        joystick.left_y = 0.0;
        joystick.right_x = 0.0;
        joystick.right_y = 0.0;
        joystick.right_trigger = 0.0;
    }

    // Step 1 — classify newly-pressed touches: stick claim vs. tap candidate.
    for touch in touches.iter_just_pressed() {
        let pos = touch.position();
        let id = touch.id();
        if pos.y >= stick_zone_top {
            // Bottom band of the screen during gameplay — claim a stick.
            if pos.x < mid_x {
                if sticks.left.touch_id.is_none() {
                    sticks.left.touch_id = Some(id);
                    sticks.left.anchor = pos;
                    sticks.left.offset = Vec2::ZERO;
                }
            } else if sticks.right.touch_id.is_none() {
                sticks.right.touch_id = Some(id);
                sticks.right.anchor = pos;
                sticks.right.offset = Vec2::ZERO;
            }
        } else {
            // In menus this branch is hit by every touch (no stick zone).
            // In gameplay it's hit only by upper-screen taps.
            sticks.pending_taps.push(TapCandidate {
                touch_id: id,
                start_pos: pos,
                start_time: now,
            });
        }
    }

    // Step 1b — resolve pending taps: drop dragged/timed-out candidates,
    // emit a confirm-button press for ones that released within thresholds.
    let mut confirm_fired = false;
    sticks.pending_taps.retain(|c| {
        if let Some(touch) = touches.get_pressed(c.touch_id) {
            // Still held — keep if within travel + duration thresholds.
            let travel = (touch.position() - c.start_pos).length();
            let duration = now - c.start_time;
            travel < TAP_MAX_TRAVEL_PX && duration < TAP_MAX_DURATION_S
        } else {
            // Released or canceled — see if it counts as a tap.
            // We only have access to the original press position; fall
            // back to "not yet exceeded thresholds while held".
            let duration = now - c.start_time;
            if duration < TAP_MAX_DURATION_S {
                confirm_fired = true;
            }
            // Drop the candidate either way.
            false
        }
    });
    if confirm_fired {
        joystick.buttons[0] = true;
    }

    // Step 2 — update tracked-touch offsets, drop released touches.
    fn update_one(stick: &mut VirtualStick, touches: &Touches) {
        let Some(id) = stick.touch_id else {
            stick.offset = Vec2::ZERO;
            return;
        };
        if let Some(touch) = touches.get_pressed(id) {
            let mut delta = touch.position() - stick.anchor;
            if delta.length() > STICK_RADIUS_PX {
                delta = delta.normalize() * STICK_RADIUS_PX;
            }
            stick.offset = delta;
        } else {
            // Released, canceled, or never re-confirmed this frame.
            stick.touch_id = None;
            stick.offset = Vec2::ZERO;
        }
    }
    update_one(&mut sticks.left, &touches);
    update_one(&mut sticks.right, &touches);

    // Step 3 — translate to JoystickState. Bevy convention: up = +1.0,
    // but touch.position().y grows downward. Negate Y on output.
    // Skipped entirely outside gameplay so menu input doesn't write
    // ghost stick axes (we already zeroed them above).
    if gameplay {
        let to_unit = |offset: Vec2| -> Vec2 {
            if offset.length() < STICK_DEAD_PX {
                return Vec2::ZERO;
            }
            let dir = offset / STICK_RADIUS_PX;
            Vec2::new(dir.x.clamp(-1.0, 1.0), (-dir.y).clamp(-1.0, 1.0))
        };
        let left = to_unit(sticks.left.offset);
        let right = to_unit(sticks.right.offset);

        joystick.left_x = left.x;
        joystick.left_y = left.y;
        joystick.right_x = right.x;
        joystick.right_y = right.y;
        // Right stick engaged → fire. Aim direction is read from
        // (right_x, right_y) by the existing player_shooting system.
        joystick.right_trigger = if right.length() > 0.18 { 1.0 } else { 0.0 };
    }
}

/// Hide stick UI in non-gameplay states so menus get the full screen.
fn update_stick_visibility(
    state: Res<State<GameState>>,
    mut bases: Query<
        &mut Visibility,
        Or<(With<LeftStickBase>, With<RightStickBase>)>,
    >,
) {
    let gameplay = matches!(state.get(), GameState::Playing | GameState::BossFight);
    let target = if gameplay {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut vis in bases.iter_mut() {
        if *vis != target {
            *vis = target;
        }
    }
}

/// Move each stick's knob to mirror its current offset.
fn update_knob_positions(
    sticks: Res<TouchJoystickState>,
    mut left: Query<&mut Node, (With<LeftStickKnob>, Without<RightStickKnob>)>,
    mut right: Query<&mut Node, (With<RightStickKnob>, Without<LeftStickKnob>)>,
) {
    let knob_d = STICK_RADIUS_PX * 2.0 * KNOB_DIAMETER_FRAC;
    let center_inset = STICK_RADIUS_PX - knob_d * 0.5;
    if let Ok(mut node) = left.get_single_mut() {
        node.left = Val::Px(center_inset + sticks.left.offset.x);
        node.top = Val::Px(center_inset + sticks.left.offset.y);
    }
    if let Ok(mut node) = right.get_single_mut() {
        node.left = Val::Px(center_inset + sticks.right.offset.x);
        node.top = Val::Px(center_inset + sticks.right.offset.y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Touch math (pure): clamp + dead-zone + Y-axis flip.
    #[test]
    fn dead_zone_returns_zero() {
        let to_unit = |offset: Vec2| -> Vec2 {
            if offset.length() < STICK_DEAD_PX {
                return Vec2::ZERO;
            }
            let dir = offset / STICK_RADIUS_PX;
            Vec2::new(dir.x.clamp(-1.0, 1.0), (-dir.y).clamp(-1.0, 1.0))
        };
        // Inside deadzone
        assert_eq!(to_unit(Vec2::new(2.0, 2.0)), Vec2::ZERO);
        assert_eq!(to_unit(Vec2::new(-3.0, 4.0)), Vec2::ZERO);
        // Past deadzone — Y flipped
        let v = to_unit(Vec2::new(STICK_RADIUS_PX, 0.0));
        assert_eq!(v, Vec2::new(1.0, 0.0));
        let v = to_unit(Vec2::new(0.0, -STICK_RADIUS_PX));
        assert_eq!(v, Vec2::new(0.0, 1.0)); // touch up = -Y, output up = +Y
    }

    #[test]
    fn radius_clamps_diagonal() {
        // Build the same clamp the real system uses
        let mut delta = Vec2::new(STICK_RADIUS_PX * 3.0, STICK_RADIUS_PX * 3.0);
        if delta.length() > STICK_RADIUS_PX {
            delta = delta.normalize() * STICK_RADIUS_PX;
        }
        assert!((delta.length() - STICK_RADIUS_PX).abs() < 0.001);
        // Should still point in the original direction
        assert!(delta.x > 0.0 && delta.y > 0.0);
    }

    #[test]
    fn mobile_mode_default_is_inactive() {
        let m = MobileMode::default();
        assert!(!m.active);
    }
}
