// #import bevy_pbr::mesh_types
// The time since startup data is in the globals binding which is part of the mesh_view_bindings import
// #import bevy_pbr::mesh_view_bindings

#import bevy_pbr::utils

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

struct Globals {
    // The time since startup in seconds
    // Wraps to 0 after 1 hour.
    time: f32,
    // The delta time since the previous frame in seconds
    delta_time: f32,
    // Frame count since the start of the app.
    // It wraps to zero when it reaches the maximum value of a u32.
    frame_count: u32,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

struct View {
    view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    world_position: vec3<f32>,
    // viewport(x_origin, y_origin, width, height)
    viewport: vec4<f32>,
};

@group(0) @binding(1)
var<uniform> view: View;

@vertex
fn vertex(
    @location(0) vertex_position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    // out.position = vec4<f32>(0.0, vertex_position.y, 0.0, 1.0);
    out.position = vec4<f32>(vertex_position, 0.0, 1.0);
    return out;
}

@fragment
fn fragment(
    // #import bevy_pbr::mesh_vertex_output
    in: VertexOutput
) -> @location(0) vec4<f32> {
    // var color = textureSample(sprite_texture, sprite_sampler, in.uv);
    // color = in.color * color;
    // return vec4<f32>(globals.time, 0.0, 0.0, 0.75);

    // (0.0 -> 1.0)
    var uv = coords_to_viewport_uv(in.position.xy, view.viewport);

    // (-1.0 -> 1.0)
    uv = uv * 2. - 1.;

    uv.x += 0.0;

    // uv = sqrt(uv * uv);
    // uv.x *= (view.viewport.z / view.viewport.w);

    var circle = length(uv);

    // var dist = distance(uv, vec2<f32>(0.0));

    // var t_wrap = (sin(globals.time * 1.5) + 1.0) / 2.0;
    // var out = smoothstep(vec2<f32>(0.0), vec2<f32>(t_wrap), dist);

    // var alpha = max(out.x, out.y);
    // var blur = 0.1;
    var radius = 1.0;

    // var vig = uv.x * uv.y * 10.;
    // vig = pow(vig, 1.55);
    // dist = smoothstep(radius - blur, radius, dist);
    var mask = step(radius, circle);

    return vec4<f32>(0.0, 0.0, 0.0, mask);
}
