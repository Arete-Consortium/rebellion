// Combat Wheel — Capacitor Core Fragment Shader
// Bevy 0.15 2D material pipeline. Minimal v1 shader.

#import bevy_sprite::mesh2d_vertex_output

@group(1) @binding(0) var<uniform> energy: f32;
@group(1) @binding(0) var<uniform> time: f32;
@group(1) @binding(0) var<uniform> pad0: f32;
@group(1) @binding(0) var<uniform> pad1: f32;
@group(1) @binding(1) var<uniform> tint: vec4<f32>;

@fragment
fn fragment(mesh: mesh2d_vertex_output) -> @location(0) vec4<f32> {
    // Bright core with soft pulse. Energy controls overall brightness.
    let pulse = 0.15 * (0.5 + 0.5 * sin(time * 3.0));
    let color = tint.rgb * (energy * 0.9 + 0.1) + pulse;
    return vec4<f32>(color, tint.a);
}