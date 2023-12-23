use nannou_audio::Stream;

use nannou_egui::{self, egui, Egui};

use super::view::view;
use super::*;
use super::{audio::*, sequencer::Sequencer};
use crate::app::params::*;
use crate::dsp::{ResonatorBankParams, SpectralMask};
use crate::generative::*;
use crate::gui::spectrum::*;
use crate::gui::UIComponents;
use nannou::prelude::WindowId as Id;

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
pub struct Model {
    window: Id,

    pub egui: Egui,
    pub ui_params: UIParams,

    // AUDIO DATA
    /// The CPAL audio stream.
    pub audio_stream: nannou_audio::Stream<AudioModel>,
    /// Channels to send messages directly to the audio thread.
    pub audio_senders: Arc<AudioMessageSenders>,

    /// A thread-safe reference to the mask used for spectral filtering.
    // pub spectral_mask: Arc<Mutex<SpectralMask>>,
    pub spectral_mask: Arc<Mutex<triple_buffer::Input<SpectralMask>>>,

    pub reso_bank_params: ResonatorBankParams,

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

    /// A Perlin noise contour generator.
    pub contours: Option<Arc<RwLock<Contours>>>,
    /// A SmoothLife simulation.
    pub smooth_life: Option<Arc<RwLock<SmoothLife>>>,
    /// The line which shows which column is being used as a spectral mask.
    pub mask_scan_line_pos: f64,
    /// The amount to increment the position of the mask scan line each frame.
    pub mask_scan_line_increment: f64,

    pub input_data: InputData,

    pub sequencer: Sequencer,

    pub mask_thread_pool: ThreadPool,
}

impl Model {
    /// Builds the app's `Model`.
    ///
    /// # Panics
    ///
    /// Panics if a new window cannot be initialized.
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
        } = build_audio_system(MAX_SPECTRAL_BLOCK_SIZE, &params);

        let (_w, _h) = (WINDOW_SIZE.x as f32, WINDOW_SIZE.y as f32);

        let window =
            build_window(app, WINDOW_SIZE.x as u32, WINDOW_SIZE.y as u32);

        let GuiElements {
            bank_rect,
            contours,
            smooth_life,
            pre_spectrum_analyzer,
            post_spectrum_analyzer,
            dsp_load,
        } = build_gui_elements(app, pre_spectrum, post_spectrum, &params);

        let sequencer = Sequencer::new(
            sample_rate_ref.lr(),
            audio_senders.note_event.clone(),
        );

        let egui = Egui::from_window(
            &app.window(window).expect("expected a valid window id"),
        );

        let audio_senders = Arc::new(audio_senders);
        let cl = Arc::clone(&audio_senders);

        let contours = Arc::new(RwLock::new(contours));
        let smooth_life = Arc::new(RwLock::new(smooth_life));
        let ctr = Arc::clone(&contours);
        let sml = Arc::clone(&smooth_life);
        let gen_algo = Arc::clone(&params.mask_algorithm);

        let ui_components = UIComponents::new(&params)
            .attach_reso_bank_randomise_callback(move |_| {
                cl.resonator_bank_reset_pitch.send(());
            })
            .attach_mask_reset_callback(move |_| {
                if let Ok(guard) = gen_algo.read() {
                    match *guard {
                        GenerativeAlgo::Contours => {
                            if let Ok(mut guard) = ctr.write() {
                                guard.reset_seed();
                            }
                        }
                        GenerativeAlgo::SmoothLife => {
                            if let Ok(mut guard) = sml.write() {
                                guard.reset();
                            }
                        }
                    }
                }
            });

        Self {
            window,

            egui,
            ui_components,
            ui_params: params,

            audio_stream,
            audio_senders,

            octave: Octave::default(), // C3 - B3

            note_handler: Arc::clone(&note_handler),

            reso_bank_params: ResonatorBankParams {
                root_note: 52.0,
                scale: Scale::MajPentatonic,
                quantise_to_scale: true,
                freq_spread: 0.7,
                freq_shift: 0.0,
                inharm: 0.3,
            },

            pressed_keys: build_pressed_keys_map(),

            audio_callback_timer,

            pre_spectrum_analyzer,
            post_spectrum_analyzer,

            voice_event_sender,

            spectral_mask: Arc::new(Mutex::new(spectral_mask)),

            bank_rect,

            contours: Some(contours),
            smooth_life: Some(smooth_life),

            mask_scan_line_pos: 0.0,
            mask_scan_line_increment: 0.1,

            input_data: InputData::default(),

            sequencer,

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
        self.contours.as_ref().map_or_else(
            || {
                self.smooth_life.as_ref().map_or_else(
                    || unreachable!("this should be unreachable as it is set to Some before this can be called."),
                    |sml| *sml.read().unwrap().rect(),
                )
            },
            |ctr| *ctr.read().unwrap().rect(),
        )
    }

    pub fn draw_mask_scan_line(&self, draw: &Draw) {
        let rect = self.mask_rect();
        let y_bot = rect.bottom();
        let y_top = rect.top();

        let x = map(
            self.mask_scan_line_pos,
            0.0,
            1.0,
            rect.left() as f64,
            rect.right() as f64,
        ) as f32;

        draw.line()
            .points(pt2(x, y_bot), pt2(x, y_top))
            .weight(4.0)
            .color(rgba::<u8>(0, 200, 0, 100));
    }

    /// Updates the model's input data.
    pub fn update_input_data(&mut self, app: &App, update: Update) {
        self.egui.set_elapsed_time(update.since_start);

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
}
