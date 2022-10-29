#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

struct Pixelate {
    block_size: f32,
};
@group(1) @binding(2)
var<uniform> pixelate: Pixelate;

fn fragment_impl(
    position: vec4<f32>,
    uv: vec2<f32>
) -> vec4<f32> {
    var uv = uv + 0.5;

    let width_height_over_block_size = view.viewport.zw / max(1.0, pixelate.block_size);

    uv *= width_height_over_block_size;
    uv = floor(uv);

    uv /= width_height_over_block_size;

    uv -= 0.5;

    return vec4<f32>(textureSample(texture, our_sampler, uv).rgb, 1.0);
}

#import bevy_vfx_bag::post_processing_passthrough
