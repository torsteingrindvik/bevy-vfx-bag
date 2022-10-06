#import bevy_pbr::utils

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

// TODO: Import the Bevy definition somehow?
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

// TODO: Import the Bevy definition somehow?
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

struct Vignette {
    radius: f32,
    feathering: f32,
    color: vec4<f32>,
}

@group(0) @binding(2)
var<uniform> vignette: Vignette;

// TODO: This can be re-used for all our simple quad effects probably
@vertex
fn vertex(
    @location(0) vertex_position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex_position, 0.0, 1.0);
    return out;
}

@fragment
fn fragment(
    in: VertexOutput
) -> @location(0) vec4<f32> {
    // (0.0 -> 1.0)
    var uv = coords_to_viewport_uv(in.position.xy, view.viewport);

    // (-1.0 -> 1.0)
    uv = uv * 2. - 1.;

    var circle = length(uv);
    var radius = 1.0;

    var mask = smoothstep(vignette.radius, vignette.radius + vignette.feathering, circle);

    return vec4<f32>(vignette.color.rgb, mask * vignette.color.a);
}
