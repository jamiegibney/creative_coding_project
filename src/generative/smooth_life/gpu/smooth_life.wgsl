fn emod(value: u32, bound: u32) -> u32 {
    return (value % bound + bound) % bound;
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

fn hash2(in: vec2<f32>) -> f32 {
    var p3 = fract(vec2<f32>(in.xyx) * 0.1031);
    p3 = p3 + dot(p3, p3.yzx + 33.33);

    return fract((p3.x + p3.y) * p3.z);
}

[[block]]
struct State {
    ri: f32;
    ra: f32;

    alpha_n: f32;
    alpha_m: f32;

    b1: f32;
    b2: f32;

    d1: f32;
    d2: f32;

    dt: f32;
    delta_time: f32;

    width: u32;
    height: u32;
    should_randomize: u32;
};

[[block]]
struct Buffer {
    data: [[stride(4)]] array<f32>;
};

[[group(0), binding(0)]]
var<storage, read_write> output: Buffer;

[[group(0), binding(1)]]
var<uniform> state: State;

[[block]]
struct Grid {
    data: array<f32, 65536>; // 256 * 256
};

var<workgroup> grid_diff: Grid;
var<workgroup> grid_main: Grid;

fn set_grid_main_at(x: u32, y: u32, value: f32) {
    let h = u32(256);

    // grid_main.data[y * h + x] = value;
    output.data[y * h + x] = value;

    return;
}

fn get_grid_main_at(x: u32, y: u32) -> f32 {
    let h = u32(256);
    // return grid_main.data[y * h + x];
    return output.data[y * h + x];
}

fn set_grid_diff_at(x: u32, y: u32, value: f32) {
    let h = u32(256);

    grid_diff.data[y * h + x] = value;

    return;
}

fn get_grid_diff_at(x: u32, y: u32) -> f32 {
    let h = u32(256);

    return grid_diff.data[y * h + x];
}

// Smoothlife functions

fn sigmoid(x: f32, a: f32, alpha: f32) -> f32 {
    return 1.0 / (1.0 + exp(-(x - a) * 4.0 / alpha));
}

fn sigmoid_n(x: f32, a: f32, b: f32) -> f32 {
    return sigmoid(x, a, state.alpha_n) * (1.0 - sigmoid(x, b, state.alpha_n));
}

fn sigmoid_m(x: f32, y: f32, m: f32) -> f32 {
    let sgm = sigmoid(m, 0.5, state.alpha_m);

    return x * (1.0 - sgm) + y * sgm;
}

// Source: https://arxiv.org/abs/1111.1567
fn transition(n: f32, m: f32) -> f32 {
    return sigmoid_n(n, sigmoid_m(state.b1, state.d1, m), sigmoid_m(state.b2, state.d2, m));
}

fn compute_diff(x: u32, y: u32) {
    let w: u32 = state.width;
    let h: u32 = state.height;
    let ri = state.ri;
    let ra = state.ra;

    var m: f32 = 0.0;
    var m_norm: f32 = 0.0;
    var n: f32 = 0.0;
    var n_norm: f32 = 0.0;

    let min: f32 = -(ra - 1.0);
    let max: f32 = -min;

    for (var dx: f32 = min; dx <= max; dx = dx + 1.0) {
        for (var dy: f32 = min; dy <= max; dy = dy + 1.0) {
            let rx = emod(x + u32(dx), w);
            let ry = emod(y + u32(dy), h);

            let d = dx * dx + dy * dy;

            if (d <= ri * ri) {
                m = m + get_grid_main_at(rx, ry);
                m_norm = m_norm + 1.0;
            }
            else {
                if (d <= ra * ra) {
                    n = n + get_grid_main_at(rx, ry);
                    n_norm = n_norm + 1.0;
                }
            }
        }
    }

    n = n / n_norm;
    m = m / m_norm;

    let q: f32 = transition(n, m);
    set_grid_diff_at(x, y, 2.0 * q - 1.0);
}

fn apply_diff(x: u32, y: u32, dt: f32) {
    let diff = get_grid_diff_at(x, y);
    let main = get_grid_main_at(x, y);
    let val = dt * diff + main + 0.002;

    set_grid_main_at(x, y, clamp(val, 0.0, 1.0));
    return;
}

[[stage(compute), workgroup_size(16, 16, 1)]]
fn main([[builtin(global_invocation_id)]] id: vec3<u32>) {
    let uv = vec2<f32>(f32(id.x) / f32(state.width), f32(id.y) / f32(state.height));

    let x = u32(id.x);
    let y = u32(id.y);

    let dt = state.dt * state.delta_time;

    var pxl: f32;
    if (state.should_randomize >= u32(1)) {
        set_grid_main_at(x, y, hash2(uv * 256.0));
    } else {
        compute_diff(x, y);
        apply_diff(x, y, dt);
    }

    return;
}

