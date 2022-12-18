#import bevy_core_pipeline::fullscreen_vertex_shader
#import bevy_pbr::mesh_view_types

@group(0) @binding(0)
var source: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;
@group(0) @binding(2)
var<uniform> globals: Globals;

struct ChromaticAberration {
    dir_r: vec2<f32>,
    magnitude_r: f32,

    dir_g: vec2<f32>,
    magnitude_g: f32,

    dir_b: vec2<f32>,
    magnitude_b: f32,
};

@group(0) @binding(3)
var<uniform> chromatic_aberration: ChromaticAberration;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let out = vec3<f32>(
        textureSample(source, source_sampler, in.uv + (chromatic_aberration.dir_r * chromatic_aberration.magnitude_r)).r,
        textureSample(source, source_sampler, in.uv + (chromatic_aberration.dir_g * chromatic_aberration.magnitude_g)).g,
        textureSample(source, source_sampler, in.uv + (chromatic_aberration.dir_b * chromatic_aberration.magnitude_b)).b,
    );

    return vec4<f32>(out, 1.0);
}
