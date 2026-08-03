use bevy::prelude::*;
use bevy::render::mesh::Mesh2d;
use bevy::sprite::MeshMaterial2d;
use std::f32::consts::TAU;

use super::*;

/// Spawns the wheel root entity. The actual position is overwritten at
/// runtime by `resolve_layout` based on window size.
pub fn spawn_combat_wheel(mut commands: Commands) {
    commands.spawn((
        CombatWheel,
        Transform::from_xyz(0.0, -120.0, 100.0),
        Visibility::default(),
    ));
}

pub fn spawn_shield_ring(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShieldMaterial>>,
    wheel_query: Query<Entity, With<CombatWheel>>,
) {
    let Ok(wheel) = wheel_query.get_single() else {
        return;
    };

    let segment_count = 48;
    let inner_radius = 90.0;
    let outer_radius = 110.0;
    let segment_angles = ring_segment_angles(segment_count);

    let ring_root = commands
        .spawn((ShieldRing, Transform::default(), Visibility::default()))
        .id();

    for (idx, (start, end)) in segment_angles.iter().enumerate() {
        let mesh = arc_mesh(inner_radius, outer_radius, *start, *end, 4);
        let material = materials.add(default_shield_material());
        let mesh_handle: Handle<Mesh> = meshes.add(mesh);
        let segment_entity = commands
            .spawn((
                ShieldSegment {
                    index: idx,
                    health: 1.0,
                    max_health: 1.0,
                    ..default()
                },
                Mesh2d(mesh_handle),
                MeshMaterial2d(material),
                Transform::from_xyz(0.0, 0.0, 0.0),
                Visibility::default(),
            ))
            .id();

        commands.entity(ring_root).add_child(segment_entity);
    }

    commands.entity(wheel).add_child(ring_root);
}

pub fn spawn_integrity_ring(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<IntegrityMaterial>>,
    wheel_query: Query<Entity, With<CombatWheel>>,
) {
    let Ok(wheel) = wheel_query.get_single() else {
        return;
    };

    let segment_count = 48;
    let inner_radius = 60.0;
    let outer_radius = 85.0;
    let angles = ring_segment_angles(segment_count);

    let ring_root = commands
        .spawn((IntegrityRing, Transform::default(), Visibility::default()))
        .id();

    for (idx, (start, end)) in angles.iter().enumerate() {
        let mesh = arc_mesh(inner_radius, outer_radius, *start, *end, 4);
        let material = materials.add(default_integrity_material());
        let mesh_handle: Handle<Mesh> = meshes.add(mesh);
        let entity = commands
            .spawn((
                IntegritySegment {
                    index: idx,
                    health: 1.0,
                    armor: 1.0,
                    max_health: 1.0,
                    max_armor: 1.0,
                    ..default()
                },
                Mesh2d(mesh_handle),
                MeshMaterial2d(material),
                Transform::default(),
                Visibility::default(),
            ))
            .id();

        commands.entity(ring_root).add_child(entity);
    }

    commands.entity(wheel).add_child(ring_root);
}

pub fn spawn_capacitor_core(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CapacitorMaterial>>,
    wheel_query: Query<Entity, With<CombatWheel>>,
) {
    let Ok(wheel) = wheel_query.get_single() else {
        return;
    };

    let mesh = arc_mesh(0.0, 45.0, 0.0, TAU, 32);
    let material = materials.add(default_capacitor_material());
    let mesh_handle: Handle<Mesh> = meshes.add(mesh);

    let core = commands
        .spawn((
            CapacitorCore,
            Mesh2d(mesh_handle),
            MeshMaterial2d(material),
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    commands.entity(wheel).add_child(core);
}

pub fn spawn_heat_arc(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<HeatMaterial>>,
    wheel_query: Query<Entity, With<CombatWheel>>,
) {
    let Ok(wheel) = wheel_query.get_single() else {
        return;
    };

    let segment_count = 16;
    let inner_radius = 115.0;
    let outer_radius = 125.0;
    let angles = ring_segment_angles(segment_count);

    let mid = segment_count / 2;
    let start_idx = mid / 2;
    let end_idx = segment_count - start_idx;

    let arc_root = commands
        .spawn((HeatArc, Transform::default(), Visibility::default()))
        .id();

    let material = materials.add(default_heat_material());

    for (idx, (start, end)) in angles.iter().enumerate() {
        if idx < start_idx || idx >= end_idx {
            continue;
        }
        let mesh = arc_mesh(inner_radius, outer_radius, *start, *end, 4);
        let mesh_handle: Handle<Mesh> = meshes.add(mesh);
        let entity = commands
            .spawn((
                HeatSegment { index: idx },
                Mesh2d(mesh_handle),
                MeshMaterial2d(material.clone()),
                Transform::default(),
                Visibility::default(),
            ))
            .id();

        commands.entity(arc_root).add_child(entity);
    }

    commands.entity(wheel).add_child(arc_root);
}

pub fn spawn_percentage_labels(
    mut commands: Commands,
    wheel_query: Query<Entity, With<CombatWheel>>,
) {
    let Ok(wheel) = wheel_query.get_single() else {
        return;
    };

    let labels = [
        (StatType::Shield, "SHD: 100%", Vec3::new(0.0, 125.0, 3.0)),
        (StatType::Integrity, "INT: 100%", Vec3::new(0.0, -95.0, 3.0)),
        (StatType::Capacitor, "CAP: 100%", Vec3::new(0.0, 0.0, 4.0)),
        (StatType::Heat, "HEAT: 0%", Vec3::new(135.0, 0.0, 3.0)),
    ];

    for (stat, initial, offset) in labels {
        let label = commands
            .spawn((
                PercentageLabel { stat },
                Text2d::new(initial),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                Transform::from_translation(offset),
                Visibility::Hidden,
            ))
            .id();
        commands.entity(wheel).add_child(label);
    }
}

pub fn spawn_module_slots(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    wheel_query: Query<Entity, With<CombatWheel>>,
) {
    let Ok(wheel) = wheel_query.get_single() else {
        return;
    };

    // Drop Deployable — Triglavian ships have no deployable capability.
    // Initial angles are overwritten by `resolve_layout` each frame; these
    // values are placeholders so the entity has a position before the layout
    // system runs (avoids a one-frame flicker at z=0).
    let slots = [
        (
            ModuleSlotId::PrimaryWeapon,
            ModuleCategory::PrimaryWeapon,
            Vec3::new(0.0, 140.0, 101.0),
        ),
        (
            ModuleSlotId::SecondaryWeapon,
            ModuleCategory::SecondaryWeapon,
            Vec3::new(120.0, 70.0, 101.0),
        ),
        (
            ModuleSlotId::Propulsion,
            ModuleCategory::Propulsion,
            Vec3::new(120.0, -70.0, 101.0),
        ),
        (
            ModuleSlotId::Defense,
            ModuleCategory::Defense,
            Vec3::new(0.0, -140.0, 101.0),
        ),
        (
            ModuleSlotId::Ability,
            ModuleCategory::Tactical,
            Vec3::new(-120.0, -70.0, 101.0),
        ),
    ];

    let mesh = arc_mesh(0.0, 18.0, 0.0, TAU, 16);
    let material = materials.add(ColorMaterial::from(Color::srgb(0.3, 0.3, 0.3)));
    let mesh_handle: Handle<Mesh> = meshes.add(mesh);

    for (slot_id, category, offset) in slots {
        let slot = commands
            .spawn((
                ModuleSlot {
                    slot_id,
                    category,
                    ..default()
                },
                ModuleInputGlyph {
                    binding: InputBinding::Unbound,
                },
                Mesh2d(mesh_handle.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_translation(offset),
                Visibility::default(),
            ))
            .id();

        commands.entity(wheel).add_child(slot);
    }
}

/// Despawn all entities tagged with `CombatWheel` and their children.
#[allow(dead_code)] // Wired in Phase 2 (campaign gating)
pub fn despawn_combat_wheel(mut commands: Commands, query: Query<Entity, With<CombatWheel>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
