// Combat Wheel — Shield Ring Fragment Shader
// Bevy 0.15 2D material pipeline. Minimal v1 shader; visual sophistication
// (electric arcs, surge shader) lands in a follow-up.

#import bevy_sprite::mesh2d_vertex_output

@group(1) @binding(0) var<uniform> health: f32;
@group(1) @binding(0) var<uniform> surge_intensity: f32;
@group(1) @binding(0) var<uniform> time: f32;
@group(1) @binding(0) var<uniform> pad: f32;
@group(1) @binding(1) var<uniform> tint: vec4<f32>;

@fragment
fn fragment(mesh: mesh2d_vertex_output) -> @location(0) vec4<f32> {
    // Healthy shield = bright tint; depleted shield = dark tint. Surge adds
    // a brief white flash proportional to intensity.
    let dim = mix(0.15, 1.0, health);
    let surge = vec4<f32>(surge_intensity, surge_intensity, surge_intensity, 0.0);
    let color = tint.rgb * dim + surge.rgb * 0.4;
    return vec4<f32>(color, tint.a);
}