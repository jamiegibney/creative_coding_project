// The MIT License
// Copyright © 2013 Inigo Quilez
// https://www.youtube.com/c/InigoQuilez
// https://iquilezles.org/
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

// I've not seen anybody out there computing correct cell interior distances for Voronoi
// patterns yet. That's why they cannot shade the cell interior correctly, and why you've
// never seen cell boundaries rendered correctly.
//
// However, here's how you do mathematically correct distances (note the equidistant and non
// degenerated grey isolines inside the cells) and hence edges (in yellow):
//
// https://iquilezles.org/articles/voronoilines
//
// More Voronoi shaders:
//
// Exact edges:  https://www.shadertoy.com/view/ldl3W8
// Hierarchical: https://www.shadertoy.com/view/Xll3zX
// Smooth:       https://www.shadertoy.com/view/ldB3zc
// Voronoise:    https://www.shadertoy.com/view/Xd23Dh

#define ANIMATE

vec2 hash2(vec2 p) {
    // texture based white noise
    // return textureLod(iChannel0, (p + 0.5) / 256.0, 0.0).xy;

    // procedural white noise
    return fract(sin(vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)))) * 43758.5453);
}

vec3 voronoi(in vec2 x) {
    vec2 integerPart = floor(x);
    vec2 fractionalPart = fract(x);

    // first pass: regular voronoi
    vec2 minGrid, minRemainder;
    float minDistance = 8.0;

    // iterate 3x3 grid
    for (int j = -1; j <= 1; j++) {
        for (int i = -1; i <= 1; i++) {
            // offset for this iteration
            vec2 cellOffset = vec2(float(i), float(j));
            // get noise value for cell
            vec2 offset = hash2(integerPart + cellOffset);

            #ifdef ANIMATE
            offset = 0.5 + 0.5 * sin(iTime + 6.2831 * offset);
            #endif

            vec2 remainder = cellOffset + offset - fractionalPart;
            float dist = dot(remainder, remainder);

            if (dist < minDistance) {
                minDistance = dist;
                minRemainder = remainder;
                minGrid = cellOffset;
            }
        }
    }

    // second pass: distance to borders
    minDistance = 8.0;

    for (int j = -2; j <= 2; j++) {
        for (int i = -2; i <= 2; i++) {
            vec2 cellOffset = minGrid + vec2(float(i), float(j));
            vec2 offset = hash2(integerPart + cellOffset);

            #ifdef ANIMATE
            offset = 0.5 + 0.5 * sin(iTime + 6.2831 * offset);
            #endif

            vec2 remainder = cellOffset + offset - fractionalPart;

            if (dot(
                    minRemainder - remainder,
                    minRemainder - remainder
                ) > 0.00001)
            {
                minDistance =
                    min(
                        minDistance,
                        dot(
                            0.5 * (minRemainder + remainder),
                            normalize(remainder - minRemainder)
                        )
                    );
            }
        }
    }

    return vec3(minDistance, minRemainder);
}

vec2 voronoi2(in vec2 x) {
    ivec2 integerPart = floor(x);
    vec2 fractionalPart = fract(x);
    vec2 res = vec2(8.0);

    for (int j = -1; j <= 1; j++) {
        for (int i = -1; i <= 1; i++) {
            ivec2 offset = ivec2(i, j);
            vec2 r = pvec2(offset) - fractionalPart + random2f(integerPart + offset);
            float distance = dot(r, r);

            if (d < res.x) {
                res.y = res.x;
                res.x = distance;
            }
            else if (d < res.y) {
                res.y = distance;
            }
        }
    }

    return sqrt(res);
}

void mainImage(out vec4 fragColor, in vec2 fragCoord) {
    vec2 p = fragCoord / iResolution.xx;

    vec3 c = voronoi(8.0 * p);

    // feature points
    float dd = length(c.yz);

    // isolines
    vec3 col = c.x * (0.5 + 0.5 * sin(64.0 * c.x)) * vec3(1.0);
    // borders
    col = mix(vec3(1.0, 0.6, 0.0), col, smoothstep(0.04, 0.07, c.x));
    col = mix(vec3(1.0, 0.6, 0.1), col, smoothstep(0.0, 0.12, dd));
    col += vec3(1.0, 0.6, 0.1) * (1.0 - smoothstep(0.0, 0.04, dd));

    fragColor = vec4(col, 1.0);
}
