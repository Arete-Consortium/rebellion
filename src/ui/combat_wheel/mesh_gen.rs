use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::TAU;

/// Generates a flat arc mesh in the XY plane, centered at origin.
///
/// `inner_radius` and `outer_radius` define the ring thickness.
/// `start_angle` and `end_angle` are in radians
/// (0 = right, TAU/4 = down in screen-space).
/// `segments` controls vertex density. 8-16 is enough for a UI arc.
///
/// Bevy 0.15: `RenderAssetUsages::default()` was removed in favor of explicit
/// `RENDER_WORLD` (assets are uploaded to the render world but not the main
/// world). The Combat Wheel meshes are render-only.
pub fn arc_mesh(
    inner_radius: f32,
    outer_radius: f32,
    start_angle: f32,
    end_angle: f32,
    segments: usize,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((segments + 1) * 2);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity((segments + 1) * 2);
    let mut indices: Vec<u32> = Vec::with_capacity(segments * 6);

    let step = (end_angle - start_angle) / segments as f32;

    for i in 0..=segments {
        let angle = start_angle + step * i as f32;
        let cos = angle.cos();
        let sin = angle.sin();

        positions.push([inner_radius * cos, inner_radius * sin, 0.0]);
        uvs.push([0.0, i as f32 / segments as f32]);

        positions.push([outer_radius * cos, outer_radius * sin, 0.0]);
        uvs.push([1.0, i as f32 / segments as f32]);
    }

    for i in 0..segments {
        let base = (i * 2) as u32;
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 1, base + 3, base + 2]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Generate a complete ring split into `segment_count` discrete mesh entities.
/// Returns a Vec of `(start_angle, end_angle)` for each segment.
pub fn ring_segment_angles(segment_count: usize) -> Vec<(f32, f32)> {
    let step = TAU / segment_count as f32;
    (0..segment_count)
        .map(|i| {
            let start = i as f32 * step;
            let end = (i + 1) as f32 * step;
            (start, end)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arc_mesh_vertex_count() {
        let mesh = arc_mesh(10.0, 20.0, 0.0, TAU, 8);
        let positions: Vec<[f32; 3]> = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .to_vec();
        assert_eq!(positions.len(), 18); // (segments + 1) * 2
    }

    #[test]
    fn arc_mesh_index_count() {
        let mesh = arc_mesh(10.0, 20.0, 0.0, TAU, 4);
        let indices = mesh.indices().unwrap();
        assert_eq!(indices.len(), 24); // segments * 6
    }

    #[test]
    fn ring_segment_angles_count() {
        let angles = ring_segment_angles(48);
        assert_eq!(angles.len(), 48);
    }

    #[test]
    fn ring_segment_angles_sum_to_tau() {
        let angles = ring_segment_angles(16);
        let first = angles.first().unwrap().0;
        let last = angles.last().unwrap().1;
        assert!((last - first - TAU).abs() < 1e-6);
    }
}
