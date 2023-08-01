#import bevy_core_pipeline::fullscreen_vertex_shader FullscreenVertexOutput
#import bevy_render::globals Globals

@group(0) @binding(0)
var t: texture_2d<f32>;
@group(0) @binding(1)
var ts: sampler;
@group(0) @binding(2)
var<uniform> globals: Globals;

struct Mask {
    strength: f32,
    fade: f32,
};
@group(1) @binding(0)
var<uniform> mask: Mask;

#ifdef SQUARE
// A rounded square type mask.
fn square(uv: vec2<f32>) -> f32 {
    // The trick is to make the UV saturate quickly-
    // this impacts the width of the effect.
    // However this only creates a border in one corner.
    // The (1 - uv) version creates the diagonally mirrored border.
    let uv_big = saturate(uv * mask.strength);
    let uv_big_inv = saturate((1. - uv) * mask.strength);

    // By multiplying the mirrored borders we can get a full border.
    let square = uv_big * uv_big_inv;

    // The border is made by saturing UV coordinates.
    // This means the border is increasingly red and green in different
    // directions.
    // By multiplying them together we get a single unified border.
    let mask = square.r * square.g;

    return mask;
}
#endif

#ifdef CRT
// Also a rounded square type mask, but more oval.
// Reminiscent of a CRT television.
fn crt(uv: vec2<f32>) -> f32 {
    let square = uv * (1. - uv);
    var norm = square.r * square.g;
    norm *= norm;
    norm *= mask.strength;

    return saturate(norm);
}
#endif

#ifdef VIGNETTE
// Vignette type mask.
fn vignette(uv: vec2<f32>) -> f32 {
    // Strategy is to use the UV distance from the screen's center.
    var uv_centered = uv * 2. - 1.;

    // By scaling this we can adjust how bright/dark the vignette is.
    uv_centered *= mask.strength;

    let zero = vec2<f32>(0.);

    var dist = saturate(distance(zero, uv_centered));
    dist = pow(dist, 1.5);
    dist = 1. - dist;
    dist += 0.05;

    return saturate(dist);
}
#endif

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(t, ts, in.uv);

    #ifdef SQUARE
    let result = square(in.uv);
    #endif
    #ifdef CRT
    let result = crt(in.uv);
    #endif
    #ifdef VIGNETTE
    let result = vignette(in.uv);
    #endif

    return vec4<f32>(sample.rgb * saturate(result + mask.fade), 1.0);
}
