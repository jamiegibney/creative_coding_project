use super::audio_constructor::build_audio_model;
use super::*;
use crate::app::audio::audio_constructor::MAX_NUM_RESONATORS;
use crate::dsp::ResoBankData;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc;

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
        .title("Jamie Gibney — Creative Coding Project")
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
    pub(super) reso_bank_data: triple_buffer::Input<ResoBankData>,
}

/// Builds the audio stream, audio message channel senders, and input note handler.
pub fn build_audio_system(params: &UIParams) -> AudioSystem {
    // setup audio structs
    let note_handler = Arc::new(Mutex::new(NoteHandler::new()));
    let (spectral_mask, spectral_mask_output) =
        triple_buffer::TripleBuffer::new(&SpectralMask::new(
            MAX_SPECTRAL_BLOCK_SIZE,
        ))
        .split();

    let (reso_bank_data, reso_bank_data_output) =
        triple_buffer::TripleBuffer::new(&ResoBankData::new(
            MAX_NUM_RESONATORS,
        ))
        .split();

    let (voice_event_sender, voice_event_receiver) = mpsc::channel();
    let (note_channel_sender, note_channel_receiver) = mpsc::channel();

    // build the audio context
    let audio_context = AudioContext {
        note_channel_receiver,
        sample_rate: unsafe { SAMPLE_RATE },
        spectral_mask_output: Some(spectral_mask_output),
        reso_bank_data_output: Some(reso_bank_data_output),
        voice_event_sender: voice_event_sender.clone(),
        voice_event_receiver: Some(voice_event_receiver),
    };

    let AudioPackage {
        model: audio_model,
        spectrum_outputs: (pre_spectrum, post_spectrum),
        callback_timer_ref,
        sample_rate_ref,
        message_channels: senders,
    } = build_audio_model(audio_context, params);

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
        reso_bank_data,
    }
}

pub struct GuiElements {
    pub(super) mask_rect: Rect,
    pub(super) bank_rect: Rect,
    pub(super) spectrum_rect: Rect,

    pub(super) contours: ContoursGPU,
    pub(super) smooth_life: SmoothLifeGPU,
    pub(super) voronoi_mask: VoronoiGPU,
    pub(super) voronoi_vectors: Vectors,

    pub(super) vectors_reso_bank: Vectors,

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

    let spectrum_rect =
        Rect::from_corners(pt2(-540.0, -310.0), pt2(128.0, -40.0));

    let pre_spectrum_line_color = Rgba::new(0.2, 0.2, 0.2, 1.0);
    let post_spectrum_mesh_color = Rgba::new(0.9, 0.4, 0.0, 0.3);

    let line_weight = 2.0;
    let mut pre_spectrum_analyzer =
        RefCell::new(SpectrumAnalyzer::new(pre_spectrum, spectrum_rect));
    pre_spectrum_analyzer
        .borrow_mut()
        .set_line_color(pre_spectrum_line_color);
    let mut post_spectrum_analyzer =
        RefCell::new(SpectrumAnalyzer::new(post_spectrum, spectrum_rect));
    post_spectrum_analyzer
        .borrow_mut()
        .set_mesh_color(post_spectrum_mesh_color);

    GuiElements {
        bank_rect,
        mask_rect,
        spectrum_rect,

        // contours: Contours::new(app.main_window().device(), mask_rect)
        //     .with_num_threads(16)
        //     .expect("failed to allocate threads to contour generator")
        //     .with_feathering(false)
        //     .with_z_increment(params.contour_speed.lr())
        //     .with_num_contours(params.contour_count.lr())
        //     .with_contour_range(0.0..=params.contour_thickness.lr()),
        contours: ContoursGPU::new(app, mask_rect)
            .with_z_increment(params.contour_speed.lr() as f32)
            .with_num_contours(params.contour_count.lr())
            .with_contour_range(0.0..=(params.contour_thickness.lr() as f32)),
        smooth_life: SmoothLifeGPU::new(app, mask_rect),
        voronoi_mask: VoronoiGPU::new(app, mask_rect),
        voronoi_vectors: Vectors::new(MAX_NUM_RESONATORS, mask_rect)
            .with_point_radius(5.0),

        vectors_reso_bank: Vectors::new(MAX_NUM_RESONATORS, bank_rect)
            .with_point_radius(5.0)
            .with_point_color(Rgba::new(0.9, 0.4, 0.0, 0.6)),

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
