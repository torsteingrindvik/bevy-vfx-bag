#import bevy_core_pipeline::fullscreen_vertex_shader
#import bevy_pbr::globals

@group(0) @binding(0)
var t: texture_2d<f32>;
@group(0) @binding(1)
var ts: sampler;
@group(0) @binding(2)
var<uniform> globals: Globals;

@group(1) @binding(0)
var lut: texture_3d<f32>;

@group(1) @binding(1)
var luts: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // https://developer.nvidia.com/gpugems/gpugems2/part-iii-high-quality-rendering/chapter-24-using-lookup-tables-accelerate-color
    // I'm honestly not sure why this is necessary, I don't quite follow the reasoning.
    // But the neutral LUT seems indistinguishable from the original input texture
    // when this is used. Great!
    let half_texel = vec3<f32>(1.0 / 64. / 2.);

    // Notice the ".rbg".
    // If we sample the LUT using ".rgb" instead,
    // the way the 3D texture is loaded will mean the
    // green and blue colors are swapped.
    // This mitigates that.
    let raw_color = textureSample(t, ts, in.uv).rbg;
    return vec4<f32>(textureSample(lut, luts, raw_color + half_texel).rgb, 1.0);
}