#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

struct Blur {
    amount: f32,
    kernel_radius: f32
};
@group(1) @binding(2)
var<uniform> blur: Blur;

fn s(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(texture, our_sampler, uv).rgb;
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

fn fragment_impl(
    position: vec4<f32>,
    uv: vec2<f32>
) -> vec4<f32> {
    let original = s(uv);
    let blurred = s_blurred(uv);

    let output = mix(original, blurred, blur.amount);

    return vec4<f32>(output, 1.0);
}

#import bevy_vfx_bag::post_processing_passthrough
