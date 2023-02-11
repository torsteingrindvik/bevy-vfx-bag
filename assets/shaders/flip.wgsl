#import bevy_core_pipeline::fullscreen_vertex_shader
#import bevy_render::globals

@group(0) @binding(0)
var t: texture_2d<f32>;
@group(0) @binding(1)
var ts: sampler;
@group(0) @binding(2)
var<uniform> globals: Globals;

struct Flip {
    x: f32,
    y: f32,
};
@group(1) @binding(0)
var<uniform> flip: Flip;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = abs(vec2<f32>(flip.x, flip.y) - in.uv);
    return textureSample(t, ts, uv);
}