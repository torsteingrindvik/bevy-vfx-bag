#define_import_path bevy_vfx_bag::fbm
#import bevy_vfx_bag::value_noise

/*
TODO:
- Expose parameters
- Consider if parameters should be shader preprocessor values
- The noise used should be defined by a shader preprocessor definition
*/

const NUM_OCTAVES: u32 = 8u;
const hurst: f32 = 1.0;

fn fbm(coords: vec2<f32>) -> f32 {
    let gain = exp2(-hurst); 
    var result = 0.0;

    var amplitude = 1.0;
    var frequency = 1.0;

    for (var i: u32 = 0u; i < NUM_OCTAVES; i++) {
        let noise = value_noise(coords * frequency);
        result += amplitude * noise;

        amplitude *= gain;
        frequency *= 2.041002312;
    }

    return result;
}