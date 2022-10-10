#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils


@group(1) @binding(0)
var texture: texture_2d<f32>;

@group(1) @binding(1)
var our_sampler: sampler;

@group(1) @binding(2)
var texture_raindrops: texture_2d<f32>;

@group(1) @binding(3)
var sampler_raindrops: sampler;

struct Raindrops {
    time_scaling: f32,
    intensity: f32,
    zoom: f32
};
@group(1) @binding(4)
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
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    let uv = coords_to_viewport_uv(position.xy, view.viewport);

    // Raindrops texture wraps.
    // Make aspect-ratio independent UV coords.
    let uv_aspect_fixed = vec2<f32>(uv.x * view.viewport.z / view.viewport.w, uv.y);

    let t_raindrops = textureSample(texture_raindrops, sampler_raindrops, uv_aspect_fixed * raindrops.zoom).rgba;
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

    return vec4<f32>(textureSample(texture, our_sampler, uv + masked_norms).rgb, 1.0);
}
