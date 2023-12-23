use super::audio_constructor::build_audio_model;
use super::*;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc;

fn egui_raw_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

/// Builds the app window.
pub fn build_window(app: &App, width: u32, height: u32) -> Id {
    app.new_window()
        .size(width, height)
        .resizable(false)
        .msaa_samples(4)
        .key_pressed(key::key_pressed)
        .key_released(key::key_released)
        .mouse_moved(mouse::mouse_moved)
        .event(event)
        .view(view)
        // .raw_event(egui_raw_event)
        .build()
        .expect("failed to build app window!")
}

pub struct AudioSystem {
    pub(super) stream: Stream<AudioModel>,
    pub(super) sample_rate_ref: Arc<AtomicF64>,
    pub(super) senders: AudioMessageSenders,
    pub(super) callback_timer_ref: CallbackTimerRef,
    pub(super) note_handler: NoteHandlerRef,
    pub(super) pre_spectrum: SpectrumOutput,
    pub(super) post_spectrum: SpectrumOutput,
    pub(super) voice_event_sender: mpsc::Sender<VoiceEvent>,
    pub(super) spectral_mask: triple_buffer::Input<SpectralMask>,
}

/// Builds the audio stream, audio message channel senders, and input note handler.
pub fn build_audio_system(spectral_block_size: usize, params: &UIParams) -> AudioSystem {
    // setup audio structs
    let note_handler = Arc::new(Mutex::new(NoteHandler::new()));
    let (spectral_mask, spectral_mask_output) =
        triple_buffer::TripleBuffer::new(&SpectralMask::new(spectral_block_size).with_size(512))
            .split();

    let (voice_event_sender, voice_event_receiver) = mpsc::channel();
    let (note_channel_sender, note_channel_receiver) = mpsc::channel();

    // build the audio context
    let audio_context = AudioContext {
        // note_handler: Arc::clone(&note_handler),
        note_channel_receiver,
        sample_rate: unsafe { SAMPLE_RATE },
        spectral_mask_output: Some(spectral_mask_output),
        voice_event_sender: voice_event_sender.clone(),
        voice_event_receiver: Some(voice_event_receiver),
    };

    let AudioPackage {
        model: audio_model,
        spectrum_outputs: (pre_spectrum, post_spectrum),
        callback_timer_ref,
        sample_rate_ref,
        message_channels: senders,
    } = build_audio_model(audio_context);

    // setup audio stream
    let audio_host = nannou_audio::Host::new();

    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio::process)
        .channels(NUM_CHANNELS)
        .sample_rate(sample_rate_ref.load(Relaxed) as u32)
        .frames_per_buffer(BUFFER_SIZE)
        .build()
        .unwrap();

    stream.play().unwrap();

    // construct audio system
    AudioSystem {
        stream,
        sample_rate_ref,
        senders,
        callback_timer_ref,
        note_handler,
        pre_spectrum,
        post_spectrum,
        voice_event_sender,
        spectral_mask,
    }
}

pub struct GuiElements {
    pub(super) bank_rect: Rect,

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
    params: &UIParams,
) -> GuiElements {
    let upper_size = 256.0;

    let bank_rect = Rect::from_corners(
        pt2(-540.0, 50.0),
        pt2(-540.0 + upper_size, 50.0 + upper_size),
    );

    let contour_size_fl = upper_size / 2.0;
    let mask_rect = Rect::from_corners(
        pt2(-contour_size_fl, 50.0),
        pt2(contour_size_fl, 50.0 + upper_size),
    );

    let spectrum_rect = Rect::from_corners(pt2(-540.0, -310.0), pt2(128.0, -40.0));

    let pre_spectrum_analyzer = RefCell::new(SpectrumAnalyzer::new(pre_spectrum, spectrum_rect));
    let post_spectrum_analyzer = RefCell::new(SpectrumAnalyzer::new(post_spectrum, spectrum_rect));

    GuiElements {
        bank_rect,

        contours: Contours::new(app.main_window().device(), mask_rect)
            .with_num_threads(8)
            .expect("failed to allocate threads to contour generator")
            .with_feathering(false)
            .with_z_increment(params.contour_speed.lr())
            .with_num_contours(params.contour_count.lr())
            .with_contour_range(0.0..=params.contour_thickness.lr()),
        smooth_life: SmoothLife::new(
            app.main_window().device(),
            mask_rect,
            params
                .smoothlife_resolution
                .try_read()
                .map_or(DEFAULT_SMOOTHLIFE_SIZE, |mut guard| guard.value()),
        ),

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

pub fn build_ui_parameters() -> UIParams {
    UIParams::default()
}
