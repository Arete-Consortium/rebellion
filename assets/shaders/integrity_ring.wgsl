// Combat Wheel — Integrity Ring Fragment Shader
// Bevy 0.15 2D material pipeline. Minimal v1 shader.

#import bevy_sprite::mesh2d_vertex_output

@group(1) @binding(0) var<uniform> health: f32;
@group(1) @binding(0) var<uniform> armor: f32;
@group(1) @binding(0) var<uniform> time: f32;
@group(1) @binding(0) var<uniform> repair_active: f32;
@group(1) @binding(1) var<uniform> tint: vec4<f32>;

@fragment
fn fragment(mesh: mesh2d_vertex_output) -> @location(0) vec4<f32> {
    // Blend between pristine (armor > 0.8) and breached (armor < 0.3).
    let dim = mix(0.20, 1.0, armor);
    let pulse = 0.05 * repair_active * sin(time * 4.0);
    let color = tint.rgb * dim + pulse;
    return vec4<f32>(color, tint.a);
}