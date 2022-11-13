#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

struct Tint {
    color: vec4<f32>,
};
@group(1) @binding(2)
var<uniform> tint: Tint;

fn fragment_impl(
    position: vec4<f32>,
    uv: vec2<f32>
) -> vec4<f32> {
    return vec4<f32>(textureSample(texture, our_sampler, uv).rgb * tint.color.rgb, 1.0);
}

#import bevy_vfx_bag::post_processing_passthrough
