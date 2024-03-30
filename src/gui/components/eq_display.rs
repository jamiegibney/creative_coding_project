//! 3-band EQ UI component.

use super::*;
use crate::dsp::{BiquadParams, Filter, FilterType};
use crate::gui::rdp::rdp;
use crate::{app::UIParams, dsp::BiquadFilter};
use atomic_float::AtomicF64;
use std::f64::consts::SQRT_2;
use std::sync::{atomic::AtomicBool, Arc};

/// Used to determine the currently-clicked node.
#[derive(Clone, Copy, Debug)]
enum NodeType {
    Low,
    Peak,
    High,
}

/// Common filter params for each node.
#[derive(Clone, Debug)]
pub struct EQFilterParams {
    pub cutoff: f64,
    pub gain: f64,
    pub q: f64,
}

/// A struct for managing the display of the draggable filter nodes and the
/// filter magnitude response line.
#[derive(Clone, Debug)]
pub struct EQDisplay {
    /// The node position for the low filter.
    low_filter_node: Vec2,
    /// Whether the low filter is a shelf or not.
    low_filter_is_shelf: Arc<AtomicBool>,
    /// The general parameters of the low filter.
    pub low_filter_params: EQFilterParams,
    /// The low filter, used to compute the frequency response.
    low_filter: BiquadFilter,

    /// The node position for the peak filter.
    peak_filter_node: Vec2,
    /// The general parameters of the peak filter.
    pub peak_filter_params: EQFilterParams,
    /// The peak filter, used to compute the frequency response.
    peak_filter: BiquadFilter,

    /// The node position for the high filter.
    high_filter_node: Vec2,
    /// Whether the high filter is a shelf or not.
    high_filter_is_shelf: Arc<AtomicBool>,
    /// The general parameters of the high filter.
    pub high_filter_params: EQFilterParams,
    /// The high filter, used to compute the frequency response.
    high_filter: BiquadFilter,

    /// The currently-clicked node.
    clicked_node: Option<NodeType>,

    /// The raw x-pos and mangitude points.
    filter_raw_points: Vec<[f64; 2]>,
    /// The filter points to be drawn.
    filter_points: Vec<Vec2>,

    /// Whether the spectrum rect is clicked or not.
    pub spectrum_is_clicked: bool,
    /// Whether the mouse is alread clicked outside of the spectrum or not.
    pub clicked_outside_of_spectrum: bool,

    /// The bounding rect (i.e. the spectrum rect, for this device).
    rect: Rect,
    /// Reference to the device's sample rate.
    sample_rate: Arc<AtomicF64>,
}

impl EQDisplay {
    // in this case, rect is spectrum_rect
    pub fn new(
        rect: Rect,
        params: &UIParams,
        sample_rate: Arc<AtomicF64>,
    ) -> Self {
        let mid = rect.y();
        let left = rect.left();
        let right = rect.right();
        let len = rect.w() as usize;

        Self {
            low_filter_node: Vec2::new(-244.94064, -175.0),
            low_filter_is_shelf: Arc::clone(&params.low_filter_is_shelf),
            low_filter_params: EQFilterParams {
                cutoff: params.low_filter_cutoff.current_value(),
                gain: params.low_filter_gain_db.current_value(),
                q: params.low_filter_q.current_value().recip(),
            },
            low_filter: {
                let mut filter = BiquadFilter::new(sample_rate.lr());
                filter.set_params(&BiquadParams {
                    freq: params.low_filter_cutoff.current_value(),
                    gain: params.low_filter_gain_db.current_value(),
                    q: params.low_filter_q.current_value().recip(),
                    filter_type: if params.low_filter_is_shelf.lr() {
                        FilterType::Lowshelf
                    }
                    else {
                        FilterType::Highpass
                    },
                });
                filter
            },

            peak_filter_node: Vec2::new(-176.9, -175.0),
            peak_filter_params: EQFilterParams {
                cutoff: params.peak_filter_cutoff.current_value(),
                gain: params.peak_filter_gain_db.current_value(),
                q: params.peak_filter_q.current_value().recip(),
            },
            peak_filter: {
                let mut filter = BiquadFilter::new(sample_rate.lr());
                filter.set_params(&BiquadParams {
                    freq: params.peak_filter_cutoff.current_value(),
                    gain: params.peak_filter_gain_db.current_value(),
                    q: params.peak_filter_q.current_value().recip(),
                    filter_type: FilterType::Peak,
                });
                filter
            },

            high_filter_node: Vec2::new(-108.4, -175.0),
            high_filter_is_shelf: Arc::clone(&params.high_filter_is_shelf),
            high_filter_params: EQFilterParams {
                cutoff: params.high_filter_cutoff.current_value(),
                gain: params.high_filter_gain_db.current_value(),
                q: params.high_filter_q.current_value().recip(),
            },
            high_filter: {
                let mut filter = BiquadFilter::new(sample_rate.lr());
                filter.set_params(&BiquadParams {
                    freq: params.high_filter_cutoff.current_value(),
                    gain: params.high_filter_gain_db.current_value(),
                    q: params.high_filter_q.current_value().recip(),
                    filter_type: if params.high_filter_is_shelf.lr() {
                        FilterType::Highshelf
                    }
                    else {
                        FilterType::Lowpass
                    },
                });
                filter
            },

            clicked_node: None,

            filter_raw_points: (0..len)
                .map(|i| {
                    let x_pos = left as f64 + i as f64;
                    [x_pos, mid as f64]
                })
                .collect(),
            filter_points: vec![Vec2::ZERO; len],

            spectrum_is_clicked: false,
            clicked_outside_of_spectrum: false,

            rect,
            sample_rate,
        }
    }

    fn draw_filter_nodes(&self, draw: &Draw) {
        let node_color_1 = Rgba::new(0.9, 0.4, 0.0, 0.5);
        let node_color_2 = Rgba::new(0.4, 0.7, 0.0, 0.5);
        let node_color_3 = Rgba::new(0.0, 0.4, 0.9, 0.5);

        draw.ellipse()
            .xy(self.low_filter_node)
            .radius(7.0)
            .color(node_color_1);

        draw.ellipse()
            .xy(self.peak_filter_node)
            .radius(7.0)
            .color(node_color_1);

        draw.ellipse()
            .xy(self.high_filter_node)
            .radius(7.0)
            .color(node_color_1);
    }

    /// Updates the filter line based on the response of the internal filters.
    fn update_filter_line(&mut self) {
        let sr = self.sample_rate.lr();
        let (l, r) = (self.rect.left() as f64, self.rect.right() as f64);
        let (b, t) = (self.rect.bottom() as f64, self.rect.top() as f64);
        let w = self.rect.w() as f64;
        let half_height = self.rect.h() as f64 * 0.5; // marks +- 30 db
        let mid = self.rect.mid_left().y as f64;

        for (i, point) in self.filter_raw_points.iter_mut().enumerate().skip(1)
        {
            let x = i as f64 / w;
            let freq = freq_lin_from_log(x, 25.0, sr);
            let scaled = map(freq, 0.0, sr * 0.5, 0.0, PI_F64);

            let mag_db = self.low_filter.response_at(scaled)
                + self.peak_filter.response_at(scaled)
                + self.high_filter.response_at(scaled);

            point[1] = (mid
                + map(mag_db, -30.0, 30.0, -half_height, half_height))
            .clamp(b, t);
        }

        // copy second point to first, as the first is ignored.
        self.filter_raw_points[0][1] = self.filter_raw_points[1][1];

        // decimate redundant points — this reduces the number of points to be drawn by
        // over 10 times.
        let indices = rdp(self.filter_raw_points.as_slice(), 0.2);
        let len = indices.len();

        unsafe {
            self.filter_points.set_len(len);
        }

        for i in 0..len {
            let point = self.filter_raw_points[indices[i]];
            self.filter_points[i] =
                Vec2::from([point[0] as f32, point[1] as f32]);
        }
    }

    /// Updates the internal filters (and therefore the filter response line);
    fn update_filters(&mut self) {
        let lf_shelf = self.low_filter_is_shelf.lr();
        self.low_filter.set_params(&BiquadParams {
            freq: self.low_filter_params.cutoff,
            gain: self.low_filter_params.gain,
            q: if lf_shelf { SQRT_2 } else { self.low_filter_params.q },
            filter_type: if lf_shelf {
                FilterType::Lowshelf
            }
            else {
                FilterType::Highpass
            },
        });

        self.peak_filter.set_params(&BiquadParams {
            freq: self.peak_filter_params.cutoff,
            gain: self.peak_filter_params.gain,
            q: self.peak_filter_params.q,
            filter_type: FilterType::Peak,
        });

        let hf_shelf = self.high_filter_is_shelf.lr();
        self.high_filter.set_params(&BiquadParams {
            freq: self.high_filter_params.cutoff,
            gain: self.high_filter_params.gain,
            q: if hf_shelf { SQRT_2 } else { self.high_filter_params.q },
            filter_type: if hf_shelf {
                FilterType::Highshelf
            }
            else {
                FilterType::Lowpass
            },
        });

        self.low_filter.process(0.0);
        self.peak_filter.process(0.0);
        self.high_filter.process(0.0);
    }

    fn update_clicked_node(&mut self, mouse_pos: Vec2) {
        if self.clicked_node.is_some() {
            return;
        }

        let low_rect = Rect::from_xy_wh(self.low_filter_node, pt2(14.0, 14.0));
        let peak_rect =
            Rect::from_xy_wh(self.peak_filter_node, pt2(14.0, 14.0));
        let high_rect =
            Rect::from_xy_wh(self.high_filter_node, pt2(14.0, 14.0));

        if low_rect.contains(mouse_pos) {
            self.clicked_node = Some(NodeType::Low);
        }
        else if peak_rect.contains(mouse_pos) {
            self.clicked_node = Some(NodeType::Peak);
        }
        else if high_rect.contains(mouse_pos) {
            self.clicked_node = Some(NodeType::High);
        }
        else {
            self.clicked_node = None;
        }
    }

    /// Updates the node positions from external parameter changes.
    fn update_nodes_from_param_changes(&mut self, input: &InputData) {
        const Q_SCALE_FACTOR: f32 = 3.8206;
        let q_scale_tanh = -Q_SCALE_FACTOR.tanh();

        let mp = input.mouse_pos;
        let clicked = input.is_left_clicked;
        let rect = self.rect;
        let padded = rect.pad(8.0);
        let sr = self.sample_rate.lr();

        // Set node positions based on filter parameter updates.
        // LOW FILTER
        // frequency
        let low_freq = self.low_filter_params.cutoff;
        self.low_filter_node.x = rect.left()
            + freq_log_norm(low_freq as f64, 25.0, sr) as f32 * rect.w();

        // gain
        if self.low_filter_is_shelf.lr() {
            let gain = self.low_filter_params.gain as f32;
            let norm = map_f32(gain, -24.0, 24.0, 0.02962963, 0.97037035);
            self.low_filter_node.y =
                map_f32(norm, 0.0, 1.0, rect.bottom(), rect.top());
        }
        // q
        else {
            let q = normalize_f32(
                self.low_filter_params.q.recip() as f32,
                0.3,
                10.0,
            );
            let q_norm = (q_scale_tanh * (q - 1.0)).atanh() / Q_SCALE_FACTOR;
            let norm = scale_f32(q_norm, 0.02962963, 0.97037035);
            self.low_filter_node.y =
                map_f32(norm, 1.0, 0.0, rect.bottom(), rect.top());
        }

        // PEAK FILTER
        // freq
        let peak_freq = self.peak_filter_params.cutoff;
        self.peak_filter_node.x = rect.left()
            + freq_log_norm(peak_freq as f64, 25.0, sr) as f32 * rect.w();

        // gain
        let gain = self.peak_filter_params.gain as f32;
        let norm = map_f32(gain, -24.0, 24.0, 0.02962963, 0.97037035);
        self.peak_filter_node.y =
            map_f32(norm, 0.0, 1.0, rect.bottom(), rect.top());

        // HIGH FILTER

        // freq
        let high_freq = self.high_filter_params.cutoff;
        self.high_filter_node.x = rect.left()
            + freq_log_norm(high_freq as f64, 25.0, sr) as f32 * rect.w();

        // gain
        if self.high_filter_is_shelf.lr() {
            let gain = self.high_filter_params.gain as f32;
            let norm = map_f32(gain, -24.0, 24.0, 0.02962963, 0.97037035);
            self.high_filter_node.y =
                map_f32(norm, 0.0, 1.0, rect.bottom(), rect.top());
        }
        // q
        else {
            let q = normalize_f32(
                self.high_filter_params.q.recip() as f32,
                0.3,
                10.0,
            );
            let q_norm = (q_scale_tanh * (q - 1.0)).atanh() / Q_SCALE_FACTOR;
            let norm = scale_f32(q_norm, 0.02962963, 0.97037035);
            self.high_filter_node.y =
                map_f32(norm, 1.0, 0.0, rect.bottom(), rect.top());
        }

        // clamp nodes into the padded rect
        self.low_filter_node = self
            .low_filter_node
            .clamp(padded.bottom_left(), padded.top_right());
        self.peak_filter_node = self
            .peak_filter_node
            .clamp(padded.bottom_left(), padded.top_right());
        self.high_filter_node = self
            .high_filter_node
            .clamp(padded.bottom_left(), padded.top_right());
    }
}

impl UIDraw for EQDisplay {
    /// Y-pos to Q (and back) conversions found experimenting on Desmos:
    /// <https://www.desmos.com/calculator/ddgep83pq2>
    #[allow(clippy::too_many_lines)]
    fn update(&mut self, app: &App, input: &InputData) {
        const Q_SCALE_FACTOR: f32 = 3.8206;
        let q_scale_tanh = -Q_SCALE_FACTOR.tanh();

        let mp = input.mouse_pos;
        let clicked = input.is_left_clicked;
        let rect = self.rect;
        let padded = rect.pad(8.0);
        let sr = self.sample_rate.lr();

        // horrendous indenting but it works
        // TODO refactor, please...
        if clicked {
            if rect.contains(mp) || self.spectrum_is_clicked {
                if !self.clicked_outside_of_spectrum {
                    self.update_clicked_node(input.mouse_pos);

                    // if a node is being dragged, update the relevant FilterParams.
                    if let Some(node) = self.clicked_node {
                        match node {
                            NodeType::Low => {
                                self.low_filter_node = mp.clamp(
                                    padded.bottom_left(),
                                    padded.top_right(),
                                );

                                // frequency
                                let xpos_norm = (self.low_filter_node.x
                                    - rect.left())
                                    / rect.w();
                                let freq = freq_lin_from_log(
                                    xpos_norm as f64, 25.0, sr,
                                );
                                self.low_filter_params.cutoff =
                                    freq.clamp(25.0, 20000.0);

                                // gain
                                let ypos_norm = normalize_f32(
                                    (self.low_filter_node.y - rect.bottom())
                                        / rect.h(),
                                    0.02962963,
                                    0.97037035,
                                );
                                let gain = scale_f32(ypos_norm, -24.0, 24.0);
                                self.low_filter_params.gain = gain as f64;

                                // q
                                let q = scale_f32(
                                    1.0 + ((1.0 - ypos_norm) * Q_SCALE_FACTOR)
                                        .tanh()
                                        / q_scale_tanh,
                                    0.3,
                                    10.0,
                                );
                                self.low_filter_params.q = (q as f64).recip();
                            }
                            NodeType::Peak => {
                                self.peak_filter_node = mp.clamp(
                                    padded.bottom_left(),
                                    padded.top_right(),
                                );

                                // frequency
                                let xpos_norm = (self.peak_filter_node.x
                                    - rect.left())
                                    / rect.w();
                                let freq = freq_lin_from_log(
                                    xpos_norm as f64, 25.0, sr,
                                );
                                self.peak_filter_params.cutoff =
                                    freq.clamp(25.0, 20000.0);

                                // gain
                                let ypos_norm = normalize_f32(
                                    (self.peak_filter_node.y - rect.bottom())
                                        / rect.h(),
                                    0.02962963,
                                    0.97037035,
                                );
                                let gain = scale_f32(ypos_norm, -24.0, 24.0);
                                self.peak_filter_params.gain = gain as f64;
                            }
                            NodeType::High => {
                                self.high_filter_node = mp.clamp(
                                    padded.bottom_left(),
                                    padded.top_right(),
                                );

                                // frequency
                                let xpos_norm = (self.high_filter_node.x
                                    - rect.left())
                                    / rect.w();
                                let freq = freq_lin_from_log(
                                    xpos_norm as f64, 25.0, sr,
                                );
                                self.high_filter_params.cutoff =
                                    freq.clamp(25.0, 20000.0);

                                // gain
                                let ypos_norm = normalize_f32(
                                    (self.high_filter_node.y - rect.bottom())
                                        / rect.h(),
                                    0.02962963,
                                    0.97037035,
                                );
                                let gain = scale_f32(ypos_norm, -24.0, 24.0);
                                self.high_filter_params.gain = gain as f64;

                                // q
                                let q = scale_f32(
                                    1.0 + ((1.0 - ypos_norm) * Q_SCALE_FACTOR)
                                        .tanh()
                                        / q_scale_tanh,
                                    0.3,
                                    10.0,
                                );
                                self.high_filter_params.q = (q as f64).recip();
                            }
                        }
                    }
                }

                self.spectrum_is_clicked = true;
            }
            else if !self.spectrum_is_clicked {
                self.clicked_outside_of_spectrum = true;
            }
        }
        else {
            self.spectrum_is_clicked = false;
            self.clicked_outside_of_spectrum = false;
            self.clicked_node = None;
        }

        if clicked && self.clicked_outside_of_spectrum {
            self.update_nodes_from_param_changes(input);
        }

        self.update_filters();
        self.update_filter_line();
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        self.draw_filter_nodes(draw);

        // draw the filter line
        draw.polyline()
            .weight(2.0)
            .points(self.filter_points.iter().copied())
            .color(Rgba::new(1.0, 1.0, 1.0, 0.08));
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}
