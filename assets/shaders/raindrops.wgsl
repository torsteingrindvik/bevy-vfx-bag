#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::utils


@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

@group(1) @binding(2)
var texture_raindrops: texture_2d<f32>;

@group(1) @binding(3)
var sampler_raindrops: sampler;

struct Raindrops {
    hmm: f32
};
@group(1) @binding(4)
var<uniform> raindrops: Raindrops;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv = coords_to_viewport_uv(position.xy, view.viewport);

    // return vec4<f32>(textureSample(texture, our_sampler, uv).rgb, 1.0);

    // return vec4<f32>(textureSample(texture_raindrops, sampler_raindrops, uv).rgba);

    return vec4<f32>(uv, 0.0, 1.0);
}
