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

// fn heart(
// 	uv: vec2<f32>
// ) -> f32 {
// 	let width = view.viewport.z;
// 	let height = view.viewport.w;
	
// 	// Range (-1., 1.) and aspect ratio corrected
// 	var uv = 2. * (-0.5 + uv) * vec2(width / height, 1.);

// 	uv.y -= 0.2;
// 	uv.y += 0.7 * sqrt(abs(uv.x));
// 	uv *= vec2(0.8, 1.5);

// 	let d = smoothstep(1.0, 0.99, length(uv));

// 	return d;
// }

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

fn sd_u(object_1: f32, object_2: f32, k: f32) -> f32 {
	let h = max(k - abs(object_1 - object_2), 0.0) / k;
	return min(object_1, object_2) - h * h * k * (1./4.);
}

// https://iquilezles.org/articles/smin/
fn op_union(object_1: f32, object_2: f32) -> f32 {
	return sd_u(object_1, object_2, 0.13);
}

fn sd_capsule_x(p: vec3<f32>, h: f32, r: f32) -> f32
{
	var p = p;
	// center
	p.x += h / 2.;
  	p.x -= clamp(p.x, 0.0, h);

  	return length(p) - r;
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

fn repeat(p: f32, factor: f32) -> vec2<f32> {
	let scaled = p / factor;
	let idx = floor(scaled);

	return vec2((p - factor * idx) - 0.5 * factor, floor(scaled));
}

// fn repeat(p: f32, factor: f32) -> vec2<f32> {
//     let q = (p % factor) - 0.5 * factor;
//     return vec2(q, p % factor);
// }

// fn repeat_xz(p: vec3<f32>, factor: vec2<f32>) -> vec3<f32> {
// 	let tmp = vec2(
//         p.x - factor.x * floor(p.x/factor.x),
//         p.z - factor.y * floor(p.z/factor.y)
//     ) - 0.5 * factor;
    
// 	return vec3(tmp.x, p.y, tmp.y);
// }

// fn repeat2(p: vec2<f32>, factor: vec2<f32>) -> vec2<f32> {
// 	return vec2(
//         p.x - factor.x * floor(p.x/factor.x),
//         p.y - factor.y * floor(p.y/factor.y)
//     ) - 0.5 * factor;
// }

fn ball(p: vec3<f32>, radius: f32) -> f32 {
	return length(p) - radius;
}

fn sabs(v: f32, k: f32) -> f32 {
	return sqrt(v*v + k);
}

fn abschill(v: f32) -> f32 {
	// When v is big the extra term is negligible.
	// When v is small it smooths it out.
	return sabs(v, 0.005);
}


fn heart(p: vec3<f32>, radius: f32) -> f32 {
	var p = p;

	p.x *= 0.90;

	p.x = sabs(p.x, 0.004);
	p.y -= p.x * sqrt((9. - p.x) / 10.);
	p.y += 0.1;

	return ball(p, radius);
}

// iq
fn smax(a: f32, b: f32, k: f32) -> f32 {
	let h = max(k - abs(a - b), 0.);
	return max(a, b) + h * h * 0.25 / k;
}

#ifdef BVB_UI_HEART
fn map(p: vec3<f32>) -> vec2<f32> {
	var res = vec2(
		heart(p, 0.5),
		THING
	);

	return res;
}
#else ifdef BVB_UI_BALL
fn map(p: vec3<f32>) -> vec2<f32> {
	var res = vec2(
		ball(p, 0.5),
		THING
	);

	return res;
}
#else ifdef BVB_UI_BONE
fn map(p: vec3<f32>) -> vec2<f32> {
	var p = p * 0.6;
	p.x = abs(p.x);
	p.y = sabs(p.y, 0.0001);

	var d1 = length(p - vec3(0.3, 0.1, 0.)) - 0.1;

	p.x *= 0.4;
	d1 = op_union(
		d1,
		ball(p, 0.08),
	);

	var res = vec2(
		d1,
		THING
	);

	return res;
}
#else ifdef BVB_UI_HAT
fn map(p: vec3<f32>) -> vec2<f32> {

	// Chop off a ball to get a smooth midsection
	var d1 = ball(p, 0.6);
	d1 = smax(d1, abs(p.y) - 0.04, 0.03);

	// let w = -1.3 + sqrt(length(p.xz)) * 5.;
	let w = 1.;

	d1 = op_union(
		d1,	
		length(p.xz) - 0.60 + sqrt(p.y * 0.45),
		// ball(vec3(abschill(p.x) + max(0.0, p.y * 0.4), p.y - 0.4, abs(p.z) + max(0.0, p.y * 0.4)), 0.45)
	);

	var res = vec2(
		d1,
		THING
	);

	return res;
}
#else ifdef BVB_UI_UNDECIDED
fn mod_(x: f32, y: f32) -> f32 {
	return x - y * floor(x / y);
}

fn map(p: vec3<f32>) -> vec2<f32> {

	let r = 0.4;
	let hr = r / 2.;
	var y = mod_(p.y + hr, r) - hr;

	// let y = ((p.y + 1.0 + hr) % r) - hr;
	// let idx = ceil(p.y + hr / r);

	// let ry = repeat(p.y, 0.4);
	// let an = globals.time + ry.y * 2.5;
	let an = globals.time + 2.5;

	let h = 1.0;
	let hh = h / 2.;

	let rotated = mat2x2(
		cos(an), -sin(an),
		sin(an), cos(an),
	) * p.xz;
	// let rotated = p.xz;

	// var y = clamp((p.y + 1.) / 2., 0.0, 1.0);
	// y *= 10.;
	// let yi = floor(y);
	// y = fract(y) - 0.5;

	// y /= 1.5;
	// y += 0.5 * sin(p.y * 5.);
	// vec3 q = mod(p+2.5, 5.0)-2.5;
	// let y = (abs(p.y) % 0.3) * sign(p.y);

	let q = vec3(
		rotated.x,
		// p.y + 0.2 * sin(globals.time * 3.),
		// y,
		// p.y + 0.4,
		y,
		// ry.x,
		rotated.y,
	);

	var d1 = sd_capsule_x(q, h, 0.10);

	var res = vec2(
		d1,
		THING
	);
	return res;
}
#endif

// iq
fn hash1(n: f32) -> f32
{
    return fract(sin(n)*813851.838134);
}

// iq
fn forward_sf(i: f32, n: f32) -> vec3<f32>
{
    let PI  = 3.141592653589793238;
    let PHI = 1.618033988749894848;
    let phi = 2.0*PI*fract(i/PHI);
    let zi = 1.0 - (2.0*i+1.0)/n;
    let sinTheta = sqrt( 1.0 - zi*zi);
    return vec3( cos(phi)*sinTheta, sin(phi)*sinTheta, zi);
}

// iq
fn ao(p: vec3<f32>, n: vec3<f32>) -> f32
{
	var ao = 0.0;

    for(var i: i32 = 0; i < 32; i++ )
    {
        var ap = forward_sf(f32(i), 32.0);

        let h = hash1(f32(i));
		ap *= sign( dot(ap, n) ) * h*0.1;
        ao += clamp( map( p + n*0.01 + ap).x*3.0, 0.0, 1.0 );
    }
	ao /= 32.0;
	
    return clamp( ao*6.0, 0.0, 1.0 );
}


fn normal(p: vec3<f32>) -> vec3<f32> {
	let eps = vec3(0.001, 0., 0.);

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

	var t = 0.1;
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

	// each index
	let uvxi = floor(uv.x);
	uv.x = uv.x % 1.0;

	var p = (uv * 2.) - 1.;

	// aspect ratio independent (assume width > height)
	let width = view.viewport.z;
	let height = view.viewport.w;
	// p.x *= width / height;

	// TODO: Let's rather fix the model than the space?
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
	let pi2 = pi * 2.;

	// let angle = pi2 * ( (1. - hd.angle) / 2. + 0.2 + 0.02 * sin(globals.time * 3.));
	let angle = pi2 * ( (1. - hd.angle) / 2. + 0.2 + 0.02 * globals.time * 3.);

	let r = 1.0;

	let ta = vec3(0., 0., 0.);
	let ro = ta + vec3(r * cos(angle), 0.4, r * sin(angle));

	let lo = vec3(1., 3., 3.);

	// light: direction is towards sun as seen from target
	let ld = normalize(lo);

	// camera looks from ro towards ta
	let cam_look = normalize(-ro + ta);
	let world_up = vec3(0., 1., 0.);

	let cam_right = normalize(cross(cam_look, world_up));
	let cam_up = normalize(cross(cam_right, cam_look));

	let zoom = hd.scale;

	let rd = normalize(p.x * cam_right + p.y * cam_up + zoom * cam_look);
	let rm = ray_march(
		ro,
		rd,
	);

	let t = rm.x;
	// Add a bit to avoid float comparison issues
	let id = rm.y + 0.1;

	var col = vec3(0.);
	var a = 0.;

	if (t > 0.) {
		let point = ro + rd * t;
		let n = normal(point);

		if (id > THING) {
			col = color.xyz;

			a = pow(hd.opacity, 6.);
			
			let fresnel = clamp(1.0 + dot(n, rd), 0.0, 1.0);
			col = 0.5 * col + 0.4 * fresnel * col;

		} else if (id > FLOOR) {
			col = vec3(1.);
		}

		let sun_norm_alignment = max(0., dot(n, ld));

		let view = -rd;
		let sun_half_vector = normalize(view + ld);

		// 1.0 if can see sun directly,
		// 0.0 else
		let sun_unobstructed = step(ray_march(
			point + 0.1 * n,
			ld,
		).y, 0.);

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
		light_in += vec3(0.5, 0.7, 1.) * sky_factor * 1.;

		light_in += vec3(0.4, 1., 0.4) * bottom_up_factor * 2.5;

		let ambient_occ = ao(point + n * 0.01, n);
		col *= light_in * 0.20;
		col *= ambient_occ;

		// specular
		let specular_alignment = max(0.0, dot(n, sun_half_vector));
		let specular = pow(specular_alignment, 16.) * sun_norm_alignment;
		col += sun_color * specular * sun_unobstructed * 0.05;



		// /*
		// TODO: Maybe make a debug flag or something?
		if (idx < 1u) {
			col = vec3(ambient_occ);
		} else if idx < 2u {
			col = vec3(n);
		}
		//  */
	}

    // return vec4(col + vec3(p, 0.)*0.01, max(a, 0.2));
    return vec4(col, a);
}
