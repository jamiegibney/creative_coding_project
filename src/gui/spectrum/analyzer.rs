use super::{process::RESULT_BUFFER_SIZE, *};
use crate::{dsp::*, gui::rdp::rdp_in_place, prelude::*};
use nannou::prelude::*;
use std::ptr::{addr_of, copy_nonoverlapping};

const NUM_SPECTRUM_AVERAGES: usize = 10;

const MIN_FREQ: f64 = 25.0;
const MAX_FREQ: f64 = 22_050.0;

pub struct SpectrumAnalyzer {
    bin_step: f64,

    /// The spectral data received from a corresponding `SpectrumInput`.
    spectrum: SpectrumOutput,

    spectrum_averaging: Vec<Vec<f64>>,
    averaging_write_pos: usize,

    averaged_data: Vec<f64>,

    /// All the interpolated points used for the spectrum.
    interpolated: Vec<[f64; 2]>,

    /// The spectrum line.
    spectrum_line: Vec<DVec2>,

    /// The bounding box of the analyzer.
    rect: Rect,

    /// A filter for smoothing out the spectrum line.
    filter: FirstOrderFilter,

    mesh_points: Vec<DVec2>,
    indices: Vec<usize>,
}

impl SpectrumAnalyzer {
    pub fn new(spectrum: SpectrumOutput, rect: Rect) -> Self {
        let width = rect.w() as f64;
        let resolution = width as usize;
        let sample_rate = unsafe { SAMPLE_RATE };

        let mut filter = FirstOrderFilter::new(10.0);
        filter.set_type(FilterType::Lowpass);
        filter.set_freq(1.0);

        let vec: Vec<[f64; 2]> = (0..resolution)
            .map(|i| {
                let xpos = (i as f64 / resolution as f64) * width;
                [xpos, rect.bottom() as f64]
            })
            .collect();

        Self {
            spectrum,

            spectrum_averaging: vec![vec![0.0; RESULT_BUFFER_SIZE]; NUM_SPECTRUM_AVERAGES],
            averaging_write_pos: 0,

            averaged_data: vec![0.0; RESULT_BUFFER_SIZE],

            bin_step: sample_rate / SPECTRUM_WINDOW_SIZE as f64,

            interpolated: vec.clone(),

            spectrum_line: vec.iter().map(|p| DVec2::from_slice(p)).collect(),
            indices: (0..vec.len()).collect(),
            rect,

            mesh_points: {
                let mut v: Vec<DVec2> = vec.iter().map(|p| DVec2::from_slice(p)).collect();
                // add the extra elements which allow the mesh to be pinned to the
                // bottom corners.
                v.push(rect.bottom_right().as_f64());
                v.push(rect.bottom_left().as_f64());
                v.rotate_right(1);

                v
            },

            filter,
        }
    }

    /// Returns a reference to the bounding rect of the analyzer.
    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    /// Draws the spectrum. If `None` is passed to either `line_color` or `mesh_color`, those
    /// parts of the spectum will not be computed, saving processing time. If both are `None`
    /// (for some reason), then the spectrum visual is not computed.
    pub fn draw(
        &mut self,
        draw: &Draw,
        line_color: Option<Rgba>,
        line_weight: f32,
        mesh_color: Option<Rgba>,
    ) {
        if line_color.is_some() || mesh_color.is_some() {
            self.compute_spectrum();
        }
        if let Some(color) = mesh_color {
            self.draw_mesh(draw, color);
        }
        if let Some(color) = line_color {
            self.draw_line(draw, color, line_weight);
        }
    }

    fn draw_line(&mut self, draw: &Draw, color: Rgba, weight: f32) {
        draw.polyline()
            .weight(weight)
            .points_colored(self.spectrum_line.iter().map(|x| (x.as_f32(), color)));
    }

    fn draw_mesh(&mut self, draw: &Draw, color: Rgba) {
        let br_point = self.rect.bottom_right().as_f64();

        unsafe {
            let ptr = self.mesh_points.as_mut_ptr();
            let len = self.spectrum_line.len();

            // we can then truncate the length of the mesh buffer to ignore decimated points.
            self.mesh_points.set_len(self.spectrum_line.len() + 2);

            // we always ignore the first point as it is set to the bottom-left point.
            copy_nonoverlapping(self.spectrum_line.as_ptr(), ptr.add(1), len);
            // but we always need to add the bottom-right point here, as we don't know
            // how many points were decimated.
            copy_nonoverlapping(addr_of!(br_point), ptr.add(len + 1), 1);
        }

        // now we can get the triangulation indices
        let indices = earcutr::earcut(&interleave_dvec2_to_f64(&self.mesh_points), &[], 2).unwrap();

        // and draw the mesh :D
        draw.mesh().indexed_colored(
            self.mesh_points
                .iter()
                .map(|x| (x.extend(0.0).as_f32(), color)),
            indices,
        );
    }

    fn compute_spectrum(&mut self) {
        let width = self.width();
        let left = self.rect.left() as f64;

        // first, average the spectrum data...
        self.average_spectrum_data();

        // then get an interpolated magnitude value for each point along the curve.
        for (i, pt) in self.interpolated.iter_mut().enumerate() {
            // get the x coordinate
            let x = (i as f64 / width).mul_add(width, left);

            // get the frequency bin index and its interpolation value
            let (idx, interp) = Self::bin_idx_t(xpos_to_freq(&self.rect, x), self.bin_step);

            // points outside of the working range are set to -inf dB
            if !(1..self.averaged_data.len() - 2).contains(&idx) {
                *pt = [x, MINUS_INFINITY_DB];
                continue;
            }

            // get a slice of 4 points used for cubic interpolation
            let slice = &self.averaged_data[(idx - 1)..=(idx + 2)];

            // catmull-rom interpolation is used to produce a nicely smoothed path
            let mut mag = catmull_from_slice(slice, interp);

            // the magnitude is then clamped to -inf dB.
            if mag.is_nan() || mag < MINUS_INFINITY_DB {
                mag = MINUS_INFINITY_DB;
            }

            *pt = [x, mag];
        }

        // process the interpolated magnitude data with a low-pass filter to smooth it out.
        for _ in 0..16 {
            // this step ensures that the low-frequency points are consistent after filtering; the
            // filter will maintain the last high-frequency points it processes from the last frame,
            // so this effectively "flushes" that information and prepares it for the lower frequencies.
            let dc = self.interpolated[0][1];
            self.filter.process_mono(dc, 0);
        }
        self.interpolated.iter_mut().for_each(|x| {
            x[1] = self.filter.process_mono(x[1], 0);
        });

        // point decimation is performed here to remove unneeded points, speeding up
        // the rendering process.
        rdp_in_place(&self.interpolated, &mut self.indices, 0.01);
        // we need to truncate the length of this buffer so we ignore the unused elements.
        unsafe {
            self.spectrum_line.set_len(self.indices.len());
        }
        for (i, &idx) in self.indices.iter().enumerate() {
            self.spectrum_line[i] = {
                let point = self.interpolated[idx];

                let x = point[0];
                let y = self.gain_to_ypos(point[1]);

                dvec2(x, y)
            };
        }

        self.indices.clear();
    }

    fn average_spectrum_data(&mut self) {
        // copy the new set of magnitudes to the averaging buffers
        let mags = self.spectrum.read();
        self.spectrum_averaging[self.averaging_write_pos].copy_from_slice(mags);

        // increment the write pos for next time
        self.increment_averaging_pos();

        let frames = NUM_SPECTRUM_AVERAGES;
        let norm = 1.0 / frames as f64;

        // iterate through each sample...
        for smp in 0..RESULT_BUFFER_SIZE {
            // and sum up all elements from each frame
            self.averaged_data[smp] = level_to_db(
                (0..frames)
                    .map(|fr| {
                        // normalise each value
                        self.spectrum_averaging[fr][smp] * norm
                    })
                    .sum(),
            );
        }
    }

    fn increment_averaging_pos(&mut self) {
        self.averaging_write_pos = (self.averaging_write_pos + 1) % NUM_SPECTRUM_AVERAGES;
    }

    fn bin_idx_t(freq: f64, step: f64) -> (usize, f64) {
        let idx_exact = freq / step;
        let idx = idx_exact.floor();

        (idx as usize, idx_exact - idx)
    }

    fn width(&self) -> f64 {
        self.rect.w() as f64
    }

    fn height(&self) -> f64 {
        self.rect.h() as f64
    }

    fn gain_to_ypos(&self, gain: f64) -> f64 {
        let bottom = self.rect.bottom() as f64;
        let top = self.rect.top() as f64;

        (gain * 3.0 + bottom - bottom * 0.3).clamp(bottom, top)
    }
}

/// Transposes an x (horizontal) value within a rectangle to a logarithmically-scaled
/// frequency value.
fn xpos_to_freq(rect: &Rect, x: f64) -> f64 {
    let norm = normalize(x, rect.left() as f64, rect.right() as f64);
    freq_lin_from_log(norm, MIN_FREQ, MAX_FREQ * 2.0)
}

/// Transposes a frequency value to an x (horizontal) value logarithmically-scaled
/// within a rectangle.
fn freq_to_xpos(rect: &Rect, freq_hz: f64) -> f64 {
    let left = rect.left() as f64;
    let width = rect.w() as f64;

    // MAX_FREQ * 2 as it acts as the sampling rate, which gets halved anyway
    freq_log_norm(freq_hz, MIN_FREQ, MAX_FREQ * 2.0).mul_add(width, left)
}

fn catmull_from_slice(points: &[f64], t: f64) -> f64 {
    debug_assert!(points.len() >= 4);
    interp::cubic_catmull(points[0], points[1], points[2], points[3], t)
}
