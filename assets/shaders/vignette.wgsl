#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::utils


@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

struct Vignette {
    radius: f32,
    feathering: f32,
    color: vec4<f32>,
};
@group(1) @binding(2)
var<uniform> vignette: Vignette;


@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    // (0.0 -> 1.0)
    var uv = coords_to_viewport_uv(position.xy, view.viewport);

    var scene = textureSample(texture, our_sampler, uv);

    // (-1.0 -> 1.0)
    uv = uv * 2. - 1.;

    var circle = length(uv);
    var mask = smoothstep(vignette.radius + vignette.feathering, vignette.radius, circle);

    var output = scene * mask;
    
    return vec4<f32>(output.rgb, 1.0);
}