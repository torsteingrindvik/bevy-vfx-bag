#define_import_path bevy_vfx_bag::math

// See Mark Jarzynski and Marc Olano, Hash Functions for GPU Rendering, Journal of Computer
// Graphics Techniques (JCGT), vol. 9, no. 3, 20–38, 2020 
fn pcg3d(v: vec3<u32>) -> vec3<u32> {
    var v = v * 1664525u + 1013904223u;

    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;

    v = v ^ (v >> 16u);

    v.x += v.y*v.z;
    v.y += v.z*v.x;
    v.z += v.x*v.y;

    return v;
}

// https://github.com/gfx-rs/naga/issues/1908
fn ldexp_workaround(v: f32, e: f32) -> f32 {
    return v * exp2(e);
}
fn ldexp_workaround2(v: vec2<f32>, e: f32) -> vec2<f32> {
    return v * exp2(e);
}

fn hash21(point: vec2<u32>) -> u32 {
    return pcg3d(vec3<u32>(point.xy, 0u)).x;
}

fn hash21f(point: vec2<u32>) -> f32 {
    // https://www.pcg-random.org/using-pcg-c.html 
    // We get a random value in a u32's range.
    // Divide it by 1/2^32 to produce a value in the [0, 1) range.
    return ldexp_workaround(f32(hash21(point)), -32.0);
}

fn hash22(point: vec2<u32>) -> vec2<u32> {
    return pcg3d(vec3<u32>(point.xy, 0u)).xy;
}

fn hash22f(point: vec2<u32>) -> vec2<f32> {
    return ldexp_workaround2(vec2<f32>(hash22(point)), -32.0);
}
