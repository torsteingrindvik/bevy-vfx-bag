#import bevy_vfx_bag::math

struct Material {
    scale: f32,
    offset_x: f32,
    offset_y: f32,
};
@group(1) @binding(0)
var<uniform> material: Material;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    var uv = uv * 8.;
    let id = vec2<i32>(1. + floor(uv));
    let f = -0.5 + fract(uv);

    var min_dist = 10.;

    // If the wrapping gets strange we can always bias with a +1
    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            let offset = vec2(x, y);

            // Random 2D vector in range (-0.5, +0.5)
            let random2d = hash22f(vec2<u32>(id + offset)) / 2.;
            let p = vec2<f32>(offset) + sin(random2d * material.offset_x);

            let dist = length(f - p);

            if (dist < min_dist) {
                min_dist = dist;
            }
        }
    }

    min_dist = 1. - min_dist;
    min_dist = smoothstep(0.5, 1.0, min_dist);

    return vec4<f32>(min_dist, min_dist * 2., 0.0, 1.0);
}
