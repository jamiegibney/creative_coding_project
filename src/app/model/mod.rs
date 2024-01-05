//! The whole app's state.

use super::audio::audio_constructor;
use super::audio::*;
use super::view::view;
use super::*;
use crate::app::params::*;
use crate::dsp::{
    BiquadFilter, BiquadParams, Filter, FilterType, ResoBankData,
    ResonatorBankParams, SpectralMask, BUTTERWORTH_Q,
};
use crate::generative::*;
use crate::gui::rdp::rdp_in_place;
use crate::gui::{spectrum::*, EQDisplay};
use crate::gui::{EQFilterParams, UIComponents};
use crate::prelude::interp::linear_unclamped;
use crossbeam_channel::{unbounded, Receiver, Sender};
use nannou::prelude::WindowId as Id;
use nannou_audio::Stream;
use std::f64::consts::SQRT_2;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{mpsc, Arc, Mutex, RwLock},
    time::Instant,
};

mod constructors;
use constructors::*;

type CallbackTimerRef = Arc<Mutex<Instant>>;

/// The app's model, i.e. its state.
#[allow(clippy::struct_excessive_bools)]
pub struct Model {
    window: Id,

    /// UI parameters.
    pub ui_params: UIParams,

    /// The CPAL audio stream.
    pub audio_stream: nannou_audio::Stream<AudioModel>,
    /// Channels to send messages directly to the audio thread.
    pub audio_senders: Arc<AudioMessageSenders>,

    /// Input to the spectral mask channel, sent to the audio thread.
    pub spectral_mask: triple_buffer::Input<SpectralMask>,

    /// Channel to send voice events (such as killing all voices).
    pub voice_event_sender: mpsc::Sender<VoiceEvent>,

    /// A thread-safe reference to the timer which tracks when the audio callback
    /// was last called.
    pub audio_callback_timer: CallbackTimerRef,

    /// A reference to the sample rate value.
    pub sample_rate_ref: Arc<AtomicF64>,

    /// Current octave for note input (via typing keyboard).
    pub octave: Octave,
    /// A thread-safe reference to the note handler.
    pub note_handler: NoteHandlerRef,
    /// A HashMap of the currently-pressed keys.
    pub pressed_keys: HashMap<Key, bool>,

    /// The pre-FX spectrogram.
    pub pre_spectrum_analyzer: SpectrumAnalyzer,
    /// The post-FX spectrogram.
    pub post_spectrum_analyzer: SpectrumAnalyzer,

    /// All GUI control components (sliders, menus, buttons, labels...)
    pub ui_components: UIComponents,

    /// The resonator bank bounding rect.
    pub bank_rect: Rect,
    /// A receiver for the resonator bank "reset" button.
    pub reso_bank_reset_receiver: Receiver<()>,
    /// A sender to allow a keybind to reset the resonator bank.
    pub reso_bank_reset_sender_key: Sender<()>,
    /// A receiver for the resonator bank "push" button.
    pub reso_bank_push_receiver: Receiver<()>,
    /// A sender to allow a keybind to push the resonator bank.
    pub reso_bank_push_sender_key: Sender<()>,
    /// Input to the resonator bank state channel, sent to the audio thread.
    pub reso_bank_data: triple_buffer::Input<ResoBankData>,

    /// The spectral filter bounding rect.
    pub mask_rect: Rect,
    /// Whether the mouse was clicked outside of the spectral filter — used to track
    /// input events applied to the mask scan line.
    pub mouse_clicked_outside_of_mask: bool,
    /// Whether the spectral filter is clicked or not.
    pub mask_clicked: bool,

    /// The spectrogram/EQ bounding rect.
    pub spectrum_rect: Rect,

    /// A Perlin noise contour generator.
    pub contours: Arc<RwLock<ContoursGPU>>,
    /// A SmoothLife simulation.
    pub smooth_life: Arc<RwLock<SmoothLifeGPU>>,
    /// A Voronoi noise generator used for the spectral mask.
    pub voronoi_mask: Arc<RwLock<VoronoiGPU>>,
    /// A vector field used to manage points for the Voronoi mask.
    pub voronoi_vectors: Arc<RwLock<VectorField>>,

    /// A simple vector field for the resonator bank points.
    pub vectors_reso_bank: VectorField,
    /// A channel to receive a message when the number of resonators has changed.
    resonator_count_receiver: Receiver<()>,
    /// The voronoi generator for the resonator bank vector field.
    pub voronoi_reso_bank: VoronoiGPU,
    /// The line which shows which column is being used as a spectral mask.
    pub mask_scan_line_pos: f64,
    /// The amount to increment the position of the mask scan line each frame.
    pub mask_scan_line_increment: f64,

    /// The EQ display — filter nodes and the frequency response line.
    pub eq_display: EQDisplay,

    /// Logarithmic frequency lines drawn in the background of the spectrogram.
    pub log_lines: Vec<[Vec2; 2]>,

    /// User input data.
    pub input_data: InputData,

    /// The time since the last called to [`update()`], used for calculating the frame
    /// delta time.
    last_frame_time: Instant,
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
        let reso_bank_reset_sender_key = reso_bank_reset_sender.clone();

        let (reso_bank_push_sender, reso_bank_push_receiver) = unbounded();
        let reso_bank_push_sender_key = reso_bank_push_sender.clone();

        let voronoi_vectors = Arc::new(RwLock::new(voronoi_vectors));
        let vv = Arc::clone(&voronoi_vectors);

        let (m_tl, m_br) = (mask_rect.top_left(), mask_rect.bottom_right());

        let mut ui_components = UIComponents::new(&params)
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

        let rb_count = Arc::clone(&params.reso_bank_resonator_count);
        let (resontor_count_sender, resonator_count_receiver) = unbounded();

        ui_components
            .reso_bank_resonator_count
            .set_callback(move |_, val| {
                rb_count.sr(val as u32);
                resontor_count_sender.send(()).unwrap();
            });

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
            eq_display: EQDisplay::new(
                spectrum_rect,
                &params,
                Arc::clone(&sample_rate_ref),
            ),

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
            reso_bank_reset_sender_key,
            reso_bank_push_receiver,
            reso_bank_push_sender_key,

            reso_bank_data,
            spectrum_rect,

            mouse_clicked_outside_of_mask: false,
            mask_clicked: false,

            contours,
            smooth_life,

            voronoi_mask: Arc::new(RwLock::new(voronoi_mask)),
            voronoi_vectors,

            voronoi_reso_bank: VoronoiGPU::new(app, bank_rect),
            vectors_reso_bank,
            resonator_count_receiver,

            log_lines: create_log_lines(&spectrum_rect),

            mask_scan_line_pos: 0.0,
            mask_scan_line_increment: 0.1,

            input_data: InputData {
                is_win_focussed: true, // required for the window to be initialized on Windows
                ..Default::default()
            },

            sample_rate_ref,

            last_frame_time: Instant::now(),
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

    /// Increments the position of the mask scan line.
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
    pub fn update_input_data(&mut self, app: &App) {
        self.input_data.delta_time =
            self.last_frame_time.elapsed().as_secs_f64();
        self.last_frame_time = Instant::now();
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

    /// Updates the resonator bank vector field.
    pub fn update_reso_bank_vector_field(&mut self, app: &App) {
        let reso_count = self.ui_params.reso_bank_resonator_count.lr() as usize;
        if reso_count != self.vectors_reso_bank.num_active_points {
            self.vectors_reso_bank.set_num_active_points(reso_count);
        }

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

        // this lazily updates the voronoi lines.
        if self.reso_bank_needs_redraw() {
            self.voronoi_reso_bank
                .copy_from_vectors(&self.vectors_reso_bank);

            self.voronoi_reso_bank.update(app, &self.input_data);
        }

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

    /// Updates the vector field used for the Voronoi noise spectral mask.
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

    /// Updates the mask scan line based on user mouse input.
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

    /// Updates the EQ GUI.
    pub fn update_eq(&mut self, app: &App) {
        if self.ui_components.exciter_osc.is_open() {
            self.eq_display.clicked_outside_of_spectrum = true;
            return;
        }

        self.eq_display.update(app, &self.input_data);

        // if the eq is being updated, update the ui components
        if self.input_data.is_left_clicked
            && (self.spectrum_rect.contains(self.input_data.mouse_pos)
                || self.eq_display.spectrum_is_clicked)
            && !self.eq_display.clicked_outside_of_spectrum
        {
            let EQFilterParams { cutoff, gain, q } =
                self.eq_display.low_filter_params;

            self.ui_components
                .low_filter_cutoff
                .set_value(freq_to_note(cutoff));
            self.ui_components.low_filter_gain.set_value(gain);
            self.ui_components.low_filter_q.set_value(q.recip());

            let EQFilterParams { cutoff, gain, q } =
                self.eq_display.peak_filter_params;
            self.ui_components
                .peak_filter_cutoff
                .set_value(freq_to_note(cutoff));
            self.ui_components.peak_filter_gain.set_value(gain);
            self.ui_components.peak_filter_q.set_value(q.recip());

            let EQFilterParams { cutoff, gain, q } =
                self.eq_display.high_filter_params;
            self.ui_components
                .high_filter_cutoff
                .set_value(freq_to_note(cutoff));
            self.ui_components.high_filter_gain.set_value(gain);
            self.ui_components.high_filter_q.set_value(q.recip());
        }
        // otherwise if a ui parameter is being changed, set the eq params
        else if self.input_data.is_left_clicked
            && self.eq_display.clicked_outside_of_spectrum
        {
            self.eq_display.low_filter_params.cutoff =
                note_to_freq(self.ui_components.low_filter_cutoff.value());
            self.eq_display.low_filter_params.gain =
                self.ui_components.low_filter_gain.value();
            self.eq_display.low_filter_params.q =
                self.ui_components.low_filter_q.value().recip();

            self.eq_display.peak_filter_params.cutoff =
                note_to_freq(self.ui_components.peak_filter_cutoff.value());
            self.eq_display.peak_filter_params.gain =
                self.ui_components.peak_filter_gain.value();
            self.eq_display.peak_filter_params.q =
                self.ui_components.peak_filter_q.value().recip();

            self.eq_display.high_filter_params.cutoff =
                note_to_freq(self.ui_components.high_filter_cutoff.value());
            self.eq_display.high_filter_params.gain =
                self.ui_components.high_filter_gain.value();
            self.eq_display.high_filter_params.q =
                self.ui_components.high_filter_q.value().recip();
        }
    }

    /// Draws the spectrogram log lines.
    pub fn draw_log_lines(&self, draw: &Draw) {
        for line in &self.log_lines {
            draw.line()
                .points(line[0], line[1])
                .weight(2.0)
                .color(Rgb::new(0.08, 0.08, 0.08));
        }
    }

    /// Determins whether the resonator bank needs to be redrawn.
    ///
    /// TODO: for some reason the resonator bank vector field and voronoi noise
    /// do not update immediately, and it seems like the voronoi generator updates
    /// a frame or two (or three?) later. For now, this method is just used to determine
    /// whether to update the voronoi generator each frame, and not when to draw it.
    pub fn reso_bank_needs_redraw(&self) -> bool {
        if self.ui_components.reso_bank_scale.needs_redraw()
            || self.vectors_reso_bank.can_mouse_interact
            || self.resonator_count_receiver.try_recv().is_ok()
        {
            return true;
        }

        for pt in self
            .vectors_reso_bank
            .points
            .iter()
            .take(self.vectors_reso_bank.num_active_points)
        {
            if pt.vel.x.abs() > f32::EPSILON || pt.vel.y.abs() > f32::EPSILON {
                return true;
            }
        }

        false
    }

    /// Redraws components under expanded when required.
    pub fn redraw_under_menus(&self, draw: &Draw, is_first_frame: bool) {
        let UIComponents {
            mask_algorithm,
            mask_resolution,
            contour_count,
            contour_speed,
            contour_thickness,
            smoothlife_preset,
            reso_bank_scale,
            exciter_osc,
            spectrogram_label,
            dist_type,
            delay_feedback,
            ..
        } = &self.ui_components;

        if mask_algorithm.needs_redraw() {
            let rect = mask_algorithm.rect();
            draw.rect().xy(rect.xy()).wh(rect.wh()).color(BLACK);
        }

        if mask_resolution.needs_redraw() {
            let rect = mask_resolution.rect();
            draw.rect().xy(rect.xy()).wh(rect.wh()).color(BLACK);
        }

        if reso_bank_scale.needs_redraw() {
            let rect = reso_bank_scale.rect();
            draw.rect().xy(rect.xy()).wh(rect.wh()).color(BLACK);
        }

        if exciter_osc.needs_redraw() {
            let rect = exciter_osc.rect();
            draw.rect().xy(rect.xy()).wh(rect.wh()).color(BLACK);

            let label_rect = spectrogram_label.rect();
            draw.rect()
                .xy(label_rect.xy())
                .wh(label_rect.wh())
                .color(BLACK);
        }

        if dist_type.needs_redraw() {
            let rect = dist_type.rect();
            draw.rect().xy(rect.xy()).wh(rect.wh()).color(BLACK);
        }
    }

    /// Redraws the filter text sliders when filter types are changed.
    pub fn redraw_filter_sliders(&self, draw: &Draw) {
        let hf_changed = self.ui_components.high_filter_type.was_just_changed();
        let lf_changed = self.ui_components.low_filter_type.was_just_changed();

        if hf_changed {
            let is_shelf = self.ui_components.high_filter_type.enabled();
            let rect = self.ui_components.high_filter_gain.rect();
            let h = rect.h();
            draw.rect()
                .xy(pt2(rect.x(), rect.y() + h * 0.5))
                .wh(pt2(rect.w(), rect.h() * 2.0))
                .color(BLACK);

            if is_shelf {
                self.ui_components.high_filter_gain.redraw_label(draw);
            }
            else {
                self.ui_components.high_filter_q.redraw_label(draw);
            }
        }

        if lf_changed {
            let is_shelf = self.ui_components.low_filter_type.enabled();
            let rect = self.ui_components.low_filter_gain.rect();
            let h = rect.h();
            draw.rect()
                .xy(pt2(rect.x(), rect.y() + h * 0.5))
                .wh(pt2(rect.w(), rect.h() * 2.0))
                .color(BLACK);

            if is_shelf {
                self.ui_components.low_filter_gain.redraw_label(draw);
            }
            else {
                self.ui_components.low_filter_q.redraw_label(draw);
            }
        }
    }
}

/// Computes the position of each log line in the spectrogram.
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

/// Log values intended to represent the logarithmic scaling from 10 Hz to 30 kHz.
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
