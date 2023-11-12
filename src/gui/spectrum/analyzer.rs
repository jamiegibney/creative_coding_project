use super::{process::RESULT_BUFFER_SIZE, *};
use crate::{dsp::*, gui::rdp::decimate_points, prelude::*};
use nannou::prelude::*;
use std::ptr::{addr_of, copy_nonoverlapping};

const NUM_SPECTRUM_AVERAGES: usize = 12;

const CURVE_RESOLUTION: usize = WINDOW_SIZE.x as usize;

/// MIDI note value for 10 Hz.
const NOTE_10_HZ: f64 = 3.486_820_576_35;
/// MIDI note value for 30,000 Hz.
const NOTE_30000_HZ: f64 = 142.095_782_001;

pub struct SpectrumAnalyzer {
    bin_step: f64,

    /// The spectral data received from a corresponding `SpectrumInput`.
    spectrum: SpectrumOutput,

    spectrum_averaging: Vec<Vec<f64>>,
    averaging_write_pos: usize,

    averaged_data: Vec<f64>,

    /// The frequency bin points.
    bin_points: Vec<[f64; 2]>,

    /// All the interpolated points used for the spectrum.
    interpolated: Vec<[f64; 2]>,

    /// The spectrum line.
    spectrum_line: Vec<DVec2>,

    /// The bounding box of the analyzer.
    rect: Rect,

    filter: FirstOrderFilter,
    // spectrum_mesh: Vec<[f32; 2]>,
}

impl SpectrumAnalyzer {
    pub fn new(spectrum: SpectrumOutput, rect: Rect) -> Self {
        let width = rect.w() as f64;
        let sample_rate = unsafe { SAMPLE_RATE };

        let mut filter = FirstOrderFilter::new(10.0);
        filter.set_type(FilterType::Lowpass);
        filter.set_freq(1.0);

        Self {
            spectrum,

            spectrum_averaging: vec![
                vec![0.0; RESULT_BUFFER_SIZE];
                NUM_SPECTRUM_AVERAGES
            ],
            averaging_write_pos: 0,

            averaged_data: vec![0.0; RESULT_BUFFER_SIZE],

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

            filter,
        }
    }

    /// Draws the spectrum. If `None` is passed to either `line_color` or `mesh_color`, those
    /// parts of the spectum will not be computed, saving processing time. If both are `None`
    /// (for some reason), then the spectrum visual is not computed.
    pub fn draw(
        &mut self,
        draw: &Draw,
        line_color: Option<Rgba>,
        mesh_color: Option<Rgba>,
    ) {
        if line_color.is_some() || mesh_color.is_some() {
            self.compute_spectrum();
        }
        if let Some(color) = mesh_color {
            self.draw_mesh(draw, color);
        }
        if let Some(color) = line_color {
            self.draw_line(draw, color);
        }
    }

    fn draw_line(&mut self, draw: &Draw, color: Rgba) {
        draw.polyline().weight(2.5).points_colored(
            self.spectrum_line.iter().map(|x| (x.as_f32(), color)),
        );
    }

    fn draw_mesh(&mut self, draw: &Draw, color: Rgba) {
        let start_point = dvec2(
            -self.width(),
            gain_to_ypos(MINUS_INFINITY_DB, self.height()),
        );
        let end_point = dvec2(self.width() + 50.0, -3000.0);

        let mut points = vec![dvec2(0.0, 0.0); self.spectrum_line.len() + 2];

        unsafe {
            let ptr = points.as_mut_ptr();
            copy_nonoverlapping(addr_of!(start_point), ptr, 1);
            copy_nonoverlapping(
                self.spectrum_line.as_ptr(),
                ptr.add(1),
                self.spectrum_line.len(),
            );
            copy_nonoverlapping(
                addr_of!(end_point),
                ptr.add(self.spectrum_line.len() + 1),
                1,
            );
        }

        let indices =
            earcutr::earcut(&interleave_dvec2_to_f64(&points), &[], 2).unwrap();

        draw.mesh().indexed_colored(
            points.iter().map(|x| (x.extend(0.0).as_f32(), color)),
            indices,
        );
    }

    fn compute_spectrum(&mut self) {
        let width = self.width();
        // first, average the spectrum data...
        self.average_spectrum_data();

        // then get an interpolated magnitude value for each point along the curve.
        for (i, pt) in self.interpolated.iter_mut().enumerate() {
            let x = (i as f64 / (CURVE_RESOLUTION - 5) as f64) * width;

            // get the frequency bin index and its offset
            let (idx, interp) =
                Self::bin_idx_t(xpos_to_freq(x, width), self.bin_step);

            // points outside of the working range are set to -inf dB
            if !(1..self.averaged_data.len() - 2).contains(&idx) {
                *pt = [x, MINUS_INFINITY_DB];
                continue;
            }

            // get a slice of 4 points used for cubic interpolation...
            let slice = &self.averaged_data[(idx - 1)..=(idx + 2)];

            // catmull-rom interpolation is used to give a lovely smoothed shape
            // to the magnitude response.
            let mut mag = cubic_catmull_db(slice, interp);

            // the magnitude is then clamped to -inf dB.
            if mag < MINUS_INFINITY_DB {
                mag = MINUS_INFINITY_DB;
            }

            *pt = [x, mag];
        }

        // bit of a hack here - the filter processes 10 samples but the values aren't
        // used because it helps to "prime" the filter, smoothing out the curve at
        // the very lowest frequencies. this needs to be done because the filter is
        // reset for each frame, so has a little "blip" at DC.
        // ideally the blip would be off screen, but this works well enough and the
        // discontinuity is difficult to spot unless you're looking for it.
        for i in 0..10 {
            self.filter.process_mono(self.interpolated[i][1]);
        }
        self.interpolated.iter_mut().skip(10).for_each(|x| {
            x[1] = self.filter.process_mono(x[1]);
        });

        // reset the filter for the next frame
        self.filter.reset();

        // decimate points to optimise the line geometry for rendering.
        // TODO: add a version of this function which mutates a buffer in-place,
        // rather than creating a new vector each time.
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

    // TODO this could probably be more efficient?
    fn average_spectrum_data(&mut self) {
        // add the new set of magnitudes to the averaging buffers
        let mags = self.spectrum.read();
        self.spectrum_averaging[self.averaging_write_pos]
            .iter_mut()
            .enumerate()
            .for_each(|(i, x)| *x = mags[i]);

        // increment the write pos for next time
        self.increment_averaging_pos();

        let frames = NUM_SPECTRUM_AVERAGES;

        // iterate through each sample...
        for smp in 0..RESULT_BUFFER_SIZE {
            // and sum up all elements from each frame
            self.averaged_data[smp] = level_to_db(
                (0..frames)
                    .map(|fr| {
                        // normalise each value
                        self.spectrum_averaging[fr][smp] / frames as f64
                    })
                    .sum(),
            );
        }
    }

    fn increment_averaging_pos(&mut self) {
        self.averaging_write_pos =
            (self.averaging_write_pos + 1) % NUM_SPECTRUM_AVERAGES;
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
    (gain + 90.0).mul_add(7.0, -height)
}

fn cubic_catmull_db(points: &[f64], t: f64) -> f64 {
    debug_assert!(points.len() >= 4);
    interp::cubic_catmull(points[0], points[1], points[2], points[3], t)
}
