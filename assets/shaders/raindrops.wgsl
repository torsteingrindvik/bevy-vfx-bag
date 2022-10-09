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

// Remap the r, g, a channels to (-1, 1) range.
fn rga_normalize(
    rga: vec3<f32>
) -> vec3<f32> {
    return (rga * 2.) - 1.; 
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv_orig = coords_to_viewport_uv(position.xy, view.viewport);

    // Raindrops texture wraps.
    // Make aspect-ratio independent UV coords.
    let uv = vec2<f32>(uv_orig.x * view.viewport.z / view.viewport.w, uv_orig.y);

    let uv = uv * 1.8;

    let t = textureSample(texture_raindrops, sampler_raindrops, uv).rgba;
    let rga_norm = rga_normalize(t.rga);

    return vec4<f32>(vec3<f32>(rga_norm.b), 1.0);
}
