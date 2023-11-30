use nannou::window::Id;
use nannou_audio::Stream;

use super::audio::*;
use super::view::view;
use super::*;
use crate::dsp::SpectralMask;
use crate::generative::*;
use crate::gui::spectrum::*;
use crate::musical::*;
use nannou::image::Rgba;

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};
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
    pub spectral_mask: Arc<Mutex<SpectralMask>>,

    /// A thread-safe reference to the timer which tracks when the audio callback
    /// was last called.
    pub audio_callback_timer: CallbackTimerRef,

    /// A string showing the (rough) DSP load.
    pub dsp_load: Option<String>,

    /// A reference to the sample rate value.
    pub sample_rate_ref: Arc<AtomicF64>,

    /// The time since the last call to `update()`.
    pub delta_time: f64,
    update_time: Instant,

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
    // unfortunately the view function has to take an immutable
    // reference to the Model, so RefCell it is...
    /// The post-FX spectrogram.
    pub post_spectrum_analyzer: RefCell<SpectrumAnalyzer>,

    /// A Perlin noise contour generator.
    pub contours: Contours,
    /// The line which shows which column is being used as a spectral mask.
    pub mask_scan_line_pos: f64,
    /// The amount to increment the position of the mask scan line each frame.
    pub mask_scan_line_increment: f64,
}

impl Model {
    /// Builds the app's `Model`.
    ///
    /// # Panics
    ///
    /// Panics if a new window cannot be initialized.
    pub fn build(app: &App) -> Self {
        let AudioSystem {
            stream: audio_stream,
            sample_rate_ref,
            senders: audio_senders,
            callback_timer_ref: audio_callback_timer,
            note_handler,
            pre_spectrum,
            post_spectrum,
            spectral_mask,
        } = build_audio_system();

        let (_w, _h) = (WINDOW_SIZE.x as f32, WINDOW_SIZE.y as f32);

        let window =
            build_window(app, WINDOW_SIZE.x as u32, WINDOW_SIZE.y as u32);

        let GuiElements {
            contours,
            pre_spectrum_analyzer,
            post_spectrum_analyzer,
            dsp_load,
        } = build_gui_elements(app, pre_spectrum, post_spectrum);

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

            spectral_mask,

            contours,
            mask_scan_line_pos: 0.0,
            mask_scan_line_increment: 0.0,

            dsp_load,
            sample_rate_ref,

            delta_time: 0.0,
            update_time: Instant::now(),
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
        self.mask_scan_line_pos += self.mask_scan_line_increment * self.delta_time;

        if self.mask_scan_line_pos > 1.0 {
            self.mask_scan_line_pos -= 1.0;
        }
        else if self.mask_scan_line_pos < 0.0 {
            self.mask_scan_line_pos += 1.0;
        }
    }

    pub fn draw_mask_scan_line(&self, draw: &Draw) {
        let contour_rect = self.contours.rect();
        let y_bot = contour_rect.bottom();
        let y_top = contour_rect.top();

        let x = map(
            self.mask_scan_line_pos,
            0.0,
            1.0,
            contour_rect.left() as f64,
            contour_rect.right() as f64,
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

/// Builds the app window.
fn build_window(app: &App, width: u32, height: u32) -> Id {
    app.new_window()
        .size(width, height)
        .key_pressed(key::key_pressed)
        .key_released(key::key_released)
        .mouse_moved(mouse::mouse_moved)
        .view(view)
        .build()
        .expect("failed to build app window!")
}

struct AudioSystem {
    stream: Stream<AudioModel>,
    sample_rate_ref: Arc<AtomicF64>,
    senders: AudioSenders,
    callback_timer_ref: CallbackTimerRef,
    note_handler: NoteHandlerRef,
    pre_spectrum: SpectrumOutput,
    post_spectrum: SpectrumOutput,
    spectral_mask: Arc<Mutex<SpectralMask>>,
}

/// Builds the audio stream, audio message channel senders, and input note handler.
fn build_audio_system() -> AudioSystem {
    // setup audio structs
    let note_handler = Arc::new(Mutex::new(NoteHandler::new()));
    let audio_context =
        AudioContext::build(Arc::clone(&note_handler), unsafe { SAMPLE_RATE });
    let mut audio_model = AudioModel::new(audio_context);

    audio_model.initialize();

    // obtain audio message channels
    let senders = audio_model.message_channels();

    let (pre_spectrum, post_spectrum) = audio_model.spectrum_outputs();

    let spectral_mask = audio_model.spectral_mask();

    let callback_timer_ref = audio_model.callback_timer_ref();

    let sample_rate_ref = audio_model.sample_rate_ref();

    // setup audio stream
    let audio_host = nannou_audio::Host::new();
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio::process)
        .channels(NUM_CHANNELS)
        .sample_rate(unsafe { SAMPLE_RATE } as u32)
        .frames_per_buffer(BUFFER_SIZE)
        .build()
        .unwrap();

    stream.play().unwrap();

    AudioSystem {
        stream,
        sample_rate_ref,
        senders,
        callback_timer_ref,
        note_handler,
        pre_spectrum,
        post_spectrum,
        spectral_mask,
    }
}

struct GuiElements {
    pub contours: Contours,

    pub pre_spectrum_analyzer: RefCell<SpectrumAnalyzer>,
    pub post_spectrum_analyzer: RefCell<SpectrumAnalyzer>,

    pub dsp_load: Option<String>,
}

fn build_gui_elements(
    app: &App,
    pre_spectrum: SpectrumOutput,
    post_spectrum: SpectrumOutput,
) -> GuiElements {
    let contour_size = 256;
    let contour_size_fl = (contour_size / 2) as f32;
    let contour_rect = Rect::from_corners(
        pt2(-contour_size_fl, -contour_size_fl),
        pt2(contour_size_fl, contour_size_fl),
    );

    let spectrum_rect =
        Rect::from_corners(pt2(178.0, -128.0), pt2(650.0, 128.0));
    let pre_spectrum_analyzer =
        RefCell::new(SpectrumAnalyzer::new(pre_spectrum, spectrum_rect));
    let post_spectrum_analyzer =
        RefCell::new(SpectrumAnalyzer::new(post_spectrum, spectrum_rect));

    GuiElements {
        contours: Contours::new(app.main_window().device(), contour_rect)
            .with_num_threads(8)
            .expect("failed to allocate 8 threads to contour generator")
            .with_z_increment(0.1)
            .with_num_contours(64)
            .with_contour_range(0.1..=0.9),

        pre_spectrum_analyzer,
        post_spectrum_analyzer,

        dsp_load: None,
    }
}

/// Builds the `HashMap` used to track which keys are currently pressed or not.
fn build_pressed_keys_map() -> HashMap<Key, bool> {
    let mut map = HashMap::new();

    for k in KEYBOARD_MIDI_NOTES {
        map.insert(k, false);
    }

    map
}
