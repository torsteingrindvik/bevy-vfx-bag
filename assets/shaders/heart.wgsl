// Provides View and Globals bindings
#import bevy_sprite::mesh2d_view_bindings

@group(1) @binding(0)
var<uniform> color: vec4<f32>;

@group(1) @binding(1)
var<uniform> num_hearts: f32;

struct Mouse {
    x: f32,
    y: f32,
};
@group(1) @binding(2)
var<uniform> mouse: Mouse;

struct HeartData {
    opacity: f32,
    scale: f32,
    angle: f32,
	_padding: f32,
};
@group(1) @binding(3)
var<uniform> heart_data: array<HeartData, 32>;

fn heart(
	uv: vec2<f32>
) -> f32 {
	let width = view.viewport.z;
	let height = view.viewport.w;
	
	// Range (-1., 1.) and aspect ratio corrected
	var uv = 2. * (-0.5 + uv) * vec2(width / height, 1.);

	uv.y -= 0.2;
	uv.y += 0.7 * sqrt(abs(uv.x));
	uv *= vec2(0.8, 1.5);

	let d = smoothstep(1.0, 0.99, length(uv));

	return d;
}

// @fragment
// fn fragment(
//     #import bevy_pbr::mesh_vertex_output
// ) -> @location(0) vec4<f32> {
// 	let p = fract(vec2(uv.x * num_hearts.x, uv.y * num_hearts.y));
// 	let h = heart(p);
// 	let col = h * color.rgb;

//     return vec4(col, h * color.a * 0.5);
// }

const FLOOR: f32 = 1.0;
const THING: f32 = 2.0;

// https://iquilezles.org/articles/smin/
fn op_union(object_1: f32, object_2: f32) -> f32 {
	let k = 0.13;
	let h = max(k - abs(object_1 - object_2), 0.0) / k;

	return min(object_1, object_2) - h * h * k * (1./4.);
}

// fn box(p: vec3<f32>, b: vec3<f32>) -> f32 {
//   let q = abs(p) - b;
//   return length(max(q, vec3(0.))) + min(max(q.x, max(q.y, q.z)), 0.);
// }

// fn weird_ball(p: vec3<f32>, radius: f32) -> f32 {
// 	let symm = abs(p);
// 	// let xabs = abs(p.x);
// 	// let xabs = p.x;

// 	// let py = p.y * 1.3 - xabs * sqrt(((10. - xabs) / 30.));

// 	// let p = vec3(p.x, p.y - pow(pabs, 1.3), p.z);
// 	// let p = vec3(p.x, py, p.z * (1. - (posy / 10.)));

// 	return length(p + 0.2) - radius;
// }

fn repeat_xz(p: vec3<f32>, factor: vec2<f32>) -> vec3<f32> {
	let tmp = vec2(
        p.x - factor.x * floor(p.x/factor.x),
        p.z - factor.y * floor(p.z/factor.y)
    ) - 0.5 * factor;
    
	return vec3(tmp.x, p.y, tmp.y);
}

fn repeat2(p: vec2<f32>, factor: vec2<f32>) -> vec2<f32> {
	return vec2(
        p.x - factor.x * floor(p.x/factor.x),
        p.y - factor.y * floor(p.y/factor.y)
    ) - 0.5 * factor;
}

fn ball(p: vec3<f32>, radius: f32) -> f32 {
	// var p = vec3(p.x, p.y / 2., p.z + 1.3 * abs(p.x));

	// let s = abs(p / 1.1);

	return length(p) - radius;
}

fn abschill(v: f32) -> f32 {
	// When v is big the extra term is negligible.
	// When v is small it smooths it out.
	return sqrt(v*v + 0.005);
}

fn map(p: vec3<f32>) -> vec2<f32> {
	let t = 0.;

	// var p = repeat_xz(p, vec2(5., 5.));
	// p.y *= 0.8;

	// let p = vec3(
	// 	abschill(p.x) % 2.0,
	// 	p.y,
	// 	abschill(p.z) % 2.0
	// );

	// var res = vec2(
	// 	ball(p - vec3(0., 1.2, 0.), 0.2),
	// 	THING,
	// );

	let height = 0.4;
	let y = sqrt(max(0.0, p.y)) - 0.2;

	var res = vec2(
		ball(vec3(abschill(p.x * 0.8) - y * 0.5, -height + p.y * 0.8, p.z), 0.3),
		THING
	);

	// floor
	let fl = 0.1 + p.y;
	if (fl < res.x) {
		// res = vec2(fl, FLOOR);
	};

	return res;
}

fn normal(p: vec3<f32>) -> vec3<f32> {
	let eps = vec3(0.0001, 0., 0.);

	return normalize(vec3(
		map(p + eps.xyy).x - map(p - eps.xyy).x,
		map(p + eps.yxy).x - map(p - eps.yxy).x,
		map(p + eps.yyx).x - map(p - eps.yyx).x
	));
}

// Returns (distance, material id)
fn ray_march(
	ro: vec3<f32>,
	rd: vec3<f32>,
) -> vec2<f32> {
	let eps = 1e-3;

	var t = 0.5;
	let t_max = 20.0;

	var res = vec2(-1.);

	for (var i: i32 = 0; i < 500 && t < t_max; i++) {
		let m = map(ro + rd * t);

		if (m.x < eps) {
			res = vec2(t, m.y);
			break;
		}

		t += m.x;
	}
	if t > t_max {
		res = vec2(-1.);
	}

	return res;
}

// Make p from -1 to 1, aspect independent, 
// bottom left (-1, -1) towards top right (1, 1)
fn uv2p(uv: vec2<f32>, view: View) -> vec3<f32> {
	var uv = vec2(uv.x, 1.0 - uv.y);
	uv.x *= num_hearts;
	let uvxi = floor(uv.x);
	uv.x = uv.x % 1.0;
	// uv.y = uv.y * 4.8;

	var p = (uv * 2.) - 1.;

	let width = view.viewport.z;
	let height = view.viewport.w;

	// aspect ratio independent (assume width > height)
	p.x *= width / height;

	return vec3(p, uvxi);
}

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
	// let uv = repeat2(uv, vec2(1.0));
	let p_idx = uv2p(uv, view);
	var p = p_idx.xy;
	let idx = u32(p_idx.z);
	let hd = heart_data[idx];

	let pi = 3.1415;

	// let angle = 0.1 * sin(globals.time * 5.) + 1.2 + mouse.x;
	let angle = (1. - hd.angle) * pi + 1.4;

	let r = 2.4;

	let ta = vec3(0., 0.65, 0.4);
	let ro = ta + vec3(r * cos(angle), -0.2, r * sin(angle));

	let lo = vec3(1., 1., 1.);
	// light: direction is towards sun as seen from target
	let ld = normalize(lo);
	let light_color = vec3(0.6, 0.35, 0.5);

	// camera looks from ro towards ta
	let cam_look = normalize(-ro + ta);
	let world_up = vec3(0., 1., 0.);

	let cam_right = normalize(cross(cam_look, world_up));
	let cam_up = normalize(cross(cam_right, cam_look));

	// let zoom = 1.8 + 3. * interp;
	let zoom = hd.scale * 4.0;

	// p = repeat2(p, vec2(2., 2.));

	let rd = normalize(p.x * cam_right + p.y * cam_up + zoom * cam_look);

	let rm = ray_march(
		ro,
		rd,
	);
	let t = rm.x;
	// Add a bit to avoid float comparison issues
	let id = rm.y + 0.1;

	// var col = vec3(0.0);
	var col = vec3(0.4, 0.7, 0.9) - vec3(0.5) * max(p.y, 0.);
	var a = 0.0;

	if (t > 0.0) {
		if (id > THING) {
			col = vec3(0.8, 0.02, 0.02);

			a = hd.opacity;
		} else if (id > FLOOR) {
			col = vec3(0.05, 0.09, 0.02);
		}

		let point = ro + rd * t;
		let n = normal(point);

		let sun_norm_alignment = max(0., dot(n, ld));

		let view = -rd;
		let sun_half_vector = normalize(view + ld);

		// 1.0 if can see sun directly,
		// 0.0 else
		let sun_unobstructed = step(ray_march(
			point + 0.001 * n,
			ld,
		).y, 0.);

		let specular_alignment = max(0.0, dot(n, sun_half_vector));
		let specular = pow(specular_alignment, 32.) * sun_norm_alignment;

		let sky_factor = sqrt(clamp(0.5 + 0.5 * n.y, 0.0, 1.0));

		let bottom_up_factor = pow(clamp(0.3 - 0.9 * n.y, 0.0, 0.5), 2.);

		var light_in = vec3(0.0);

		// create a color for the sun, adjust amplitude by alignment of
		// geometry, then forget all of it if the current point is not visible
		// to the sun.
		let sun_color = vec3(9., 6., 4.);
		light_in += sun_color * sun_norm_alignment * sun_unobstructed;

		// add some color based on the sky, adjust amplitude by how aligned
		// the geometry is to the sky.
		light_in += vec3(0.5, 0.7, 1.) * sky_factor;

		light_in += vec3(0.4, 1., 0.4) * bottom_up_factor;

		// now everything is scaled by this..
		col *= light_in;

		// specular comes in last?
		col += sun_color * specular * sun_unobstructed;


		col = mix(col, vec3(0.5, 0.6, 0.9), 1. - exp(-0.0001 * t * t * t));

		// col = vec3(bottom_up_factor);
		// a = 1.0;
		// a = heart_data[u32(idx)].opacity;
	}

    return vec4(col, a);
    // return vec4(col, 1.);
}
