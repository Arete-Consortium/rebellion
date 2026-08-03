// Combat Wheel — Heat Arc Fragment Shader
// Bevy 0.15 2D material pipeline. Minimal v1 shader.

#import bevy_sprite::mesh2d_vertex_output

@group(1) @binding(0) var<uniform> heat_norm: f32;
@group(1) @binding(0) var<uniform> time: f32;
@group(1) @binding(0) var<uniform> warning_threshold: f32;
@group(1) @binding(0) var<uniform> critical_threshold: f32;
@group(1) @binding(1) var<uniform> tint: vec4<f32>;

@fragment
fn fragment(mesh: mesh2d_vertex_output) -> @location(0) vec4<f32> {
    // Subtle pulse scaled by heat. Critical adds red bleed.
    let pulse = 0.10 * heat_norm * (0.5 + 0.5 * sin(time * 6.0));
    let crit_bleed = step(critical_threshold, heat_norm) * 0.3;
    let color = tint.rgb + pulse + vec3<f32>(crit_bleed, 0.0, 0.0);
    return vec4<f32>(color, tint.a * heat_norm);
}