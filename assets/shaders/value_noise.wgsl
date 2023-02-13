#define_import_path bevy_vfx_bag::value_noise
#import bevy_vfx_bag::math

fn value_noise(coords: vec2<f32>) -> f32 {
    let index = vec2<u32>(floor(coords));
    let frac = fract(coords);

    // Sometimes a smoothstepped frac is used instead.
    let interpolant = frac;

    let noise_xy00 = hash21f(index + vec2(0u, 0u));
    let noise_xy10 = hash21f(index + vec2(1u, 0u));
    let noise_xy01 = hash21f(index + vec2(0u, 1u));
    let noise_xy11 = hash21f(index + vec2(1u, 1u));

    // Gives us the noise at the correct point in the x direction
    // between the upper corners
    let noise_x0_lerp = mix(f32(noise_xy00), f32(noise_xy10), interpolant.x);

    // x direction lower corners
    let noise_x1_lerp = mix(f32(noise_xy01), f32(noise_xy11), interpolant.x);

    // Lastly lerp between the values found in the y direction.
    let noise = mix(noise_x0_lerp, noise_x1_lerp, interpolant.y);

    return noise;
}
