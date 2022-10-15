#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

@group(1) @binding(2)
var lut: texture_3d<f32>;

@group(1) @binding(3)
var lut_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv = coords_to_viewport_uv(position.xy, view.viewport);

    // https://developer.nvidia.com/gpugems/gpugems2/part-iii-high-quality-rendering/chapter-24-using-lookup-tables-accelerate-color
    // I'm honestly not sure why this is necessary, I don't quite follow the reasoning.
    // But the neutral LUT seems indistinguishable from the original input texture
    // when this is used. Great!
    let half_texel = vec3<f32>(1.0 / 64. / 2.);

    let raw_color = textureSample(texture, our_sampler, uv).rgb;
    let lut_color = textureSample(lut, lut_sampler, raw_color + half_texel).rgb;

    if (uv.x < 0.5) {
        return vec4<f32>(lut_color, 1.0);
    } else {
        return vec4<f32>(raw_color, 1.0);
    }
}
