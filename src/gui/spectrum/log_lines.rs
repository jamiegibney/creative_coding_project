use crate::gui::gradient;
use crate::gui::prelude::*;
use egui::{lerp, pos2, vec2, Mesh, Rect, Shape};

/// Log values intended to represent the logarithmic
/// scaling from 10 Hz to 30 kHz.
#[allow(
    clippy::unreadable_literal,
    clippy::excessive_precision,
    clippy::approx_constant,
)]
#[rustfmt::skip]
pub const LOG_10_VALUES: [f32; 30] = [
    0.0,
    0.301029995664,
    0.47712125472,
    0.602059991328,
    0.698970004336,
    0.778151250384,
    0.845098040014,
    0.903089986992,
    0.954242509439,
    1.0,
    1.301029995664,
    1.47712125472,
    1.602059991328,
    1.698970004336,
    1.778151250384,
    1.845098040014,
    1.903089986992,
    1.954242509439,
    2.0,
    2.301029995664,
    2.47712125472,
    2.602059991328,
    2.698970004336,
    2.778151250384,
    2.845098040014,
    2.903089986992,
    2.954242509439,
    3.0,
    3.301029995664,
    3.47712125472,
];

pub fn log_lines(line_width: f32, centre_line: f32, height: f32) -> Vec<Shape> {
    let mut vec = Vec::with_capacity(LOG_10_VALUES.len() * 2);
    let max = *LOG_10_VALUES.last().unwrap();
    let height = height * 0.5;

    for (i, &x) in LOG_10_VALUES.iter().enumerate() {
        // every 9th log line, starting from the 0th, is a power of 10
        let is_bold = i % 9 == 0;
        let x_pos = WINDOW_SIZE.x * x / max;
        let height = if is_bold { height } else { height * 0.8 };

        // TODO figure out proper positioning
        let top_rect = Rect::from_center_size(
            pos2(x_pos, centre_line - height / 2.0),
            vec2(line_width, height),
        );
        let bottom_rect = Rect::from_center_size(
            pos2(x_pos, centre_line + height / 2.0),
            vec2(line_width, height),
        );

        let color = if is_bold { COLOR_LOG_LINE_BOLD } else { COLOR_LOG_LINE };

        let grad = gradient::Gradient::linear_gradient(
            COLOR_UI_BLACK, color, height as usize,
        );

        let (mut top_mesh, mut bottom_mesh) =
            (Mesh::default(), Mesh::default());

        let n = grad.num_points() / 3 * 2;
        for (i, &color) in grad.points.iter().enumerate() {
            let t = i as f32 / (n - 1) as f32;
            let y1 = lerp(top_rect.y_range(), t);
            let y2 = lerp(bottom_rect.y_range(), 1.0 - t);

            top_mesh.colored_vertex(pos2(top_rect.left(), y1), color);
            top_mesh.colored_vertex(pos2(top_rect.right(), y1), color);
            bottom_mesh.colored_vertex(pos2(bottom_rect.left(), y2), color);
            bottom_mesh.colored_vertex(pos2(bottom_rect.right(), y2), color);

            if i >= n - 1 {
                continue;
            }

            let i = i as u32;

            top_mesh.add_triangle(2 * i, 2 * i + 1, 2 * i + 2);
            top_mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
            bottom_mesh.add_triangle(2 * i, 2 * i + 1, 2 * i + 2);
            bottom_mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
        }

        vec.push(Shape::mesh(top_mesh));
        vec.push(Shape::mesh(bottom_mesh));
    }

    vec
}
