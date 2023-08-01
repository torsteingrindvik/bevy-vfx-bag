#import bevy_core_pipeline::fullscreen_vertex_shader FullscreenVertexOutput
#import bevy_render::globals Globals

@group(0) @binding(0)
var t: texture_2d<f32>;
@group(0) @binding(1)
var ts: sampler;
@group(0) @binding(2)
var<uniform> globals: Globals;

struct Raindrops {
    time_scaling: f32,
    intensity: f32,
    zoom: f32
};

@group(1) @binding(0)
var t_rain: texture_2d<f32>;
@group(1) @binding(1)
var ts_rain: sampler;
@group(1) @binding(2)
var<uniform> raindrops: Raindrops;

// These channels should be (-1, 1) but come as (0, 1)
fn remap_raindrops_rga(
    rga: vec3<f32>
) -> vec3<f32> {
    return (rga * 2.) - 1.;
}

fn animation(raindrops_b: f32) -> f32 {
    return fract(raindrops_b - (globals.time * raindrops.time_scaling));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // Raindrops texture wraps.
    // Make aspect-ratio independent UV coords.
    let resolution = vec2<f32>(textureDimensions(t));
    let uv_aspect_fixed = vec2<f32>(in.uv.x * resolution.x / resolution.y, in.uv.y);

    let t_raindrops = textureSample(t_rain, ts_rain, uv_aspect_fixed * raindrops.zoom).rgba;
    let t_raindrops_rga = remap_raindrops_rga(t_raindrops.rga);

    // Really the alpha channel of the original texture.
    // The channel is intentionally made such that the saturation
    // function separates only the parts we want.
    let mask_anim = saturate(t_raindrops_rga.b);

    // If we negate the mask, we then separate not the parts
    // which should be animated but the parts which should be static.
    let mask_neg = -1. * t_raindrops_rga.b;
    let mask_static = saturate(mask_neg);

    // Using (-1, 1) range offsets in the droplet positions
    // means the droplets would span the entire scene.
    // Thus scale it far down (by default).
    let offset = t_raindrops_rga.rg * raindrops.intensity;

    let mask = (animation(t_raindrops.b) * mask_anim) + mask_static;
    let masked_norms = mask * offset;

    return vec4<f32>(textureSample(t, ts, in.uv + masked_norms).rgb, 1.0);
}
