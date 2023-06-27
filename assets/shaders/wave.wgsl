#import bevy_core_pipeline::fullscreen_vertex_shader
#import bevy_render::globals
#import bevy_pbr::utils

@group(0) @binding(0)
var source: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;
@group(0) @binding(2)
var<uniform> globals: Globals;

struct Wave {
    waves_x: f32,
    waves_y: f32,

    speed_x: f32,
    speed_y: f32,

    amplitude_x: f32,
    amplitude_y: f32,

    _padding: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> wave: Wave;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let pi_uv = PI * in.uv;
    let pi_time = PI * globals.time;

    let offset_x = sin((pi_uv.y * wave.waves_x) + (pi_time * wave.speed_x)) * wave.amplitude_x;
    let offset_y = sin((pi_uv.x * wave.waves_y) + (pi_time * wave.speed_y)) * wave.amplitude_y;

    let uv_displaced = vec2<f32>(in.uv.x + offset_x, in.uv.y + offset_y);

    return textureSample(source, source_sampler, uv_displaced);
}
