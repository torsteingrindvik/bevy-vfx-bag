#import bevy_vfx_bag::math

struct Material {
    scale: f32,
    offset_x: f32,
    offset_y: f32,
};
@group(1) @binding(0)
var<uniform> material: Material;

fn voronoi(
    uv: vec2<f32>
) -> f32 {
    var uv = uv * material.scale;
    let id = vec2<i32>(floor(uv));

    let cell_space = -0.5 + fract(uv);

    var min_dist = 10.0;

    for (var y: i32 = -1; y <= 1; y++) {
        for (var x: i32 = -1; x <= 1; x++) {
            let offset = vec2(x, y);

            // Random 2D vector in range (-0.5, +0.5).
            // Note that the strategy is to go from normal UVs (0,0) -> (1,1) to something scaled, like
            // (0,0) -> (10,10) (for example).
            // We floor that to get integer IDs.
            // Then we add an offset in order to index our neighbours.
            let neighbour_random_point = hash22f(vec2<u32>(id + offset)) / 2.;

            // So now we have a random 2D point which is deterministic with regards
            // to the ID of the cell.
            // What we care about though is how far away this point is relative to _this_ cell.
            // Therefore we have to add the offset of the cell with the point in question to realize
            // that distance.
            // var p = sin(random2d * vec2(material.offset_x, material.offset_y)) + vec2<f32>(offset);
            // var p = sin(neighbour_random_point * vec2(material.offset_x, material.offset_y)) + vec2<f32>(offset);
            var p = sin(neighbour_random_point) + vec2<f32>(offset);
            // var p = random2d + vec2<f32>(offset);

            // p += sin(vec2(material.offset_x, material.offset_y)) / 2.;

            let dist = length(cell_space - p);

            if (dist < min_dist) {
                min_dist = dist;
            }
        }
    }

    // min_dist = 1. - min(1., min_dist);
    min_dist = smoothstep(0.31, 1.0, min_dist);

    return min_dist;
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    // let v = voronoi(uv);
    // return vec4<f32>(v * 0.1, v * 4., v, 1.0);

    let uv = uv * 10.;

    let xoff = sin(material.offset_x / 2.) / 2.1;
    let yoff = cos(material.offset_x / 3.) / 2.1;
    let off = vec2(xoff, yoff);

    let a = vec2(3. + xoff);
    let b = vec2(6., 6. + yoff);

    let auv = -a + uv;
    let ab = -a + b;

    let h = clamp(dot(auv, normalize(ab)) / length(ab), 0., 1.);

    let d = smoothstep(0.5, 0.6, length(-auv + h * ab));
    // let d = length(h * ab);

    return vec4<f32>(vec3(d), 1.0);
}
