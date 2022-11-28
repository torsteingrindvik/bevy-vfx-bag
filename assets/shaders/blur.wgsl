#import bevy_core_pipeline::fullscreen_vertex_shader
#import bevy_pbr::mesh_view_types

@group(0) @binding(0)
var source: texture_2d<f32>;
@group(0) @binding(1)
var source_sampler: sampler;
@group(0) @binding(2)
var<uniform> globals: Globals;

struct Blur {
    amount: f32,
    kernel_radius: f32
};
@group(0) @binding(3)
var<uniform> blur: Blur;

fn s(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(source, source_sampler, uv).rgb;
}

fn p(x: f32, y: f32) -> vec2<f32> {
    return vec2<f32>(x, y) * blur.kernel_radius;
}

fn s_blurred(uv: vec2<f32>) -> vec3<f32> {
    let r = p(1.0, 0.0);
    let tr = p(1.0, 1.0);
    let t = p(0.0, 1.0);
    let tl = p(-1.0, 1.0);
    let l = p(-1.0, 0.0);
    let bl = p(-1.0, -1.0);
    let b = p(0.0, -1.0);
    let br = p(1.0, -1.0);

    return 
        (s(uv) + s(uv + r) + s(uv + tr) + s(uv + t) + s(uv + tl) + s(uv + l) + s(uv + bl) + s(uv + b) + s(uv + br)) / 9.
        ;
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let original = s(in.uv);
    let blurred = s_blurred(in.uv);

    let output = mix(original, blurred, blur.amount);

    return vec4<f32>(output, 1.0);
}
