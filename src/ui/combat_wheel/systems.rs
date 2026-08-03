use bevy::prelude::*;
use bevy::sprite::MeshMaterial2d;

use super::components::*;
use super::events::CombatWheelEvent;
use super::materials::*;
use super::resources::*;

/// Emits CombatWheelEvents based on adapter resource changes.
///
/// Mirrors the prototype's `emit_combat_wheel_events` but reads from the
/// unified `CombatWheelAdapter` resource instead of four separate
/// shield/integrity/capacitor/heat resources.
#[allow(clippy::too_many_arguments)]
pub fn emit_combat_wheel_events(
    adapter: Res<CombatWheelAdapter>,
    mut events: EventWriter<CombatWheelEvent>,
    mut local_shield: Local<Option<f32>>,
    mut local_heat_crit: Local<bool>,
    mut local_heat_warn: Local<bool>,
    mut local_integrity_crit: Local<bool>,
    mut local_insufficient_cap: Local<bool>,
) {
    let prev_shield = local_shield.unwrap_or(adapter.shield_current);
    if adapter.shield_current < prev_shield {
        if let Some(dir) = adapter.last_damage_direction {
            events.send(CombatWheelEvent::ShieldDamaged {
                amount: prev_shield - adapter.shield_current,
                direction: dir,
            });
        }
    }
    if adapter.shield_collapsed && prev_shield > 0.0 {
        events.send(CombatWheelEvent::ShieldCollapsed);
    }
    if !adapter.shield_collapsed
        && adapter.shield_current >= adapter.shield_max
        && prev_shield < adapter.shield_max
    {
        events.send(CombatWheelEvent::ShieldRecharged);
    }
    *local_shield = Some(adapter.shield_current);

    let heat_crit_now = adapter.heat_current >= adapter.heat_critical_threshold;
    if heat_crit_now && !*local_heat_crit {
        events.send(CombatWheelEvent::HeatCritical);
    }
    *local_heat_crit = heat_crit_now;

    let heat_warn_now = adapter.heat_current >= adapter.heat_warning_threshold
        && adapter.heat_current < adapter.heat_critical_threshold;
    if heat_warn_now && !*local_heat_warn {
        events.send(CombatWheelEvent::HeatWarning);
    }
    *local_heat_warn = heat_warn_now;

    let integrity_crit_now =
        adapter.hull_current < adapter.hull_max * 0.3 && adapter.hull_current > 0.0;
    if integrity_crit_now && !*local_integrity_crit {
        events.send(CombatWheelEvent::IntegrityCritical);
    }
    *local_integrity_crit = integrity_crit_now;

    let insufficient_now = adapter.capacitor_current < adapter.capacitor_max * 0.2;
    if insufficient_now && !*local_insufficient_cap {
        events.send(CombatWheelEvent::InsufficientCapacitor {
            attempted_action: ModuleSlotId::PrimaryWeapon,
        });
    }
    *local_insufficient_cap = insufficient_now;
}

/// Reads ShieldDamaged events and inserts a ShieldSurge component on the
/// ring root.
pub fn trigger_shield_surge(
    mut events: EventReader<CombatWheelEvent>,
    mut commands: Commands,
    ring_query: Query<Entity, With<ShieldRing>>,
) {
    for event in events.read() {
        if let CombatWheelEvent::ShieldDamaged { direction, .. } = event {
            if let Ok(ring) = ring_query.get_single() {
                let angle = direction.y.atan2(direction.x);
                commands.entity(ring).insert(ShieldSurge {
                    origin_angle: angle,
                    intensity: 0.8,
                    decay: 2.0,
                });
            }
        }
    }
}

/// Converts a Bevy Color to a shader Vec4 tint (linear RGB).
fn tint_from_color(c: Color) -> Vec4 {
    let l = c.to_linear();
    Vec4::new(l.red, l.green, l.blue, 1.0)
}

/// Binds the Combat Wheel adapter's shield fields to ShieldRing segment
/// visuals via shader uniforms.
pub fn bind_shield_to_wheel(
    adapter: Res<CombatWheelAdapter>,
    faction: Res<FactionSkin>,
    mut segments: Query<(
        &mut ShieldSegment,
        &mut MeshMaterial2d<ShieldMaterial>,
        &mut Visibility,
    )>,
    mut materials: ResMut<Assets<ShieldMaterial>>,
    time: Res<Time>,
) {
    let total = segments.iter().count().max(1);
    let health_per_segment = adapter.shield_max / total as f32;
    let current_global = adapter.shield_current;
    let tint = tint_from_color(faction.palette().shield_base);
    let t = time.elapsed_secs();

    for (mut seg, mat_handle, mut vis) in segments.iter_mut() {
        let seg_start = seg.index as f32 * health_per_segment;
        let seg_end = (seg.index + 1) as f32 * health_per_segment;

        let new_health = if current_global >= seg_end {
            1.0
        } else if current_global <= seg_start {
            0.0
        } else {
            (current_global - seg_start) / health_per_segment
        };

        if new_health <= 0.0 {
            seg.state = SegmentState::Collapsed;
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Visible;
            seg.state = if new_health < 1.0 {
                SegmentState::Damaged
            } else {
                SegmentState::Healthy
            };
        }

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.health = new_health;
            mat.time = t;
            mat.tint = tint;
        }

        seg.health = new_health;
    }
}

/// Binds the adapter's hull/armor fields to IntegrityRing segment visuals.
pub fn bind_integrity_to_wheel(
    adapter: Res<CombatWheelAdapter>,
    faction: Res<FactionSkin>,
    mut segments: Query<(
        &mut IntegritySegment,
        &mut MeshMaterial2d<IntegrityMaterial>,
        &mut Visibility,
    )>,
    mut materials: ResMut<Assets<IntegrityMaterial>>,
    time: Res<Time>,
) {
    let total = segments.iter().count().max(1);
    let health_per_seg = adapter.hull_max / total as f32;
    let armor_per_seg = adapter.armor_max / total as f32;
    let tint = tint_from_color(faction.palette().integrity_pristine);
    let t = time.elapsed_secs();
    let repair_active = if adapter.repair_active { 1.0 } else { 0.0 };

    for (mut seg, mat_handle, mut vis) in segments.iter_mut() {
        let seg_start_h = seg.index as f32 * health_per_seg;
        let seg_end_h = (seg.index + 1) as f32 * health_per_seg;
        let seg_start_a = seg.index as f32 * armor_per_seg;
        let seg_end_a = (seg.index + 1) as f32 * armor_per_seg;

        let new_health = if adapter.hull_current >= seg_end_h {
            1.0
        } else if adapter.hull_current <= seg_start_h {
            0.0
        } else {
            (adapter.hull_current - seg_start_h) / health_per_seg
        };

        let new_armor = if adapter.armor_current >= seg_end_a {
            1.0
        } else if adapter.armor_current <= seg_start_a {
            0.0
        } else {
            (adapter.armor_current - seg_start_a) / armor_per_seg
        };

        seg.health = new_health;
        seg.armor = new_armor;

        if new_health <= 0.0 {
            seg.state = IntegrityVisual::Destroyed;
            *vis = Visibility::Hidden;
            continue;
        }

        *vis = Visibility::Visible;

        seg.state = if new_armor >= 0.8 {
            IntegrityVisual::Pristine
        } else if new_armor >= 0.3 {
            IntegrityVisual::Fractured
        } else if new_armor > 0.0 {
            IntegrityVisual::Breached
        } else if new_health < 0.3 {
            IntegrityVisual::Critical
        } else {
            IntegrityVisual::Breached
        };

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.health = new_health;
            mat.armor = new_armor;
            mat.time = t;
            mat.repair_active = repair_active;
            mat.tint = tint;
        }
    }
}

/// Binds adapter's capacitor fields to CapacitorCore shader uniforms.
pub fn bind_capacitor_to_wheel(
    adapter: Res<CombatWheelAdapter>,
    faction: Res<FactionSkin>,
    mut query: Query<&mut MeshMaterial2d<CapacitorMaterial>, With<CapacitorCore>>,
    mut materials: ResMut<Assets<CapacitorMaterial>>,
    time: Res<Time>,
) {
    let tint = tint_from_color(faction.palette().capacitor_bright);
    let t = time.elapsed_secs();
    let energy = adapter.capacitor_current / adapter.capacitor_max.max(1.0);

    for mat_handle in query.iter_mut() {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.energy = energy;
            mat.time = t;
            mat.tint = tint;
        }
    }
}

/// Binds adapter's heat fields to HeatArc segment visuals.
pub fn bind_heat_to_wheel(
    adapter: Res<CombatWheelAdapter>,
    faction: Res<FactionSkin>,
    mut segments: Query<(&mut Visibility, &mut MeshMaterial2d<HeatMaterial>), With<HeatSegment>>,
    mut materials: ResMut<Assets<HeatMaterial>>,
    time: Res<Time>,
) {
    let heat_norm = adapter.heat_current / adapter.heat_maximum.max(1.0);
    let warn = adapter.heat_warning_threshold / adapter.heat_maximum.max(1.0);
    let crit = adapter.heat_critical_threshold / adapter.heat_maximum.max(1.0);
    let tint = tint_from_color(faction.palette().heat_safe);
    let t = time.elapsed_secs();

    for (mut vis, mat_handle) in segments.iter_mut() {
        if heat_norm > 0.01 {
            *vis = Visibility::Visible;
            if let Some(mat) = materials.get_mut(&mat_handle.0) {
                mat.heat_norm = heat_norm;
                mat.time = t;
                mat.warning_threshold = warn;
                mat.critical_threshold = crit;
                mat.tint = tint;
            }
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

/// Default mapping of slot visuals — module runtime state integration is
/// handled by the triglavian_invasion `combat_wheel_bind` submodule; for
/// now every slot just shows as Ready. The slot tint is derived from the
/// active `FactionSkin::palette().shield_base` so EDENCOM slots read blue,
/// Triglavian slots read orange-red, Minmatar slots read arc-blue, etc.
pub fn bind_modules_to_wheel(
    faction: Res<FactionSkin>,
    mut slots: Query<(&mut ModuleSlot, &mut MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let base = faction.palette().shield_base;
    for (mut slot, mat_handle) in slots.iter_mut() {
        slot.visual_state = ModuleVisualState::Ready;
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = base;
        }
    }
}

/// Animates shield surge fading over time by updating per-segment shader
/// uniforms.
pub fn animate_shield_surge(
    mut commands: Commands,
    mut surges: Query<(Entity, &mut ShieldSurge, &Children)>,
    mut segments: Query<(&ShieldSegment, &mut MeshMaterial2d<ShieldMaterial>)>,
    mut materials: ResMut<Assets<ShieldMaterial>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for (entity, mut surge, children) in surges.iter_mut() {
        surge.intensity -= surge.decay * time.delta_secs();
        if surge.intensity <= 0.0 {
            commands.entity(entity).remove::<ShieldSurge>();
            continue;
        }

        for &child in children.iter() {
            if let Ok((seg, mat_handle)) = segments.get_mut(child) {
                let seg_center = (seg.index as f32 + 0.5) / 48.0 * std::f32::consts::TAU;
                let mut angle_diff = (seg_center - surge.origin_angle).abs();
                if angle_diff > std::f32::consts::PI {
                    angle_diff = std::f32::consts::TAU - angle_diff;
                }
                let falloff = (-angle_diff.powi(2) / 0.05).exp();
                let boost = surge.intensity * falloff;

                if let Some(mat) = materials.get_mut(&mat_handle.0) {
                    mat.surge_intensity = boost;
                    mat.time = t;
                }
            }
        }
    }
}

/// Subtle capacitor center pulse via CapacitorMaterial time uniform.
pub fn animate_capacitor_pulse(
    mut query: Query<&mut MeshMaterial2d<CapacitorMaterial>, With<CapacitorCore>>,
    mut materials: ResMut<Assets<CapacitorMaterial>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for mat_handle in query.iter_mut() {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.time = t;
        }
    }
}

/// Heat arc ambient glow pulse via HeatMaterial time uniform.
pub fn animate_heat_glow(
    mut segments: Query<&mut MeshMaterial2d<HeatMaterial>, With<HeatSegment>>,
    mut materials: ResMut<Assets<HeatMaterial>>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();
    for mat_handle in segments.iter_mut() {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.time = t;
        }
    }
}

/// Updates module input glyphs based on active device.
pub fn update_input_glyphs(
    device: Res<ActiveInputDevice>,
    mut glyphs: Query<(&ModuleSlot, &mut ModuleInputGlyph)>,
) {
    for (slot, mut glyph) in glyphs.iter_mut() {
        let binding = if *device == ActiveInputDevice::Keyboard {
            match slot.slot_id {
                ModuleSlotId::PrimaryWeapon => InputBinding::Keyboard(KeyCode::Digit1),
                ModuleSlotId::SecondaryWeapon => InputBinding::Keyboard(KeyCode::Digit2),
                ModuleSlotId::Propulsion => InputBinding::Keyboard(KeyCode::Digit3),
                ModuleSlotId::Defense => InputBinding::Keyboard(KeyCode::Digit4),
                ModuleSlotId::Ability => InputBinding::Keyboard(KeyCode::Digit5),
                ModuleSlotId::Deployable => InputBinding::Keyboard(KeyCode::Digit6),
            }
        } else {
            InputBinding::Unbound
        };
        glyph.binding = binding;
    }
}

/// Detects FactionSkin changes and updates all existing material asset
/// tints (custom shaders + module slot `ColorMaterial`).
pub fn update_faction_materials(
    faction: Res<FactionSkin>,
    mut shield_mats: ResMut<Assets<ShieldMaterial>>,
    mut integrity_mats: ResMut<Assets<IntegrityMaterial>>,
    mut heat_mats: ResMut<Assets<HeatMaterial>>,
    mut cap_mats: ResMut<Assets<CapacitorMaterial>>,
    mut slot_mats: ResMut<Assets<ColorMaterial>>,
) {
    if !faction.is_changed() {
        return;
    }

    let palette = faction.palette();
    let shield_tint = tint_from_color(palette.shield_base);
    let integrity_tint = tint_from_color(palette.integrity_pristine);
    let heat_tint = tint_from_color(palette.heat_safe);
    let cap_tint = tint_from_color(palette.capacitor_bright);
    let slot_color = palette.shield_base;

    for (_, mat) in shield_mats.iter_mut() {
        mat.tint = shield_tint;
    }
    for (_, mat) in integrity_mats.iter_mut() {
        mat.tint = integrity_tint;
    }
    for (_, mat) in heat_mats.iter_mut() {
        mat.tint = heat_tint;
    }
    for (_, mat) in cap_mats.iter_mut() {
        mat.tint = cap_tint;
    }
    for (_, mat) in slot_mats.iter_mut() {
        mat.color = slot_color;
    }
}

/// Updates numeric percentage labels and toggles visibility based on HUD
/// settings.
pub fn update_percentage_labels(
    adapter: Res<CombatWheelAdapter>,
    settings: Res<HudSettings>,
    mut labels: Query<(&PercentageLabel, &mut Text2d, &mut Visibility)>,
) {
    let show = settings.show_numeric_percentages;

    for (label, mut text, mut vis) in labels.iter_mut() {
        *vis = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if !show {
            continue;
        }

        let pct = match label.stat {
            StatType::Shield => {
                (adapter.shield_current / adapter.shield_max.max(1.0) * 100.0).round() as i32
            }
            StatType::Integrity => {
                (adapter.hull_current / adapter.hull_max.max(1.0) * 100.0).round() as i32
            }
            StatType::Capacitor => {
                (adapter.capacitor_current / adapter.capacitor_max.max(1.0) * 100.0).round() as i32
            }
            StatType::Heat => {
                (adapter.heat_current / adapter.heat_maximum.max(1.0) * 100.0).round() as i32
            }
        };

        let prefix = match label.stat {
            StatType::Shield => "SHD",
            StatType::Integrity => "INT",
            StatType::Capacitor => "CAP",
            StatType::Heat => "HEAT",
        };

        // Bevy 0.15: Text2d holds an inner Text. Mutate the section value
        // through DerefMut to Text.
        let new_value = format!("{}: {}%", prefix, pct);
        **text = new_value;
    }
}

/// Cycles FactionSkin with the F key for palette testing.
pub fn cycle_faction_skin(keys: Res<ButtonInput<KeyCode>>, mut skin: ResMut<FactionSkin>) {
    if keys.just_pressed(KeyCode::KeyF) {
        *skin = skin.next();
    }
}

/// Placeholder — no shader-based accessibility in v1.
pub fn update_accessibility(_settings: Res<AccessibilitySettings>) {
    // TODO: implement high-contrast mode by recompiling shader uniforms
    // or using a post-processing overlay for the entire HUD.
}
