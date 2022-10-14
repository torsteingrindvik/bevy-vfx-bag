#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

struct ChromaticAberration {
    dir_r: vec2<f32>,
    magnitude_r: f32,

    dir_g: vec2<f32>,
    magnitude_g: f32,

    dir_b: vec2<f32>,
    magnitude_b: f32,
};

@group(1) @binding(2)
var<uniform> chromatic_aberration: ChromaticAberration;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv = coords_to_viewport_uv(position.xy, view.viewport);

    let out = vec3<f32>(
        textureSample(texture, our_sampler, uv + (chromatic_aberration.dir_r * chromatic_aberration.magnitude_r)).r,
        textureSample(texture, our_sampler, uv + (chromatic_aberration.dir_g * chromatic_aberration.magnitude_g)).g,
        textureSample(texture, our_sampler, uv + (chromatic_aberration.dir_b * chromatic_aberration.magnitude_b)).b,
    );

    return vec4<f32>(out, 1.0);
}
