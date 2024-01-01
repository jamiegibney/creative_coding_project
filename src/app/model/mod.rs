use crate::gui::rdp::rdp_in_place;
use crate::prelude::interp::linear_unclamped;
use crossbeam_channel::{unbounded, Receiver, Sender};
use nannou_audio::Stream;

// use nannou_egui::{self, egui, Egui};

use super::audio::*;
use super::view::view;
use super::*;
use crate::app::params::*;
use crate::dsp::{
    BiquadFilter, BiquadParams, Filter, FilterType, ResoBankData,
    ResonatorBankParams, SpectralMask, BUTTERWORTH_Q,
};
use crate::generative::*;
use crate::gui::spectrum::*;
use crate::gui::UIComponents;
use nannou::prelude::WindowId as Id;

use std::f64::consts::SQRT_2;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{mpsc, Arc, Mutex, RwLock},
    time::Instant,
};

mod constructors;
use super::audio::audio_constructor;
use constructors::*;

type CallbackTimerRef = Arc<Mutex<Instant>>;

/// The app's model, i.e. its general state.
#[allow(clippy::struct_excessive_bools)]
pub struct Model {
    window: Id,

    // pub egui: Egui,
    pub ui_params: UIParams,

    // AUDIO DATA
    /// The CPAL audio stream.
    pub audio_stream: nannou_audio::Stream<AudioModel>,
    /// Channels to send messages directly to the audio thread.
    pub audio_senders: Arc<AudioMessageSenders>,

    /// A thread-safe reference to the mask used for spectral filtering.
    // pub spectral_mask: Arc<Mutex<SpectralMask>>,
    pub spectral_mask: triple_buffer::Input<SpectralMask>,

    pub voice_event_sender: mpsc::Sender<VoiceEvent>,

    /// A thread-safe reference to the timer which tracks when the audio callback
    /// was last called.
    pub audio_callback_timer: CallbackTimerRef,

    /// A string showing the (rough) DSP load.
    pub dsp_load: Option<String>,

    /// A reference to the sample rate value.
    pub sample_rate_ref: Arc<AtomicF64>,

    // NOTES
    /// Current octave for note input (via typing keyboard).
    pub octave: Octave,
    /// A thread-safe reference to the note handler.
    pub note_handler: NoteHandlerRef,
    /// A HashMap of the currently-pressed keys.
    // TODO: this doesn't register that keys are unpressed when "switching octaves"
    // (Z and X)
    pub pressed_keys: HashMap<Key, bool>,

    // GUI
    /// The pre-FX spectrogram.
    pub pre_spectrum_analyzer: RefCell<SpectrumAnalyzer>,
    /// The post-FX spectrogram.
    pub post_spectrum_analyzer: RefCell<SpectrumAnalyzer>,

    pub ui_components: UIComponents,

    pub bank_rect: Rect,
    pub reso_bank_reset_receiver: Receiver<()>,
    pub reso_bank_push_receiver: Receiver<()>,
    pub reso_bank_data: triple_buffer::Input<ResoBankData>,
    pub mask_rect: Rect,
    pub mouse_clicked_outside_of_mask: bool,
    pub mask_clicked: bool,
    pub spectrum_rect: Rect,

    /// A Perlin noise contour generator.
    pub contours: Arc<RwLock<ContoursGPU>>,
    /// A SmoothLife simulation.
    pub smooth_life: Arc<RwLock<SmoothLifeGPU>>,
    /// A Voronoi noise generator used for the spectral mask.
    pub voronoi_mask: Arc<RwLock<VoronoiGPU>>,
    /// A vector field used to manage points for the Voronoi mask.
    pub voronoi_vectors: Arc<RwLock<Vectors>>,
    pub voronoi_z: Arc<AtomicF64>,

    /// A simple vector field for the resonator bank points.
    pub vectors_reso_bank: Vectors,
    /// The voronoi generator for the resonator bank vector field.
    pub voronoi_reso_bank: VoronoiGPU,
    /// The line which shows which column is being used as a spectral mask.
    pub mask_scan_line_pos: f64,
    /// The amount to increment the position of the mask scan line each frame.
    pub mask_scan_line_increment: f64,

    pub low_filter: BiquadFilter,
    pub high_filter: BiquadFilter,
    low_filter_node: Vec2,
    low_filter_node_is_clicked: bool,
    high_filter_node: Vec2,
    high_filter_node_is_clicked: bool,
    spectrum_is_clicked: bool,
    pub clicked_outside_of_spectrum: bool,

    pub filter_raw_points: Vec<[f64; 2]>,
    pub filter_indices: Vec<usize>,
    pub filter_points: Vec<Vec2>,
    pub log_lines: Vec<[Vec2; 2]>,

    pub input_data: InputData,

    pub mask_thread_pool: ThreadPool,
}

impl Model {
    /// Builds the app's `Model`.
    ///
    /// # Panics
    ///
    /// Panics if a new window cannot be initialized.
    #[allow(clippy::too_many_lines)]
    pub fn build(app: &App) -> Self {
        let params = build_ui_parameters();
        let AudioSystem {
            stream: audio_stream,
            sample_rate_ref,
            senders: audio_senders,
            callback_timer_ref: audio_callback_timer,
            note_handler,
            pre_spectrum,
            post_spectrum,
            voice_event_sender,
            spectral_mask,
            reso_bank_data,
        } = build_audio_system(&params);

        let (_w, _h) = (WINDOW_SIZE.x as f32, WINDOW_SIZE.y as f32);

        let window =
            build_window(app, WINDOW_SIZE.x as u32, WINDOW_SIZE.y as u32);

        let GuiElements {
            bank_rect,
            mask_rect,
            spectrum_rect,
            contours,
            smooth_life,
            voronoi_mask,
            mut voronoi_vectors,
            pre_spectrum_analyzer,
            post_spectrum_analyzer,
            dsp_load,
            vectors_reso_bank,
        } = build_gui_elements(app, pre_spectrum, post_spectrum, &params);

        voronoi_vectors.override_points().iter_mut().for_each(|p| {
            p.vel.x = random_range(-1.0, 1.0);
            p.vel.y = random_range(-1.0, 1.0);
        });

        let audio_senders = Arc::new(audio_senders);
        let audio_senders_cl = Arc::clone(&audio_senders);

        let contours = Arc::new(RwLock::new(contours));
        let smooth_life = Arc::new(RwLock::new(smooth_life));
        let ctr_1 = Arc::clone(&contours);
        let sml = Arc::clone(&smooth_life);
        let gen_algo = Arc::clone(&params.mask_algorithm);

        let (reso_bank_reset_sender, reso_bank_reset_receiver) = unbounded();

        let (reso_bank_push_sender, reso_bank_push_receiver) = unbounded();

        let voronoi_z = Arc::new(AtomicF64::new(0.0));

        let voronoi_vectors = Arc::new(RwLock::new(voronoi_vectors));
        let vv = Arc::clone(&voronoi_vectors);

        let (m_tl, m_br) = (mask_rect.top_left(), mask_rect.bottom_right());

        let ui_components = UIComponents::new(&params)
            .attach_reso_bank_randomize_callback(move |_| {
                reso_bank_reset_sender.send(());
            })
            .attach_reso_bank_push_callback(move |_| {
                reso_bank_push_sender.send(());
            })
            .attach_mask_reset_callback(move |_| match gen_algo.lr() {
                GenerativeAlgo::Contours => {
                    if let Ok(guard) = ctr_1.read() {
                        guard.randomize();
                    }
                }
                GenerativeAlgo::SmoothLife => {
                    if let Ok(guard) = sml.read() {
                        guard.randomize();
                    }
                }
                GenerativeAlgo::Voronoi => {
                    if let Ok(mut guard) = vv.write() {
                        guard.override_points().iter_mut().for_each(|p| {
                            p.vel.x = random_range(-1.0, 1.0);
                            p.vel.y = random_range(-1.0, 1.0);

                            p.pos.x = random_range(m_tl.x, m_br.x);
                            p.pos.y = random_range(m_br.y, m_tl.y);
                        });
                    }
                }
            })
            .setup_mask_callbacks(
                Arc::clone(&contours),
                Arc::clone(&smooth_life),
                &params,
            );

        let mut low_filter = BiquadFilter::new(sample_rate_ref.lr());
        low_filter.set_params(&BiquadParams {
            freq: params.low_filter_cutoff.current_value(),
            gain: params.low_filter_gain_db.current_value(),
            q: params.low_filter_q.current_value(),
            filter_type: if params.low_filter_is_shelf.lr() {
                FilterType::Lowshelf
            }
            else {
                FilterType::Highpass
            },
        });

        let mut high_filter = BiquadFilter::new(sample_rate_ref.lr());
        high_filter.set_params(&BiquadParams {
            freq: params.high_filter_cutoff.current_value(),
            gain: params.high_filter_gain_db.current_value(),
            q: params.high_filter_q.current_value(),
            filter_type: if params.high_filter_is_shelf.lr() {
                FilterType::Highshelf
            }
            else {
                FilterType::Lowpass
            },
        });

        Self {
            window,

            // egui,
            ui_components,
            ui_params: params,

            audio_stream,
            audio_senders,

            octave: Octave::default(), // C3 - B3

            note_handler: Arc::clone(&note_handler),

            pressed_keys: build_pressed_keys_map(),

            audio_callback_timer,

            pre_spectrum_analyzer,
            post_spectrum_analyzer,

            voice_event_sender,

            spectral_mask,

            bank_rect,
            mask_rect,
            reso_bank_reset_receiver,
            reso_bank_push_receiver,
            reso_bank_data,
            spectrum_rect,

            mouse_clicked_outside_of_mask: false,
            mask_clicked: false,

            contours,
            smooth_life,

            voronoi_mask: Arc::new(RwLock::new(voronoi_mask)),
            voronoi_vectors,
            voronoi_z,

            voronoi_reso_bank: VoronoiGPU::new(app, bank_rect),
            vectors_reso_bank,

            low_filter,
            high_filter,
            low_filter_node: Vec2::new(-244.94064, -174.99988),
            low_filter_node_is_clicked: false,
            high_filter_node: Vec2::new(-108.400024, -175.00002),
            high_filter_node_is_clicked: false,
            spectrum_is_clicked: false,
            clicked_outside_of_spectrum: false,

            filter_raw_points: {
                let mid = spectrum_rect.mid_left().y;
                let left = spectrum_rect.left();
                let right = spectrum_rect.right();
                let points = spectrum_rect.w() as usize;

                (0..spectrum_rect.w() as usize)
                    .map(|i| {
                        let x_pos = left as f64 + i as f64;
                        [x_pos, mid as f64]
                    })
                    .collect()
            },
            filter_points: vec![Vec2::ZERO; spectrum_rect.w() as usize],
            filter_indices: vec![0; spectrum_rect.w() as usize],
            log_lines: create_log_lines(&spectrum_rect),

            mask_scan_line_pos: 0.0,
            mask_scan_line_increment: 0.1,

            input_data: InputData::default(),

            mask_thread_pool: ThreadPool::build(1)
                .expect("failed to build mask thread pool"),

            dsp_load,
            sample_rate_ref,
        }
    }

    /// Returns the (approximate) sample index for the current moment in time.
    ///
    /// This is **not** a particularly precise method of tracking time events,
    /// but should be more than adequate for things like note events.
    ///
    /// If a lock on the callback timer is not obtained, then `0` is returned.
    /// This doesn't create too much of an issue as note events are still
    /// handled quite quickly in the audio thread.
    pub fn current_sample_idx(&self) -> u32 {
        self.audio_callback_timer.lock().map_or(0, |guard| {
            let samples_exact =
                guard.elapsed().as_secs_f64() * unsafe { SAMPLE_RATE };
            samples_exact.round() as u32 % BUFFER_SIZE as u32
        })
    }

    /// Increments the internal position of the mask scan line.
    pub fn increment_mask_scan_line(&mut self) {
        let increment = self.ui_params.mask_scan_line_speed.lr();
        self.mask_scan_line_pos += increment * self.input_data.delta_time;

        if self.mask_scan_line_pos > 1.0 {
            self.mask_scan_line_pos -= 1.0;
        }
        else if self.mask_scan_line_pos < 0.0 {
            self.mask_scan_line_pos += 1.0;
        }
    }

    /// # Panics
    ///
    /// This will panic if the `SmoothLife` generator cannot be locked.
    pub fn mask_rect(&self) -> Rect {
        self.mask_rect
    }

    /// Draws the spectral mask scan line.
    pub fn draw_mask_scan_line(&self, draw: &Draw) {
        let rect = self.mask_rect();
        let y_top = rect.top() - 1.0;
        let y_bot = rect.bottom() + 1.0;

        let x = map(
            self.mask_scan_line_pos,
            0.0,
            1.0,
            rect.left() as f64,
            rect.right() as f64,
        ) as f32;

        draw.line()
            .points(pt2(x, y_bot), pt2(x, y_top))
            .weight(3.0)
            .color(Rgba::new(0.9, 0.4, 0.0, 0.5));
    }

    /// Updates the model's input data.
    pub fn update_input_data(&mut self, app: &App, update: Update) {
        // self.egui.set_elapsed_time(update.since_start);

        self.input_data.delta_time = update.since_last.as_secs_f64();
        self.input_data.mouse_pos = app.mouse.position();
        // mouse scroll delta is updated in the event() callback

        self.input_data.is_left_clicked = app.mouse.buttons.left().is_down();
        self.input_data.is_right_clicked = app.mouse.buttons.right().is_down();

        let modifers = &app.keys.mods;
        self.input_data.is_alt_down = modifers.alt();
        self.input_data.is_os_mod_down = modifers.logo();
        self.input_data.is_shift_down = modifers.shift();
        self.input_data.is_ctrl_down = modifers.ctrl();
    }

    pub fn update_vectors(&mut self, app: &App) {
        self.vectors_reso_bank.set_num_active_points(
            self.ui_params.reso_bank_resonator_count.lr() as usize,
        );
        self.vectors_reso_bank
            .set_friction(self.ui_params.reso_bank_field_friction.lr());

        if self.reso_bank_reset_receiver.try_recv().is_ok() {
            self.vectors_reso_bank.randomize_points();
        }
        if self.reso_bank_push_receiver.try_recv().is_ok() {
            self.vectors_reso_bank.push_points();
        }

        self.vectors_reso_bank.update(app, &self.input_data);

        self.vectors_reso_bank
            .set_reso_bank_data(self.reso_bank_data.input_buffer());

        self.reso_bank_data.publish();

        self.voronoi_reso_bank
            .copy_from_vectors(&self.vectors_reso_bank);

        if self.ui_components.reso_bank_scale.is_open()
            && self
                .ui_components
                .reso_bank_scale
                .rect()
                .contains(self.input_data.mouse_pos)
        {
            self.vectors_reso_bank.can_mouse_interact = false;
        }
    }

    pub fn update_voronoi_vectors(&mut self, app: &App) {
        let mut guard = self.voronoi_vectors.write().unwrap();
        guard.set_num_active_points(
            self.ui_params.voronoi_cell_count.lr() as usize
        );

        let (l, r) = (self.mask_rect.left(), self.mask_rect.right());
        let (b, t) = (self.mask_rect.bottom(), self.mask_rect.top());
        let (w, h) = (self.mask_rect.w(), self.mask_rect.h());

        let points = guard.override_points();

        for point in points {
            point.pos +=
                point.vel * self.ui_params.voronoi_cell_speed.lr() as f32 * 3.0;

            if point.pos.x < l {
                point.vel.x *= -1.0;
            }
            if point.pos.x > r {
                point.vel.x *= -1.0;
            }

            if point.pos.y < b {
                point.vel.y *= -1.0;
            }
            if point.pos.y > t {
                point.vel.y *= -1.0;
            }
        }

        guard.update(app, &self.input_data);
    }

    pub fn update_mask_scan_line_from_mouse(&mut self) {
        if self.input_data.is_left_clicked {
            if self.mask_rect.contains(self.input_data.mouse_pos)
                || self.mask_clicked
            {
                if !self.mouse_clicked_outside_of_mask {
                    let x_pos = self.input_data.mouse_pos.x as f64;
                    let l = self.mask_rect.left() as f64;
                    let r = self.mask_rect.right() as f64;

                    self.mask_scan_line_pos = normalize(x_pos, l, r).fract();
                    if self.mask_scan_line_pos.is_sign_negative() {
                        self.mask_scan_line_pos = 1.0 + self.mask_scan_line_pos;
                    }
                }
                self.mask_clicked = true;
            }
            else if !self.mask_clicked {
                self.mouse_clicked_outside_of_mask = true;
            }
        }
        else {
            self.mouse_clicked_outside_of_mask = false;
            self.mask_clicked = false;
        }
    }

    pub fn update_filters(&mut self) {
        let low_filter_is_shelf = self.ui_components.low_filter_type.enabled();
        self.low_filter.set_params(&BiquadParams {
            freq: note_to_freq(self.ui_components.low_filter_cutoff.value()),
            gain: self.ui_components.low_filter_gain.value(),
            q: if low_filter_is_shelf {
                SQRT_2
            }
            else {
                self.ui_components.low_filter_q.value().recip()
            },
            filter_type: if low_filter_is_shelf {
                FilterType::Lowshelf
            }
            else {
                FilterType::Highpass
            },
        });

        let high_filter_is_shelf =
            self.ui_components.high_filter_type.enabled();

        self.high_filter.set_params(&BiquadParams {
            freq: note_to_freq(self.ui_components.high_filter_cutoff.value()),
            gain: self.ui_components.high_filter_gain.value(),
            q: if high_filter_is_shelf {
                SQRT_2
            }
            else {
                self.ui_components.high_filter_q.value().recip()
            },
            filter_type: if high_filter_is_shelf {
                FilterType::Highshelf
            }
            else {
                FilterType::Lowpass
            },
        });

        self.low_filter.process(0.0);
        self.high_filter.process(0.0);
    }

    #[allow(clippy::many_single_char_names)]
    pub fn update_filter_line(&mut self) {
        let sr = self.sample_rate_ref.lr();
        let (l, r) = (
            self.spectrum_rect.left() as f64,
            self.spectrum_rect.right() as f64,
        );
        let (b, t) = (
            self.spectrum_rect.bottom() as f64,
            self.spectrum_rect.top() as f64,
        );
        let w = self.spectrum_rect.w() as f64;
        let half_height = self.spectrum_rect.h() as f64 * 0.5; // marks +- 30 db
        let mid = self.spectrum_rect.mid_left().y as f64;

        for (i, point) in self.filter_raw_points.iter_mut().enumerate().skip(1)
        {
            let x = i as f64 / w;
            let freq = freq_lin_from_log(x, 25.0, sr);
            let scaled = map(freq, 0.0, sr * 0.5, 0.0, PI_F64);

            let mag_db = self.low_filter.response_at(scaled)
                + self.high_filter.response_at(scaled);

            point[1] = (mid
                + map(mag_db, -30.0, 30.0, -half_height, half_height))
            .clamp(b, t);
        }

        // copy second point to first, as the first is ignored.
        self.filter_raw_points[0][1] = self.filter_raw_points[1][1];

        // decimate redundant points — this reduces the number of points to be drawn by
        // over 10 times.
        rdp_in_place(
            self.filter_raw_points.as_slice(),
            &mut self.filter_indices,
            0.2,
        );

        let len = self.filter_indices.len();

        unsafe {
            self.filter_points.set_len(len);
        }

        for i in 0..len {
            let point = self.filter_raw_points[self.filter_indices[i]];
            self.filter_points[i] =
                Vec2::from([point[0] as f32, point[1] as f32]);
        }
    }

    pub fn draw_filter_line(&self, draw: &Draw) {
        draw.polyline()
            .weight(2.0)
            .points(self.filter_points.iter().copied())
            .color(Rgba::new(1.0, 1.0, 1.0, 0.08));
    }

    pub fn draw_log_lines(&self, draw: &Draw) {
        for line in &self.log_lines {
            draw.line()
                .points(line[0], line[1])
                .weight(2.0)
                .color(Rgb::new(0.08, 0.08, 0.08));
        }
    }

    #[allow(clippy::too_many_lines)]
    /// Y-pos to Q (and back) conversions found experimenting on Desmos:
    /// <https://www.desmos.com/calculator/ddgep83pq2>
    pub fn update_filter_nodes(&mut self, app: &App) {
        const Q_SCALE_FACTOR: f32 = 3.8206;
        let q_scale_tanh = -Q_SCALE_FACTOR.tanh();

        if self.ui_components.exciter_osc.is_open() {
            self.clicked_outside_of_spectrum = true;
            return;
        }

        let mp = self.input_data.mouse_pos;
        let clicked = self.input_data.is_left_clicked;
        let rect = self.spectrum_rect;
        let low_rect = Rect::from_xy_wh(self.low_filter_node, pt2(14.0, 14.0));
        let high_rect =
            Rect::from_xy_wh(self.high_filter_node, pt2(14.0, 14.0));

        let padded = rect.pad(8.0);
        let sr = self.sample_rate_ref.lr();

        if clicked {
            if rect.contains(mp) || self.spectrum_is_clicked {
                if !self.clicked_outside_of_spectrum {
                    if (low_rect.contains(mp)
                        || self.low_filter_node_is_clicked)
                        && !self.high_filter_node_is_clicked
                    {
                        self.low_filter_node =
                            mp.clamp(padded.bottom_left(), padded.top_right());
                        self.low_filter_node_is_clicked = true;

                        // frequency
                        let xpos_norm =
                            (self.low_filter_node.x - rect.left()) / rect.w();
                        let freq =
                            freq_lin_from_log(xpos_norm as f64, 25.0, sr);
                        self.ui_components
                            .low_filter_cutoff
                            .set_value(freq_to_note(freq.clamp(25.0, 20000.0)));

                        // gain
                        let ypos_norm = normalize_f32(
                            (self.low_filter_node.y - rect.bottom()) / rect.h(),
                            0.02962963,
                            0.97037035,
                        );
                        let gain = scale_f32(ypos_norm, -24.0, 24.0);
                        self.ui_components
                            .low_filter_gain
                            .set_value(gain as f64);

                        // q
                        let q = scale_f32(
                            1.0 + ((1.0 - ypos_norm) * Q_SCALE_FACTOR).tanh()
                                / q_scale_tanh,
                            0.3,
                            10.0,
                        );
                        self.ui_components.low_filter_q.set_value(q as f64);
                    }
                    if (high_rect.contains(mp)
                        || self.high_filter_node_is_clicked)
                        && !self.low_filter_node_is_clicked
                    {
                        self.high_filter_node =
                            mp.clamp(padded.bottom_left(), padded.top_right());
                        self.high_filter_node_is_clicked = true;

                        // frequency
                        let xpos_norm =
                            (self.high_filter_node.x - rect.left()) / rect.w();
                        let freq =
                            freq_lin_from_log(xpos_norm as f64, 25.0, sr);
                        self.ui_components
                            .high_filter_cutoff
                            .set_value(freq_to_note(freq.clamp(25.0, 20000.0)));

                        // gain
                        let ypos_norm = normalize_f32(
                            (self.high_filter_node.y - rect.bottom())
                                / rect.h(),
                            0.02962963,
                            0.97037035,
                        );
                        let gain = scale_f32(ypos_norm, -24.0, 24.0);
                        self.ui_components
                            .high_filter_gain
                            .set_value(gain as f64);

                        // q
                        let q = scale_f32(
                            1.0 + ((1.0 - ypos_norm) * Q_SCALE_FACTOR).tanh()
                                / q_scale_tanh,
                            0.3,
                            10.0,
                        );
                        self.ui_components.high_filter_q.set_value(q as f64);
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
            self.low_filter_node_is_clicked = false;
            self.high_filter_node_is_clicked = false;
        }
        if clicked && self.clicked_outside_of_spectrum {
            let low_freq =
                note_to_freq(self.ui_components.low_filter_cutoff.value());

            self.low_filter_node.x = rect.left()
                + freq_log_norm(low_freq as f64, 25.0, sr) as f32 * rect.w();
            if self.ui_components.low_filter_type.enabled() {
                let gain = self.ui_components.low_filter_gain.value() as f32;
                let norm = map_f32(gain, -24.0, 24.0, 0.02962963, 0.97037035);
                self.low_filter_node.y =
                    map_f32(norm, 0.0, 1.0, rect.bottom(), rect.top());
            }
            else {
                let q = normalize_f32(
                    self.ui_components.low_filter_q.value() as f32,
                    0.3,
                    10.0,
                );
                let q_norm =
                    (q_scale_tanh * (q - 1.0)).atanh() / Q_SCALE_FACTOR;
                let norm = scale_f32(q_norm, 0.02962963, 0.97037035);
                self.low_filter_node.y =
                    map_f32(norm, 1.0, 0.0, rect.bottom(), rect.top());
            }

            let high_freq =
                note_to_freq(self.ui_components.high_filter_cutoff.value());

            self.high_filter_node.x = rect.left()
                + freq_log_norm(high_freq as f64, 25.0, sr) as f32 * rect.w();
            if self.ui_components.high_filter_type.enabled() {
                let gain = self.ui_components.high_filter_gain.value() as f32;
                let norm = map_f32(gain, -24.0, 24.0, 0.02962963, 0.97037035);
                self.high_filter_node.y =
                    map_f32(norm, 0.0, 1.0, rect.bottom(), rect.top());
            }
            else {
                let q = normalize_f32(
                    self.ui_components.high_filter_q.value() as f32,
                    0.3,
                    10.0,
                );
                let q_norm =
                    (q_scale_tanh * (q - 1.0)).atanh() / Q_SCALE_FACTOR;
                let norm = scale_f32(q_norm, 0.02962963, 0.97037035);
                self.high_filter_node.y =
                    map_f32(norm, 1.0, 0.0, rect.bottom(), rect.top());
            }

            self.low_filter_node = self
                .low_filter_node
                .clamp(padded.bottom_left(), padded.top_right());
            self.high_filter_node = self
                .high_filter_node
                .clamp(padded.bottom_left(), padded.top_right());
        }
    }

    pub fn draw_filter_nodes(&self, draw: &Draw) {
        draw.ellipse()
            .xy(self.low_filter_node)
            .radius(7.0)
            .color(Rgba::new(0.9, 0.4, 0.0, 0.5));

        draw.ellipse()
            .xy(self.high_filter_node)
            .radius(7.0)
            .color(Rgba::new(0.9, 0.4, 0.0, 0.5));
    }
}

fn create_log_lines(rect: &Rect) -> Vec<[Vec2; 2]> {
    let (w, h) = (rect.w() as f64, rect.h() as f64);
    let (l, r) = (rect.left() as f64, rect.right() as f64);
    let (b, t) = (rect.bottom(), rect.top());

    let max = *LOG_10_VALUES.last().unwrap();

    let lower = freq_log_norm(10.0, 25.0, 44100.0);
    let upper = freq_log_norm(30000.0, 25.0, 44100.0);
    let lower_x = linear_unclamped(l, r, lower);
    let upper_x = linear_unclamped(l, r, upper);

    LOG_10_VALUES
        .iter()
        .skip(2)
        .take(26)
        .map(|&x| {
            let norm = normalize(x, 0.0, max);
            let x_pos = lerp(lower_x, upper_x, norm) as f32;

            [Vec2::new(x_pos, b), Vec2::new(x_pos, t)]
        })
        .collect()
}

/// Log values intended to represent the logarithmic
/// scaling from 10 Hz to 30 kHz.
#[allow(
    clippy::unreadable_literal,
    clippy::excessive_precision,
    clippy::approx_constant,
)]
#[rustfmt::skip]
pub const LOG_10_VALUES: [f64; 30] = [
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
