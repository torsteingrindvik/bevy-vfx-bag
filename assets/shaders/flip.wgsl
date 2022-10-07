#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::utils


@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

struct FlipUniform {
    x: f32,
    y: f32,
};
@group(1) @binding(2)
var<uniform> flip: FlipUniform;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv_unaltered = coords_to_viewport_uv(position.xy, view.viewport);
    let uv = abs(vec2<f32>(flip.x, flip.y) - uv_unaltered);

    return vec4<f32>(textureSample(texture, our_sampler, uv).rgb, 1.0);
}
