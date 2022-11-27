#import bevy_core_pipeline::fullscreen_vertex_shader

@group(0) @binding(0)
var source: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;

struct Pixelate {
    block_size: f32,
};
@group(0) @binding(2)
var<uniform> pixelate: Pixelate;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let resolution = vec2<f32>(textureDimensions(source));

    let width_height_over_block_size = resolution / max(1.0, pixelate.block_size);

    var uv = in.uv + 0.5;
    uv *= width_height_over_block_size;
    uv = floor(uv);
    uv /= width_height_over_block_size;
    uv -= 0.5;

    return textureSample(source, source_sampler, uv); 

    // asdf 11
}
