// Provides View and Globals bindings
#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var<uniform> color: vec4<f32>;

@group(1) @binding(1)
var<uniform> num_hearts: vec2<f32>;

fn heart(
	uv: vec2<f32>
) -> f32 {
	let width = view.viewport.z;
	let height = view.viewport.w;
	
	// Range (-1., 1.) and aspect ratio corrected
	var uv = 2. * (-0.5 + uv) * vec2(width / height, 1.);

	uv.y -= 0.2;
	uv.y += 0.7 * sqrt(abs(uv.x));
	uv *= vec2(0.9, 1.5);

	let d = smoothstep(1.0, 0.99, length(uv));

	return d;
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
	let p = fract(vec2(uv.x * num_hearts.x, uv.y * num_hearts.y));
	let h = heart(p);
	let col = h * color.rgb;

    return vec4(col, h * color.a);
}
