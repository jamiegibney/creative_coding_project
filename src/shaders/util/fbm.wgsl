// MIT License. © Inigo Quilez, Munrocket
// SOURCE: https://gist.github.com/munrocket/236ed5ba7e409b8bdf1ff6eca5dcdc39

var m2: mat2x2f = mat2x2f(vec2f(0.8, 0.6), vec2f(-0.6, 0.8));

fn fbm(p: vec2f) -> f32 {
    var f: f32 = 0.;

    f = f + 0.5000 * noise2(p); p = m2 * p * 2.02;
    f = f + 0.2500 * noise2(p); p = m2 * p * 2.03;
    f = f + 0.1250 * noise2(p); p = m2 * p * 2.01;
    f = f + 0.0625 * noise2(p);

    return f / 0.9375;
}

fn noise2(p: vec2f) -> f32 {
    // put any noise here: Value, Perlin, Simplex, Worley
}
