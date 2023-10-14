use super::*;
use crate::dsp::*;
use crate::{gui::rdp::decimate_points, prelude::*};
use nannou::prelude::*;
use std::ptr::{addr_of, copy};
// use std::sync::{Arc, Mutex};

const CURVE_RESOLUTION: usize = WINDOW_SIZE.x as usize;

/// MIDI note value for 10 Hz.
const NOTE_10_HZ: f64 = 3.486_820_576_35;
/// MIDI note value for 30,000 Hz.
const NOTE_30000_HZ: f64 = 142.095_782_001;

pub struct SpectrumAnalyzer {
    bin_step: f64,

    /// The spectral data received from a corresponding `SpectrumInput`.
    spectrum: SpectrumOutput,

    /// The frequency bin points.
    bin_points: Vec<[f64; 2]>,

    /// All the interpolated points used for the spectrum.
    interpolated: Vec<[f64; 2]>,

    /// The spectrum line.
    spectrum_line: Vec<DVec2>,

    /// The bounding box of the analyzer.
    rect: Rect,
    // spectrum_mesh: Vec<[f32; 2]>,
}

impl SpectrumAnalyzer {
    pub fn new(spectrum: SpectrumOutput, rect: Rect) -> Self {
        let width = rect.w() as f64;
        let sample_rate = unsafe { SAMPLE_RATE };

        Self {
            spectrum,

            bin_step: sample_rate / SPECTRUM_WINDOW_SIZE as f64,

            bin_points: (0..width as usize)
                .map(|i| {
                    let xpos = (i as f64 / width) * width;
                    [xpos_to_freq(xpos, width), MINUS_INFINITY_DB]
                })
                .collect(),

            interpolated: (0..CURVE_RESOLUTION)
                .map(|i| {
                    let xpos = (i as f64 / CURVE_RESOLUTION as f64) * width;
                    [xpos, rect.h() as f64]
                })
                .collect(),

            spectrum_line: Vec::with_capacity(CURVE_RESOLUTION),

            rect,
        }
    }

    pub fn draw(&mut self, draw: &Draw) {
        self.compute_spectrum();
        self.draw_line(draw);
        // self.draw_mesh(draw);
    }

    fn draw_line(&mut self, draw: &Draw) {
        draw.polyline().weight(1.5).points_colored(
            self.spectrum_line.iter().map(|x| (x.as_f32(), WHITE)),
        );
    }

    fn draw_mesh(&mut self, draw: &Draw) {
        let start_point =
            dvec2(-5.0, gain_to_ypos(MINUS_INFINITY_DB, self.height()));
        let end_point = dvec2(self.width() + 50.0, 3000.0);

        let mut points = Vec::with_capacity(self.spectrum_line.len() + 2);

        unsafe {
            copy(addr_of!(start_point), points.as_mut_ptr(), 1);
            copy(
                self.spectrum_line.as_ptr(),
                points.as_mut_ptr().add(1),
                self.spectrum_line.len(),
            );
            copy(
                addr_of!(end_point),
                points.as_mut_ptr().add(self.spectrum_line.len() + 1),
                1,
            );
        }

        let indices =
            earcutr::earcut(&interleave_dvec2_to_f64(&points), &[], 2).unwrap();

        draw.mesh()
            .indexed_colored(
                points.iter().map(|x| (x.extend(0.0).as_f32(), GREY)),
                indices,
            )
            .finish();
    }

    fn compute_spectrum(&mut self) {
        let width = self.width();
        let mags = self.spectrum.read();

        for (i, pt) in self.interpolated.iter_mut().enumerate() {
            let x = (i as f64 / (CURVE_RESOLUTION - 5) as f64) * width;

            let (idx, interp) =
                Self::bin_idx_t(xpos_to_freq(x, width), self.bin_step);

            if !(1..mags.len() - 2).contains(&idx) {
                *pt = [x, MINUS_INFINITY_DB];
                continue;
            }

            let range_min = idx - 1;
            let range_max = idx + 2;

            let slice = &mags[range_min..=range_max];

            // TODO: so much recalculation going on here which could be cached!
            let mut mag = cubic_catmull_db(slice, interp);

            if mag <= MINUS_INFINITY_DB {
                mag = MINUS_INFINITY_DB;
            }

            *pt = [x, mag];
        }

        // decimation removes about 1/3 of the total points here.
        // TODO: could the decimate_points() function mutate a buffer in-place?
        self.spectrum_line = decimate_points(&self.interpolated, 0.1)
            .iter()
            .map(|i| {
                let mut x = self.interpolated[*i][0] - width / 2.0;
                let mut y =
                    gain_to_ypos(self.interpolated[*i][1], self.height());
                // TODO: find out why some of these points are not finite
                if !x.is_finite() {
                    x = width / 2.0;
                }
                if !y.is_finite() {
                    y = self.height() / 2.0;
                }
                DVec2::new(x, y)
            })
            .collect();
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
}

fn freq_to_xpos(freq: f64, width: f64) -> f64 {
    let half = width / 2.0;
    map(freq_to_note(freq), NOTE_10_HZ, NOTE_30000_HZ, -half, half)
}

fn xpos_to_freq(x: f64, width: f64) -> f64 {
    // let half = width / 2.0;
    note_to_freq(map(x, 0.0, width, NOTE_10_HZ, NOTE_30000_HZ))
}

fn gain_to_ypos(gain: f64, height: f64) -> f64 {
    (gain + 85.0).mul_add(8.0, -height)
}

fn cubic_catmull_db(points: &[f64], t: f64) -> f64 {
    debug_assert!(points.len() >= 4);

    interp::cubic_catmull(
        level_to_db(points[0]),
        level_to_db(points[1]),
        level_to_db(points[2]),
        level_to_db(points[3]),
        t,
    )
}
