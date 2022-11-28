#import bevy_core_pipeline::fullscreen_vertex_shader
#import bevy_pbr::mesh_view_types

@group(0) @binding(0)
var source: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;
@group(0) @binding(2)
var<uniform> globals: Globals;

struct Flip {
    x: f32,
    y: f32,
};
@group(0) @binding(3)
var<uniform> flip: Flip;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = abs(vec2<f32>(flip.x, flip.y) - in.uv);
    return textureSample(source, source_sampler, uv);
}
