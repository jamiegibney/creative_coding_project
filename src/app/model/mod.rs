use nannou_audio::Stream;

use super::audio::*;
use super::view::view;
use super::*;
use crate::gui::spectrum::*;
use crate::musical::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Instant,
};

type CallbackTimerRef = Arc<Mutex<Instant>>;

/// The app's model, i.e. its general state.
pub struct Model {
    window: window::Id,

    pub audio_stream: nannou_audio::Stream<AudioModel>,
    pub audio_senders: AudioSenders,

    pub octave: Octave,
    pub note_handler: NoteHandlerRef,

    pub pressed_keys: HashMap<Key, bool>,

    // unfortunately the view function has to take an immutable reference
    // to the Model, so RefCell it is...
    pub pre_spectrum_analyzer: RefCell<SpectrumAnalyzer>,
    pub post_spectrum_analyzer: RefCell<SpectrumAnalyzer>,

    pub audio_callback_timer: CallbackTimerRef,
}

impl Model {
    /// Builds the app's `Model`.
    pub fn build(app: &App) -> Self {
        let AudioSystem {
            stream: audio_stream,
            senders: audio_senders,
            callback_timer_ref: audio_callback_timer,
            note_handler,
            pre_spectrum,
            post_spectrum,
        } = build_audio_system();

        let (w, h) = (WINDOW_SIZE.x as f32, WINDOW_SIZE.y as f32);
        let rect =
            Rect::from_corners(pt2(-w / 2.0, -h / 2.0), pt2(w / 2.0, h / 2.0));

        let pre_spectrum_analyzer =
            RefCell::new(SpectrumAnalyzer::new(pre_spectrum, rect));
        let post_spectrum_analyzer =
            RefCell::new(SpectrumAnalyzer::new(post_spectrum, rect));

        Self {
            window: app
                .new_window()
                .size(WINDOW_SIZE.x as u32, WINDOW_SIZE.y as u32)
                .key_pressed(key::key_pressed)
                .key_released(key::key_released)
                .mouse_moved(mouse::mouse_moved)
                .view(view)
                .build()
                .expect("failed to build app window!"),

            audio_stream,
            audio_senders,

            octave: Octave::default(), // C3 - B3

            note_handler: Arc::clone(&note_handler),

            pressed_keys: build_pressed_keys_map(),

            audio_callback_timer,

            pre_spectrum_analyzer,
            post_spectrum_analyzer,
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
}

/// Builds the app window.
fn build_window(app: &App) -> window::Id {
    app.new_window()
        .size(1400, 800)
        .key_pressed(key::key_pressed)
        .key_released(key::key_released)
        .mouse_moved(mouse::mouse_moved)
        .view(view)
        .build()
        .expect("failed to build app window!")
}

struct AudioSystem {
    stream: Stream<AudioModel>,
    senders: AudioSenders,
    callback_timer_ref: CallbackTimerRef,
    note_handler: NoteHandlerRef,
    pre_spectrum: SpectrumOutput,
    post_spectrum: SpectrumOutput,
}

/// Builds the audio stream, audio message channel senders, and input note handler.
fn build_audio_system() -> AudioSystem {
    // setup audio structs
    let note_handler = Arc::new(Mutex::new(NoteHandler::new()));
    let audio_context = AudioContext::build(Arc::clone(&note_handler));
    let mut audio_model = AudioModel::new(audio_context);

    audio_model.initialize();

    // obtain audio message channels
    let senders = audio_model.message_channels();

    let (pre_spectrum, post_spectrum) = audio_model.spectrum_outputs();

    let callback_timer_ref = audio_model.callback_timer_ref();

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
        senders,
        callback_timer_ref,
        note_handler,
        pre_spectrum,
        post_spectrum,
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

// unused
fn build_clock_thread() -> (JoinHandle<()>, Arc<Mutex<f64>>) {
    let interval = std::time::Duration::from_secs_f64(
        BUFFER_SIZE as f64 / unsafe { SAMPLE_RATE },
    );
    let clock = Arc::new(Mutex::new(0.0));
    let elapsed = Arc::clone(&clock);

    let thread = thread::spawn(move || loop {
        let start = std::time::Instant::now();

        thread::sleep(interval);

        let mut guard = elapsed.lock().unwrap();
        *guard = start.elapsed().as_secs_f64();
    });

    (thread, Arc::clone(&clock))
}
