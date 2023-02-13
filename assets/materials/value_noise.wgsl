#import bevy_vfx_bag::value_noise

struct CustomMaterial {
    scale: f32,
    offset_x: f32,
    offset_y: f32,
};
@group(1) @binding(0)
var<uniform> material: CustomMaterial;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let noise = value_noise((material.scale * uv) + vec2(material.offset_x, material.offset_y));
    return vec4<f32>(vec3(noise), 1.0);
}
