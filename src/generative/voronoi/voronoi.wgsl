// Voronoi border/isolines algorithm courtesy of Inigo Quilez:
// https://iquilezles.org/articles/voronoilines
//
// Inigo's original GLSL shader code, of which this implementation is based:
// https://www.shadertoy.com/view/ldl3W8
//
// This version is adapted to cover points at arbitrary positions. In this example, an
// array of 2-dimensional vectors are passed to the shader from the CPU, and the shader 
// computes the stable Voronoi borders and so-called "isolines" from those points.


/// *** *** STATE *** *** ///

// Output image buffer.
[[block]]
struct Buffer {
    data: [[stride(4)]] array<f32>;
};

[[group(0), binding(0)]]
var<storage, read_write> output: Buffer;

// The points from the CPU.
[[block]]
struct Points {
    data: [[stride(8)]] array<vec2<f32>, 32>;
};

[[group(0), binding(1)]]
var<uniform> points: Points;

// The algorithm state.
[[block]]
struct State {
    // The number of active cells.
    num_active: u32;
    // The weight of the Voronoi border and isolines.
    weight: f32;
    // The image width.
    width: u32;
    // The image height.
    height: u32;
};

[[group(0), binding(2)]]
var<uniform> state: State;

/// *** *** LOGIC *** *** ///

fn get_uv(x: vec2<f32>) -> vec2<f32> {
    let w = f32(state.width);
    let h = f32(state.height);

    return vec2<f32>(x.x / w, x.y / h);
}

fn voronoi(in: vec2<f32>) -> f32 {
    var min_dist: f32 = 10000.0;
    var min_relative: vec2<f32> = vec2<f32>(0.0, 0.0);
    var min_idx: u32 = u32(32);

    // find the point closest to the pixel
    for (var i: u32 = u32(0); i < state.num_active; i = i + u32(1)) {
        let point = get_uv(points.data[i]);

        let relative = in - point;

        let dist: f32 = dot(relative, relative);

        if (dist < min_dist) {
            min_dist = dist;
            min_relative = relative;
            min_idx = i;
        }
    }

    min_dist = 10000.0;

    for (var i: u32 = u32(0); i < state.num_active; i = i + u32(1)) {
        let point = get_uv(points.data[i]);

        let relative = in - point;

        if (dot(
                min_relative - relative,
                min_relative - relative
           ) > 0.000001) {
            min_dist = min(
                min_dist,
                dot(
                    0.5 * (min_relative + relative),
                    normalize(relative - min_relative)
                )
            );
        }
    }

    return min_dist;
}

/// *** *** MAIN *** *** ///

fn set_output_at(x: u32, y: u32, value: f32) {
    output.data[y * state.width + x] = value;
}

[[stage(compute), workgroup_size(16, 16, 1)]]
fn main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    let uv = vec2<f32>(f32(id.x) / f32(state.width), f32(id.y) / f32(state.height));

    let x = u32(id.x);
    let y = u32(id.y);

    let voro = voronoi(uv);
    let output: f32 = voro * (0.5 + 0.9 * sin(mix(465.0, 195.0, state.weight) * voro)) * 1.5 - 0.02;

    let min = 0.02 * state.weight;
    let max = min + 0.005;
    // let output: f32 = mix(1.0, output, smoothStep(0.004, 0.009, voro));
    let output: f32 = mix(1.0, output, smoothStep(min, max, voro));

    set_output_at(x, y, output);

    return;
}
