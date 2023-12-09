// By Morgan McGuire @morgan3d, http://graphicscodex.com
// Reuse permitted under the BSD license.

// All noise functions are designed for values on integer scale.
// They are tuned to avoid visible periodicity for both positive and
// negative coordinates within a few orders of magnitude.

// adapted by 443eb9 from https://www.shadertoy.com/view/4dS3Wd
#define_import_path bevy_entitiles::value_noise

// Precision-adjusted variations of https://www.shadertoy.com/view/4djSRW

fn hash(p: f32) -> f32 {
	var px = fract(p * 0.011);
	px *= px + 7.5;
	px *= px + px;
	return fract(px);
}

fn hash_2d(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.13);
    p3 += dot(p3, p3.yzx + 3.333);
    return fract((p3.x + p3.y) * p3.z);
}

fn noise_2d(x: vec2<f32>) -> f32 {
    let i = floor(x);
    let f = fract(x);

	// Four corners in 2D of a tile
	let a = hash_2d(i);
    let b = hash_2d(i + vec2<f32>(1.0, 0.0));
    let c = hash_2d(i + vec2<f32>(0.0, 1.0));
    let d = hash_2d(i + vec2<f32>(1.0, 1.0));

    // Simple 2D lerp using smoothstep envelope between the values.
	// return vec3(mix(mix(a, b, smoothstep(0.0, 1.0, f.x)),
	//			mix(c, d, smoothstep(0.0, 1.0, f.x)),
	//			smoothstep(0.0, 1.0, f.y)));

	// Same code, with the clamps in smoothstep and common subexpressions
	// optimized away.
    let u = f * f * (3.0 - 2.0 * f);
	return mix(a, b, u.x) + (c - a) * u.y * (1.0 - u.x) + (d - b) * u.x * u.y;
}

fn noise_3d(x: vec3<f32>) -> f32 {
    let step = vec3<i32>(110, 241, 171);

    let i = floor(x);
    let f = fract(x);
 
    // For performance, compute the base input to a 1D hash from the integer part of the argument and the 
    // incremental change to the 1D based on the 3D -> 1D wrapping
    let n = dot(i, vec3<f32>(step));

    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(mix(hash(n + f32(dot(step, vec3<i32>(0, 0, 0)))), hash(n + f32(dot(step, vec3<i32>(1, 0, 0)))), u.x),
                   mix(hash(n + f32(dot(step, vec3<i32>(0, 1, 0)))), hash(n + f32(dot(step, vec3<i32>(1, 1, 0)))), u.x), u.y),
               mix(mix(hash(n + f32(dot(step, vec3<i32>(0, 0, 1)))), hash(n + f32(dot(step, vec3<i32>(1, 0, 1)))), u.x),
                   mix(hash(n + f32(dot(step, vec3<i32>(0, 1, 1)))), hash(n + f32(dot(step, vec3<i32>(1, 1, 1)))), u.x), u.y), u.z);
}

fn fbm_2d(x: vec2<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    var xx = x;
	var v = 0.0;
	var a = 0.5;
	let shift = vec2<f32>(100.);
	// Rotate to reduce axial bias
    let rot = mat2x2f(cos(0.5), sin(0.5), -sin(0.5), cos(0.50));
	for (var i = 0; i < octaves; i += 1) {
		v += a * noise_2d(xx);
		xx = rot * xx * lacunarity + shift;
		a *= gain;
	}
	return v;
}

fn fbm_3d(x: vec3<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    var xx = x;
	var v = 0.0;
	var a = 0.5;
	let shift = vec3<f32>(100.);
	for (var i = 0; i < octaves; i += 1) {
		v += a * noise_3d(xx);
		xx = xx * lacunarity + shift;
		a *= gain;
	}
	return v;
}
