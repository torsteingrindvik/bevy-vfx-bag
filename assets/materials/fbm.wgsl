#import bevy_vfx_bag::fbm

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
    return vec4<f32>(vec3(fbm((material.scale * uv) + vec2(material.offset_x, material.offset_y))), 1.0);
}
