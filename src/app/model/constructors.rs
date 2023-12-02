use super::*;

/// Builds the app window.
pub fn build_window(app: &App, width: u32, height: u32) -> Id {
    app.new_window()
        .size(width, height)
        .key_pressed(key::key_pressed)
        .key_released(key::key_released)
        .mouse_moved(mouse::mouse_moved)
        .view(view)
        .build()
        .expect("failed to build app window!")
}

pub struct AudioSystem {
    pub(super) stream: Stream<AudioModel>,
    pub(super) sample_rate_ref: Arc<AtomicF64>,
    pub(super) senders: AudioSenders,
    pub(super) callback_timer_ref: CallbackTimerRef,
    pub(super) note_handler: NoteHandlerRef,
    pub(super) pre_spectrum: SpectrumOutput,
    pub(super) post_spectrum: SpectrumOutput,
    pub(super) spectral_mask: Arc<Mutex<SpectralMask>>,
}

/// Builds the audio stream, audio message channel senders, and input note handler.
pub fn build_audio_system() -> AudioSystem {
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

pub struct GuiElements {
    pub(super) contours: Contours,
    pub(super) smooth_life: SmoothLife,

    pub(super) pre_spectrum_analyzer: RefCell<SpectrumAnalyzer>,
    pub(super) post_spectrum_analyzer: RefCell<SpectrumAnalyzer>,

    pub(super) dsp_load: Option<String>,
}

pub fn build_gui_elements(
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
            .with_num_contours(20)
            .with_contour_range(0.1..=0.9),
        smooth_life: SmoothLife::new(app.main_window().device(), contour_rect),

        pre_spectrum_analyzer,
        post_spectrum_analyzer,

        dsp_load: None,
    }
}

/// Builds the `HashMap` used to track which keys are currently pressed or not.
pub fn build_pressed_keys_map() -> HashMap<Key, bool> {
    let mut map = HashMap::new();

    for k in KEYBOARD_MIDI_NOTES {
        map.insert(k, false);
    }

    map
}
