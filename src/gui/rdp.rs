/// A struct for tracking the length of (and distance from) a line.
struct LineDistance {
    a: f64,
    b: f64,
    c: f64,
    length: f64,
}

impl LineDistance {
    /// Creates a new, initialised `LineDistance`.
    pub fn new(p1: &[f64; 2], p2: &[f64; 2]) -> Self {
        let [x1, y1] = p1;
        let [x2, y2] = p2;

        Self {
            a: y2 - y1,
            b: x2 - x1,
            c: x2 * y1 - y2 * x1,
            length: (y1 - y2).hypot(x1 - x2),
        }
    }

    /// Computes the distance from a point to the line.
    pub fn distance_to(&self, point: &[f64; 2]) -> Option<f64> {
        let Self { a, b, c, length } = self;
        let [x, y] = point;

        if *length == 0.0 {
            None
        }
        else {
            Some((a * x - b * y + c).abs() / length)
        }
    }
}

/// Casts `[f32; 2]` to `[f64; 2]` — used for better accuracy with very
/// small epsilon values and geometry calculations.
const fn cast_to_f64(arr: [f32; 2]) -> [f64; 2] {
    [arr[0] as f64, arr[1] as f64]
}

// TODO add a skewed version which alters its sensitivity as it iterates
//  through the slice, i.e. for decimating less/more points at the end

/// An (iterative) implementation of the [Ramer-Douglas-Peucker algorithm](https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm), which
/// decimates a set of points based on an epsilon value. Commonly used for
/// simplifying a path of points.
///
/// Returns a vector of all the indices to be retained, which always
/// includes the first and last elements.
#[allow(
    clippy::range_minus_one, clippy::option_if_let_else, clippy::cast_lossless
)]
#[must_use]
pub fn decimate_points(points: &[[f64; 2]], epsilon: f64) -> Vec<usize> {
    if points.len() <= 2 {
        return (0..points.len()).collect();
    }

    // each range refers to a range of points between two points,
    // represented as the start and end of each range
    // a capacity of 10 is used to cover the typical number of elements
    // used by the algorithm at a time
    let mut ranges = Vec::<std::ops::RangeInclusive<usize>>::with_capacity(10);

    // always keep index 0 — the start point
    let mut res = vec![0];

    // start with a range from the first to the last point
    ranges.push(0..=points.len() - 1);

    while let Some(range) = ranges.pop() {
        let start_index = *range.start();
        let end_index = *range.end();

        let start_point = points[start_index];
        let end_point = points[end_index];

        // the Line struct is mainly used to abstract some calculations here
        let line = LineDistance::new(&start_point, &end_point);

        // iterate through all the points *between* the start and end point...
        let (max_distance, max_index) =
            points[start_index + 1..end_index].iter().enumerate().fold(
                (0.0f64, 0),
                move |(max_distance, max_index), (index, point)| {
                    let distance = if let Some(dist) =
                        line.distance_to(point)
                    // IF the distance is not 0.0, use it
                    {
                        dist
                    }
                    // IF the distance is 0.0 (i.e. the point lies on the line), then
                    // calculate the distance from the start point
                    else {
                        let [sx, sy] = start_point;
                        let [px, py] = point;

                        (px - sx).hypot(py - sy)
                    };

                    // index + 1 is used because we start at the point AFTER the
                    // start point, so need to make up for that
                    if distance > max_distance {
                        (distance, index + 1)
                    }
                    else {
                        (max_distance, max_index)
                    }
                },
            );

        // if a point lies outside of the epsilon, divide the range and
        // process both segments separately
        if max_distance > epsilon {
            // split the line at the point of max distance
            let split_index = start_index + max_index;
            let first_segment = start_index..=split_index;
            let second_segment = split_index..=end_index;

            ranges.push(second_segment);
            // the first segment is pushed last to maintain the stack —
            // each range is popped from the vector per iteration
            ranges.push(first_segment);
        }
        else {
            res.push(end_index);
        }
    }

    res
}
