use super::spectrum::SPECTRUM_WINDOW_SIZE;
use crate::egui::epaint::Vertex;
use crate::egui::{Painter, Stroke};
use crate::gui::prelude::*;
use crate::gui::spectrum_ui::spectrum::{SpectrumInput, SpectrumOutput};
use crate::utils::interleave_pos2_vec;
use nih_plug::prelude::*;
use std::sync::{Arc, Mutex};

const CURVE_RESOLUTION: usize = WINDOW_SIZE.x as usize;
const BIN_SAMPLING_GAP: usize = 3;

/// MIDI note value for 10 Hz.
const NOTE_10_HZ: f32 = 3.486_820_576_35;
/// MIDI note value for 30,000 Hz.
const NOTE_30000_HZ: f32 = 142.095_782_001;

fn freq_to_xpos(freq: f32) -> f32 {
    utils::map(
        utils::freq_to_note(freq),
        NOTE_10_HZ,
        NOTE_30000_HZ,
        0.0,
        WINDOW_SIZE.x,
    )
}

fn xpos_to_freq(x: f32) -> f32 {
    utils::note_to_freq(utils::map(
        x, 0.0, WINDOW_SIZE.x, NOTE_10_HZ, NOTE_30000_HZ,
    ))
}

fn gain_to_ypos(gain: f32) -> f32 {
    WINDOW_SIZE.y - (gain + 45.0) * 6.0
}

/// Implements a `compute` method for computing spectral
/// information from an audio buffer frame, and a `ui`
/// method for drawing the spectrum analyser.
pub struct SpectrumAnalyser {
    sample_rate: Arc<AtomicF32>,

    bin_step: f32,
    /// This struct manages the spectral processing
    /// and input.
    pub spectrum: SpectrumInput,
    /// This is the output of the triple buffer, which
    /// receives spectral data after the `spectrum`
    /// field has computed a buffer frame.
    spectrum_output: Arc<Mutex<SpectrumOutput>>,

    /// container for the sampled bin information
    bin_points: Vec<[f32; 2]>,
    /// container for the interpolated points
    interpolated: Vec<[f32; 2]>,
    /// container for the decimated spectrum gui points
    spectrum_line: Vec<Pos2>,
    /// the filled part of the spectrum
    spectrum_mesh: Mesh,
    /// dark gradient over the bottom of the spectrum mesh
    overlay_mesh: Mesh,
}

impl SpectrumAnalyser {
    pub fn new(
        input: SpectrumInput,
        spectrum: SpectrumOutput,
        sample_rate: f32,
    ) -> Self {
        let num_sampled_points = WINDOW_SIZE.x as usize / BIN_SAMPLING_GAP;

        let overlay_height = WINDOW_SIZE.y * 0.7;
        let overlay_bottom_color = COLOR_UI_BLACK.linear_multiply(0.9);
        let overlay_mesh = Mesh {
            indices: vec![0, 1, 3, 3, 1, 2],
            vertices: vec![
                Vertex {
                    pos: pos2(0.0, WINDOW_SIZE.y - overlay_height),
                    uv: Pos2::ZERO,
                    color: Color32::TRANSPARENT,
                },
                Vertex {
                    pos: pos2(WINDOW_SIZE.x, WINDOW_SIZE.y - overlay_height),
                    uv: Pos2::ZERO,
                    color: Color32::TRANSPARENT,
                },
                Vertex {
                    pos: pos2(WINDOW_SIZE.x, WINDOW_SIZE.y),
                    uv: Pos2::ZERO,
                    color: overlay_bottom_color,
                },
                Vertex {
                    pos: pos2(0.0, WINDOW_SIZE.y),
                    uv: Pos2::ZERO,
                    color: overlay_bottom_color,
                },
            ],
            texture_id: Default::default(),
        };

        Self {
            sample_rate: Arc::new(AtomicF32::new(sample_rate)),
            bin_step: sample_rate / SPECTRUM_WINDOW_SIZE as f32,

            spectrum: input,
            spectrum_output: Arc::new(Mutex::new(spectrum)),

            // all the sampled bins
            bin_points: (0..num_sampled_points)
                .map(|i| {
                    let xpos =
                        (i as f32 / num_sampled_points as f32) * WINDOW_SIZE.x;
                    [xpos_to_freq(xpos), util::MINUS_INFINITY_DB]
                })
                .collect(),

            // all the interpolated visual points
            interpolated: (0..CURVE_RESOLUTION)
                .map(|i| {
                    let xpos =
                        (i as f32 / CURVE_RESOLUTION as f32) * WINDOW_SIZE.x;
                    [xpos, WINDOW_SIZE.y]
                })
                .collect(),

            // the array of decimated points
            spectrum_line: Vec::with_capacity(CURVE_RESOLUTION),
            spectrum_mesh: Mesh::default(),
            overlay_mesh,
        }
    }

    pub fn update_audio_config(
        &mut self,
        sample_rate: f32,
        num_channels: usize,
    ) {
        self.sample_rate = Arc::new(AtomicF32::new(sample_rate));
        self.bin_step = sample_rate / SPECTRUM_WINDOW_SIZE as f32;
        self.spectrum.update_sample_rate(sample_rate);
        self.spectrum.update_num_channels(num_channels);
    }

    pub fn compute(&mut self, buffer: &mut Buffer) {
        self.spectrum.compute(buffer);
    }

    pub fn ui(&mut self, painter: &Painter) {
        self.compute_spectrum();

        self.draw_spectrum_mesh(painter);
        self.draw_overlay_mesh(painter);
        self.draw_spectrum_line(painter);
    }

    fn draw_spectrum_line(&mut self, painter: &Painter) {
        let line = Shape::line(
            self.spectrum_line.clone(),
            Stroke::new(1.5, COLOR_UI_WHITE.linear_multiply(0.8)),
        );

        painter.add(line);
    }

    fn draw_spectrum_mesh(&mut self, painter: &Painter) {
        let mut points =
            vec![pos2(-5.0, gain_to_ypos(util::MINUS_INFINITY_DB))];
        points.append(&mut self.spectrum_line.clone());
        points.push(pos2(WINDOW_SIZE.x + 50.0, 3000.0));

        painter.add(Shape::mesh(Mesh {
            indices: earcut(&interleave_pos2_vec(&points), &[], 2)
                .unwrap()
                .iter()
                .map(|x| *x as u32)
                .collect(),
            vertices: points
                .iter()
                .map(|point| Vertex {
                    pos: *point,
                    uv: Pos2::ZERO,
                    color: Color32::from_rgb(168, 103, 85)
                        .linear_multiply(0.30),
                })
                .collect(),
            texture_id: Default::default(),
        }));
    }

    fn draw_overlay_mesh(&mut self, painter: &Painter) {
        painter.add(self.overlay_mesh.clone());
    }

    // fn mesh_color(&self, y: f32) -> Color32 {
    //     let max = self.spectrum_gradient.num_points();
    //     let range = max as f32;
    //     let y = utils::map(y, WINDOW_SIZE.y, 0.0, 0.0, range);
    //
    //     let floor = y.floor();
    //     let up = (floor as usize + 1).clamp(0, max);
    //     let lw = (floor as usize).clamp(0, max);
    //     let t = y - floor;
    //
    //     Gradient::lerp_color_gamma(
    //         self.spectrum_gradient[lw],
    //         self.spectrum_gradient[up],
    //         t,
    //     )
    // }

    fn bin_idx_t(freq: f32, step: f32) -> (usize, f32) {
        // let _bin_hz = |idx: usize| -> f32 { step * idx as f32 };
        let idx_exact = freq / step;
        let idx = idx_exact.floor();

        (idx as usize, idx_exact - idx)
    }

    #[allow(clippy::significant_drop_tightening)]
    fn compute_spectrum(&mut self) {
        let mut guard = self.spectrum_output.lock().unwrap();
        let mags = guard.read();

        /*        for (i, point) in self.bin_points.iter_mut().take(292).enumerate() {
                    // get the bin index and its offset from the pre-computed frequency
                    let (idx, t) = Self::bin_idx_t(point[0], self.bin_step);

                    // TODO: find when this happens and accommodate for it properly
                    if idx >= mags.len() - 1 {
                        dbg!(i);
                        break;
                    }

                    // linearly interpolate between the nearest two bins, such that
                    // arbitrary frequencies can be represented
                    let (lower_bin, upper_bin) = (mags[idx], mags[idx + 1]);
                    point[1] = (upper_bin - lower_bin).mul_add(t, lower_bin);
                }

                for (i, point) in self
                    .interpolated
                    .iter_mut()
                    .skip(BIN_SAMPLING_GAP)
                    .take(1167 - BIN_SAMPLING_GAP)
                    .enumerate()
                {
                    let i = i + BIN_SAMPLING_GAP;
                    let bin_point = i / BIN_SAMPLING_GAP;
                    dbg!(i, bin_point);

                    let (_, t) =
                        Self::bin_idx_t(xpos_to_freq(point[0]), self.bin_step);

                    let range_min = bin_point - 1;
                    let range_max = bin_point + 2;

                    let slice = &self.bin_points[range_min..=range_max];

                    point[1] = utils::cubic_catmull_slice_2(slice, t);

                    // let range_min = idx - 1; let range_max = idx + 2;
                    //
                    // if idx >= mags.len() {
                    //     dbg!(i);
                    //     break;
                    // }
                    //
                    // let slice = &self.bin_points[range_min..=range_max];
                    //
                    // point[1] = utils::cubic_catmull_db(slice, t);
                }
        */
        // even though spectrum_points is padded with 5 points off
        // either side of the screen, all of them are computed anyway
        // TODO check if this is necessary
        for (i, point) in self.interpolated.iter_mut().enumerate() {
            // set the x position
            let x = (i as f32 / (CURVE_RESOLUTION - 5) as f32) * WINDOW_SIZE.x;

            // get the bin index and its interpolation
            let (idx, t) = Self::bin_idx_t(xpos_to_freq(x), self.bin_step);

            if !(1..mags.len() - 2).contains(&idx) {
                *point = [x, util::MINUS_INFINITY_DB];
                continue;
            }

            let range_min = idx - 1;
            let range_max = idx + 2;

            let slice = &mags[range_min..=range_max];

            let mag = utils::cubic_catmull_db(slice, t);

            *point = [x, mag];
        }

        self.spectrum_line = decimate_points(&self.interpolated, 0.07)
            .iter()
            .map(|i| {
                pos2(
                    self.interpolated[*i][0],
                    gain_to_ypos(self.interpolated[*i][1]),
                )
            })
            .collect();
    }
}
