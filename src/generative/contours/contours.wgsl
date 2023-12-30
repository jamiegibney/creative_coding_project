fn s_curve_round(in: f32, tension: f32) -> f32 {
    let x: f32 = clamp(in, -1.0, 1.0);
    let c: f32 = tension * 10.0;

    return tanh(in * c) / tanh(c);
}

fn curve(in: f32, tension: f32) -> f32 {
    let x: f32 = clamp(in, -1.0, 1.0);
    var c: f32;
    if (tension < 0.0) {
        c = clamp(tension, -1.0, 0.0) * 0.907;
    } else {
        c = clamp(tension, 0.0, 1.0) * 10.0;
    }

    if (0.0 < x && x <= 1.0) {
        return x * (1.0 + c) / (c * x) + 1.0;
    } else {
        return -x * (1.0 + c) / (c * x) - 1.0;
    }
}

fn s_curve(in: f32) -> f32 {
    let x: f32 = clamp(in, -1.0, 1.0);

    if (0.0 <= x) {
        return 1.0 - pow(1.0 - x, 2.0);
    } else {
        return pow(1.0 + x, 2.0) - 1.0;
    }
}

fn contains(lower: f32, upper: f32, value: f32) -> bool {
    return lower <= value && value <= upper;
}

fn map_norm(val: f32, min: f32, max: f32) -> f32 {
    return (val - min) / (max - min);
}

fn scale_to(val: f32, min: f32, max: f32) -> f32 {
    return val * (max - min) + min;
}

fn map(val: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    return scale_to(map_norm(val, in_min, in_max), out_min, out_max);
}

fn brightness(num_contours: u32, upper: f32, noise: f32) -> f32 {
    let mapped = ((noise + 1.0) / 2.0) * f32(num_contours);
    let px = fract(mapped);

    let lower: f32 = 0.0;
    let mid = mix(lower, upper, 0.5);
    let feathering = 0.02;

    // "s_curve_round" is used to feather the edges of the contours, acting as a quick
    // and dirty anti-aliasing solution. this doesn't work for very thin lines very well,
    // but is great otherwise.
    if (contains(lower, mid, px)) {
        return s_curve_round(map(px, lower, mid, 0.0, 0.5), 2.0);
        //return 0.0;
        //return smoothStep(0.0, feathering, px);
    } else {
        if (contains(mid, upper, px)) {
            return s_curve_round(map(px, upper, mid, 0.0, 0.5), 2.0);
            //return smoothStep(0.0, feathering, upper - px);
        }
        else {
            return 0.0;
        }
    }
}

fn hash2(in: vec2<f32>) -> f32 {
    var p3 = fract(vec2<f32>(in.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);

    return fract((p3.x + p3.y) * p3.z);
}

fn grad(pos: vec2<f32>, t: f32) -> vec2<f32> {
    //let rand: f32 = fract(sin(dot(int_pos, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let rand = hash2(pos);

    let angle = 6.283185 * rand + 4.0 * t * rand;

    return vec2<f32>(cos(angle), sin(angle));
}

// Perlin noise approximation
// Source: https://www.shadertoy.com/view/MtcGRl
fn pseudo_noise3(pos: vec3<f32>) -> f32 {
    let i: vec2<f32> = floor(pos.xy);
    let f: vec2<f32> = pos.xy - i;
    let blend = f * f * (3.0 - 2.0 * f);

    let noise = mix(
        mix(
            dot(grad(i + vec2<f32>(0.0, 0.0), pos.z), f - vec2<f32>(0.0, 0.0)),
            dot(grad(i + vec2<f32>(1.0, 0.0), pos.z), f - vec2<f32>(1.0, 0.0)),
            blend.x,
        ),
        mix(
            dot(grad(i + vec2<f32>(0.0, 1.0), pos.z), f - vec2<f32>(0.0, 1.0)),
            dot(grad(i + vec2<f32>(1.0, 1.0), pos.z), f - vec2<f32>(1.0, 1.0)),
            blend.x,
        ),
        blend.y
    );

    return noise / 0.7;
}

[[block]]
struct Buffer {
     data: [[stride(4)]] array<f32>;
};

[[block]]
struct Params {
    // Number of contour lines to draw.
    num_contours: u32;
    // Upper bound of the contour range.
    upper: f32;
    // The current z-value for the perlin noise.
    z: f32;
    // The horizontal resolution.
    width: u32;
    // The vertical resolution.
    height: u32;
};

// @group(0)
// @binding(0)
// [[group(0), binding(0)]]
[[group(0), binding(0)]]
var<storage, read_write> output: Buffer;

//var thread_buf: texture_storage_2d_array<f32>;

// @group(0)
// @binding(1)
[[group(0), binding(1)]]
var<uniform> params: Params;

// @compute
// @workgroup_size(1, 1, 1)
[[stage(compute), workgroup_size(16, 16, 1)]]
fn main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    // get uv coords
    let uv: vec2<f32> = vec2<f32>(f32(id.x) / f32(params.width), f32(id.y) / f32(params.height));

    // generate raw noise value
    let noise: f32 = pseudo_noise3(vec3<f32>(uv, params.z));

    // get contoured brightness value
    let br: f32 = brightness(params.num_contours, params.upper, noise);

    // copy value to output buffer
    output.data[id.y * params.height + id.x] = br;

    return;
}
