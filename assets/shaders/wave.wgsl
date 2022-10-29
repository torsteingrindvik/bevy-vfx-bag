#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

struct Wave {
    waves_x: f32,
    waves_y: f32,

    speed_x: f32,
    speed_y: f32,

    amplitude_x: f32,
    amplitude_y: f32
};

@group(1) @binding(2)
var<uniform> wave: Wave;

fn fragment_impl(
    position: vec4<f32>,
    uv: vec2<f32>
) -> vec4<f32> {
    let pi_uv = PI * uv;
    let pi_time = PI * globals.time;

    let offset_x = sin((pi_uv.y * wave.waves_x) + (pi_time * wave.speed_x)) * wave.amplitude_x;
    let offset_y = sin((pi_uv.x * wave.waves_y) + (pi_time * wave.speed_y)) * wave.amplitude_y;

    let uv_displaced = vec2<f32>(uv.x + offset_x, uv.y + offset_y);

    return vec4<f32>(textureSample(texture, our_sampler, uv_displaced).rgb, 1.0);

}

#import bevy_vfx_bag::post_processing_passthrough
