#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::utils


@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {

    // Uv is 
    //  (0.0, 0.0)
    // to:
    //  (1.0, 1.0)


    let uv_original = coords_to_viewport_uv(position.xy, view.viewport);
    // uv.x = 0.0 - uv.x;
    // uv.x *= 1.0;
    let uv = vec2<f32>(1.0 - uv_original.x, uv_original.y);
    let uv = vec2<f32>(uv_original.x, uv_original.y);

    var output_color = vec4<f32>(textureSample(texture, our_sampler, uv).rgb, 1.0);

    // output_color.r = 1.0;

    return output_color;
}