use nannou::window::Id;
use nannou_audio::Stream;

use super::audio::*;
use super::view::view;
use super::*;
use crate::dsp::SpectralMask;
use crate::generative::*;
use crate::gui::spectrum::*;
use crate::musical::*;

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{mpsc, Arc, Mutex, RwLock},
    time::Instant,
};

mod constructors;
use constructors::*;

type CallbackTimerRef = Arc<Mutex<Instant>>;

/// The app's model, i.e. its general state.
pub struct Model {
    window: Id,

    // AUDIO DATA
    /// The CPAL audio stream.
    pub audio_stream: nannou_audio::Stream<AudioModel>,
    /// Channels to send messages directly to the audio thread.
    pub audio_senders: AudioSenders,

    /// A thread-safe reference to the mask used for spectral filtering.
    // pub spectral_mask: Arc<Mutex<SpectralMask>>,
    pub spectral_mask: Arc<Mutex<triple_buffer::Input<SpectralMask>>>,

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
    /// The time since the last call to `update()`.
    pub delta_time: f64,
    update_time: Instant,

    /// A Perlin noise contour generator.
    pub contours: Option<Arc<RwLock<Contours>>>,
    /// A SmoothLife simulation.
    pub smooth_life: Option<Arc<RwLock<SmoothLife>>>,
    /// The line which shows which column is being used as a spectral mask.
    pub mask_scan_line_pos: f64,
    /// The amount to increment the position of the mask scan line each frame.
    pub mask_scan_line_increment: f64,

    pub mask_thread_pool: ThreadPool,

    pub current_gen_algo: GenerativeAlgo,
}

impl Model {
    /// Builds the app's `Model`.
    ///
    /// # Panics
    ///
    /// Panics if a new window cannot be initialized.
    pub fn build(app: &App) -> Self {
        let max_spectral_block_size = 1 << 14; // 16,384
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
        } = build_audio_system(max_spectral_block_size);

        let (_w, _h) = (WINDOW_SIZE.x as f32, WINDOW_SIZE.y as f32);

        let window =
            build_window(app, WINDOW_SIZE.x as u32, WINDOW_SIZE.y as u32);

        let GuiElements {
            contours,
            smooth_life,
            pre_spectrum_analyzer,
            post_spectrum_analyzer,
            dsp_load,
        } = build_gui_elements(app, pre_spectrum, post_spectrum);

        let current_gen_algo = GenerativeAlgo::Contours;

        Self {
            window,

            audio_stream,
            audio_senders,

            octave: Octave::default(), // C3 - B3

            note_handler: Arc::clone(&note_handler),

            pressed_keys: build_pressed_keys_map(),

            audio_callback_timer,

            pre_spectrum_analyzer,
            post_spectrum_analyzer,

            voice_event_sender,

            spectral_mask: Arc::new(Mutex::new(spectral_mask)),

            contours: match current_gen_algo {
                GenerativeAlgo::Contours => {
                    Some(Arc::new(RwLock::new(contours)))
                }
                GenerativeAlgo::SmoothLife => None,
            },
            smooth_life: match current_gen_algo {
                GenerativeAlgo::Contours => None,
                GenerativeAlgo::SmoothLife => {
                    Some(Arc::new(RwLock::new(smooth_life)))
                }
            },
            mask_scan_line_pos: 0.0,
            mask_scan_line_increment: 0.1,

            mask_thread_pool: ThreadPool::build(2)
                .expect("failed to build mask thread pool"),

            dsp_load,
            sample_rate_ref,

            delta_time: 0.0,
            update_time: Instant::now(),

            current_gen_algo,
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
        self.mask_scan_line_pos +=
            self.mask_scan_line_increment * self.delta_time;

        if self.mask_scan_line_pos > 1.0 {
            self.mask_scan_line_pos -= 1.0;
        }
        else if self.mask_scan_line_pos < 0.0 {
            self.mask_scan_line_pos += 1.0;
        }
    }

    pub fn set_mask(&mut self, mask: GenerativeAlgo) {
        todo!();
    }

    pub fn mask_rect(&self) -> Rect {
        self.contours.as_ref().map_or_else(
            || {
                self.smooth_life.as_ref().map_or_else(
                    || unreachable!(),
                    |sml| sml.read().unwrap().rect(),
                )
            },
            |ctr| ctr.read().unwrap().rect(),
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

    pub fn get_delta_time(&mut self) -> f64 {
        self.delta_time = self.update_time.elapsed().as_secs_f64();
        self.update_time = Instant::now();

        self.delta_time
    }
}

pub enum GenerativeAlgo {
    Contours,
    SmoothLife,
}
