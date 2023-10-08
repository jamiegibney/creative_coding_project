use std::ops::Index;
use super::*;

pub fn vertex(
    ui: &mut Ui,
    bg_fill: Color32,
    gradient: &Gradient,
    size: Vec2,
) -> egui::Response {

    let (rect, response) = ui.allocate_at_least(size, Sense::hover());

    if bg_fill != Color32::default() {
        let mut mesh = Mesh::default();
        mesh.add_colored_rect(rect, bg_fill);
        ui.painter().add(Shape::mesh(mesh));
    }

    let n = gradient.num_points();
    assert!(n >= 2);
    let mut mesh = Mesh::default();

    for (i, &color) in gradient.points.iter().enumerate() {
        let t = i as f32 / (n as f32 - 1.0);
        let y = lerp(rect.y_range(), t);

        mesh.colored_vertex(pos2(rect.left(), y), color);
        mesh.colored_vertex(pos2(rect.right(), y), color);

        if i < n - 1 {
            let i = i as u32;

            mesh.add_triangle(2 * i, 2 * i + 1, 2 * i + 2);
            mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
        }
    }

    ui.painter().add(Shape::mesh(mesh));

    response
}

/// A simple two-point colour gradient.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Gradient {
    pub points: Vec<Color32>,
}

impl Index<usize> for Gradient {
    type Output = Color32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.points[index]
    }
}

impl Gradient {
    #[must_use]
    pub fn one_color(srgba: Color32) -> Self {
        Self { points: vec![srgba, srgba] }
    }

    #[must_use]
    pub fn endpoints(start: Color32, end: Color32) -> Self {
        Self { points: vec![start, end] }
    }

    #[must_use]
    pub fn linear_gradient_from(colors: &[Color32], num_points: usize) -> Self {
        let num_colors = colors.len();
        let increment = num_points / num_colors;
        Self {
            points: (0..num_points)
                .map(|i| {
                    let t = i as f32 / (num_points - 1) as f32;
                    let lw = (i / increment).clamp(0, num_colors - 1);
                    let up = (i / increment + 1).clamp(0, num_colors - 1);
                    Self::lerp_color_gamma(colors[lw], colors[up], t)
                })
                .collect(),
        }
    }

    /// A gradient of two *opaque* colours via linear interpolation (smoother).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn linear_gradient(
        start: Color32,
        end: Color32,
        num_points: usize,
    ) -> Self {
        let start = Rgba::from(start);
        let end = Rgba::from(end);

        Self {
            points: (0..num_points)
                .map(|i| {
                    let t = i as f32 / (num_points - 1) as f32;
                    Color32::from(lerp(start..=end, t))
                })
                .collect(),
        }
    }

    /// A gradient of two *opaque* colours via gamma interpolation (standard).
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn gamma_gradient(
        start: Color32,
        end: Color32,
        num_points: usize,
    ) -> Self {
        Self {
            points: (0..num_points)
                .map(|i| {
                    let t = i as f32 / (num_points - 1) as f32;
                    Self::lerp_color_gamma(start, end, t)
                })
                .collect(),
        }
    }

    /// Enable the gradient to be alpha-aware on top of a background colour.
    #[must_use]
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_lossless,
        clippy::cast_precision_loss
    )]
    pub fn alpha_aware_on_bg(self, bg: Color32) -> Self {
        let num_points = self.points.len();

        Self {
            points: self
                .points
                .into_iter()
                .map(|fg| {
                    let a = fg.a() as f32 / num_points as f32;

                    Color32::from_rgba_premultiplied(
                        f32::from(bg[0])
                            .mul_add(1.0 - a, f32::from(fg[0]))
                            .round() as u8,
                        f32::from(bg[1])
                            .mul_add(1.0 - a, f32::from(fg[1]))
                            .round() as u8,
                        f32::from(bg[2])
                            .mul_add(1.0 - a, f32::from(fg[2]))
                            .round() as u8,
                        f32::from(bg[3])
                            .mul_add(1.0 - a, f32::from(fg[3]))
                            .round() as u8,
                    )
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn to_pixel_row(&self) -> Vec<Color32> {
        self.points.clone()
    }

    #[must_use]
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn lerp_color_gamma(start: Color32, end: Color32, t: f32) -> Color32 {
        Color32::from_rgba_premultiplied(
            lerp(f32::from(start[0])..=f32::from(end[0]), t).round() as u8,
            lerp(f32::from(start[1])..=f32::from(end[1]), t).round() as u8,
            lerp(f32::from(start[2])..=f32::from(end[2]), t).round() as u8,
            lerp(f32::from(start[3])..=f32::from(end[3]), t).round() as u8,
        )
    }
}
