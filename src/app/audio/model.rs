use super::*;

/// When the DSP stops processing, it will continue to process for this length of time
/// to allow the audio spectrums to fully relax. After this time has passed, the DSP is 
/// skipped to reduce total load when idle.
const IDLE_TIME_SECS: f64 = 0.7;

/// The audio state for the whole program.
pub struct AudioModel {
    /// The time since the last audio callback (sort of).
    ///
    /// It seems as though the audio callback is called twice very quickly,
    /// and then left for nearly two buffers. As a result, this is set up to
    /// track every *other* callback, so only resets every second call. In
    /// other words, it resets after `BUFFER_SIZE * 2` samples (approximately).
    ///
    /// This is *not* very accurate, but it is more than adequate for capturing
    /// note events. Any discrepancies in timing are not noticeable.
    pub callback_time_elapsed: Arc<Mutex<std::time::Instant>>,
    /// A `struct` for handling polyphonic voices.
    pub voice_handler: VoiceHandler,
    /// A general audio context. Holds a reference to the input `NoteHandler`.
    pub context: AudioContext,
    /// Master gain level.
    pub gain: Smoother<f64>,
    /// Amplitude envelope which is cloned to each voice upon spawning.
    pub amp_envelope: AdsrEnvelope,

    /// The pre-FX spectrum processor. This will have a respective `SpectrumOutput`
    /// where the spectral data may be received. Held in an `Arc<Mutex<T>>` so as
    /// to be processed on a separate thread.
    pub pre_spectrum: Arc<Mutex<Option<SpectrumInput>>>,
    /// A buffer for storing the main audio buffer pre-FX. The audio thread copies
    /// its buffer to this cache, and then requests that the spectrum be processed.
    pub pre_buffer_cache: Arc<Mutex<Vec<f64>>>,
    /// The post-FX spectrum processor. This will have a respective `SpectrumOutput`
    /// where the spectral data may be received. Held in an `Arc<Mutex<T>>` so as
    /// to be processed on a separate thread.
    pub post_spectrum: Arc<Mutex<Option<SpectrumInput>>>,
    /// A buffer for storing the main audio buffer post-FX. The audio thread copies
    /// its buffer to this cache, and then requests that the spectrum be processed.
    pub post_buffer_cache: Arc<Mutex<Vec<f64>>>,
    /// A thread pool for processing both the pre- and post-FX spectra. Holds two
    /// threads which are blocked until they receive a closure to process.
    spectrum_thread_pool: ThreadPool,

    pub filter_lp: [BiquadFilter; 2],
    pub filter_hp: [BiquadFilter; 2],
    pub filter_peak: [BiquadFilter; 2],
    pub filter_peak_post: [BiquadFilter; 2],
    pub filter_comb: [IirCombFilter; 2],

    // pub filter_freq: Smoother<f64>,
    filter_freq_receiver: Option<Receiver<f64>>,

    pub waveshaper: [Waveshaper; 2],
    drive_amount_receiver: Option<Receiver<f64>>,

    pub glide_time: f64,
    pub volume: f64,

    /// The DSP oversamplers - one for each channel.
    pub oversamplers: Vec<Oversampler>,
    /// The oversamping factor.
    pub oversampling_factor: Arc<AtomicUsize>,
    /// A buffer to temporarily hold audio data for oversampling. This buffer matches the
    /// audio layout of the oversamplers, and just "relays" the data from the main
    /// audio buffer to the oversamplers.
    // pub oversampling_buffer: Vec<Vec<f64>>,
    pub oversampling_buffer: OversamplingBuffer,

    pub average_load: Vec<f64>,
    pub avr_pos: usize,
    pub is_processing: bool,
    pub idle_timer_samples: usize,

    pub latency_samples: u32,
}

impl AudioModel {
    /// Creates a new `AudioModel`.
    #[allow(clippy::too_many_lines)]
    pub fn new(context: AudioContext) -> Self {
        let sample_rate = unsafe { SAMPLE_RATE };

        unsafe {
            update_oversampled_sample_rate(
                2usize.pow(DEFAULT_OVERSAMPLING_FACTOR as u32),
            );
        }

        let upsampled_rate = unsafe { OVERSAMPLED_SAMPLE_RATE };
        let biquad = BiquadFilter::new(upsampled_rate);

        let mut comb = IirCombFilter::with_interpolation(true, upsampled_rate);
        comb.set_freq(12.0);

        let mut comb_peak = BiquadFilter::new(upsampled_rate);
        comb_peak.set_params(&BiquadParams {
            freq: 726.0,
            gain: 4.0,
            q: 0.4,
            filter_type: FilterType::Peak,
        });

        let mut comb_lp = BiquadFilter::new(upsampled_rate);
        comb_lp.set_params(&BiquadParams {
            freq: 2652.0,
            gain: 0.0,
            q: 2.0,
            filter_type: FilterType::Lowpass,
        });

        let mut comb_lp = FirstOrderFilter::new(upsampled_rate);
        comb_lp.set_type(FilterType::Lowpass);
        comb_lp.set_freq(1000.0);

        let mut comb_comb =
            IirCombFilter::with_interpolation(true, upsampled_rate);
        comb_comb.set_freq(6324.0);
        comb_comb.set_gain_db(-2.0);

        comb.set_internal_filters(vec![
            Box::new(comb_peak),
            Box::new(comb_comb),
            Box::new(comb_lp),
        ]);

        let glide_time = 0.001;

        let mut waveshaper = [Waveshaper::new(), Waveshaper::new()];

        for ws in &mut waveshaper {
            ws.set_curve(0.8);
            ws.set_asymmetric(false);
            ws.set_drive(0.5);
            ws.set_xfer_function(xfer::s_curve_round);
        }

        let note_handler_ref = context.note_handler_ref();

        let mut amp_envelope = AdsrEnvelope::new();
        amp_envelope.set_parameters(10.0, 300.0, 1.0, 10.0);

        // let thread_pool =

        Self {
            callback_time_elapsed: Arc::new(Mutex::new(
                std::time::Instant::now(),
            )),
            voice_handler: VoiceHandler::build(Arc::clone(&note_handler_ref)),
            context,
            gain: Smoother::new(1.0, 0.1),

            pre_spectrum: Arc::new(Mutex::new(None)),
            pre_buffer_cache: Arc::new(Mutex::new(vec![
                0.0;
                BUFFER_SIZE
                    * NUM_CHANNELS
            ])),
            post_spectrum: Arc::new(Mutex::new(None)),
            post_buffer_cache: Arc::new(Mutex::new(vec![
                0.0;
                BUFFER_SIZE
                    * NUM_CHANNELS
            ])),
            spectrum_thread_pool: ThreadPool::build(2).unwrap(),

            filter_lp: [biquad.clone(), biquad.clone()],
            filter_hp: [biquad.clone(), biquad.clone()],
            filter_peak: [biquad.clone(), biquad.clone()],
            filter_comb: [comb.clone(), comb],
            filter_peak_post: [biquad.clone(), biquad.clone()],

            waveshaper,
            drive_amount_receiver: None,

            filter_freq_receiver: None,

            volume: db_to_level(-24.0),

            amp_envelope,
            glide_time,

            oversamplers: vec![
                // a quality factor of 3 offers good resampling quality and decent 
                // performance, including in debug mode.
                Oversampler::new(
                    MAX_BUFFER_SIZE, MAX_OVERSAMPLING_FACTOR, 3
                );
                NUM_CHANNELS
            ],
            oversampling_factor: Arc::new(AtomicUsize::new(
                DEFAULT_OVERSAMPLING_FACTOR,
            )),

            // oversampling_buffer: vec![vec![0.0; MAX_BUFFER_SIZE]; NUM_CHANNELS],
            oversampling_buffer: OversamplingBuffer::new(
                NUM_CHANNELS, MAX_BUFFER_SIZE,
            ),

            average_load: vec![
                0.0;
                if PRINT_DSP_LOAD {
                    DSP_LOAD_AVERAGING_SAMPLES
                }
                else {
                    0
                }
            ],
            avr_pos: 0,
            is_processing: false,
            idle_timer_samples: (unsafe { SAMPLE_RATE } * IDLE_TIME_SECS)
                as usize,

            latency_samples: 0,
        }
    }

    /// Returns the `AudioModel`'s channel senders.
    pub fn message_channels(&mut self) -> AudioSenders {
        let (envelope_trigger_sender, receiver) = channel();

        let (filter_freq_sender, receiver) = channel();
        self.filter_freq_receiver = Some(receiver);

        let (drive_amount_sender, receiver) = channel();
        self.drive_amount_receiver = Some(receiver);

        AudioSenders {
            envelope_trigger: envelope_trigger_sender,
            filter_freq: filter_freq_sender,
            drive_amount: drive_amount_sender,
        }
    }

    /// Returns the pre and post `SpectrumOutput`s for the audio thread.
    pub fn spectrum_outputs(&mut self) -> (SpectrumOutput, SpectrumOutput) {
        let (pre_in, pre_out) = SpectrumInput::new(2);
        let (post_in, post_out) = SpectrumInput::new(2);

        let mut guard = self.pre_spectrum.lock().unwrap();
        *guard = Some(pre_in);
        drop(guard);

        let mut guard = self.post_spectrum.lock().unwrap();
        *guard = Some(post_in);
        drop(guard);

        (pre_out, post_out)
    }

    /// Initializes the `AudioModel`, returning an `AudioSenders` instance containing
    /// the channel senders used to communicate with the audio thread.
    pub fn initialize(&mut self) {
        self.set_filters();
    }

    pub fn compute_pre_spectrum(&mut self) {
        let spectrum = Arc::clone(&self.pre_spectrum);
        let buffer = Arc::clone(&self.pre_buffer_cache);

        self.spectrum_thread_pool.execute(move || {
            if let Some(spectrum) = spectrum.lock().unwrap().as_mut() {
                let buf = buffer.lock().unwrap();
                spectrum.compute(&buf);
            }
        });
    }

    pub fn compute_post_spectrum(&mut self) {
        let spectrum = Arc::clone(&self.post_spectrum);
        let buffer = Arc::clone(&self.post_buffer_cache);

        self.spectrum_thread_pool.execute(move || {
            if let Some(spectrum) = spectrum.lock().unwrap().as_mut() {
                let buf = buffer.lock().unwrap();
                spectrum.compute(&buf);
            }
        });
    }

    /// Sets the initial state of the filters.
    pub fn set_filters(&mut self) {
        let params = BiquadParams {
            freq: 440.0,
            gain: 0.0,
            q: 50.0,
            filter_type: FilterType::Lowpass,
        };

        for lp in &mut self.filter_lp {
            lp.set_params(&params);
            lp.set_freq(100.0);
            lp.set_q(BUTTERWORTH_Q);
        }

        for hp in &mut self.filter_hp {
            hp.set_params(&params);
            hp.set_type(FilterType::Highpass);
        }

        for peak in &mut self.filter_peak {
            peak.set_params(&params);
            peak.set_type(FilterType::Peak);
            peak.set_q(20.0);
            peak.set_gain(30.0);
        }

        for comb in &mut self.filter_comb {
            comb.set_positive_polarity(false);
            comb.set_interpolation(InterpType::Linear);
            comb.set_gain_db(-5.0);
        }

        for peak in &mut self.filter_peak_post {
            peak.set_params(&params);
            peak.set_type(FilterType::Peak);
            peak.set_q(0.3);
            peak.set_gain(18.0);
        }
    }

    /// Sets the filter frequency for all filters.
    pub fn set_filter_freq(&mut self, mut freq: f64) {
        freq = freq.clamp(10.0, unsafe { SAMPLE_RATE } / 2.0);
        for ch in 0..2 {
            // self.filter_lp[ch].set_freq(freq);
            self.filter_hp[ch].set_freq(freq);
            self.filter_peak[ch].set_freq(freq);
            self.filter_comb[ch].set_freq(freq);
            self.filter_peak_post[ch].set_freq(freq);
        }
    }

    /// Processes the selected filters.
    pub fn process_filters(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        // let l =
        //     self.filter_peak[0].process(self.filter_hp[0].process(sample_l));
        // let r =
        //     self.filter_peak[1].process(self.filter_hp[1].process(sample_r));
        let l = self.filter_lp[0].process(sample_l);
        let r = self.filter_lp[1].process(sample_r);

        (l, r)
    }

    /// Processes the "post" peak filters.
    pub fn process_post_peak_filters(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        let l = self.filter_peak_post[0].process(sample_l);
        let r = self.filter_peak_post[1].process(sample_r);

        (l, r)
    }

    /// Processes the comb filters.
    pub fn process_comb_filters(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        let l = self.filter_comb[0].process(sample_l);
        let r = self.filter_comb[1].process(sample_r);

        (l, r)
    }

    /// Processes the waveshaper.
    pub fn process_distortion(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        let l = self.waveshaper[0].process(sample_l);
        let r = self.waveshaper[1].process(sample_r);

        (l, r)
    }

    /// Tries to receive messages from the corresponding `Senders`. Non-blocking.
    ///
    /// Will update internal values upon successfully receiving from a channel.
    pub fn try_receive(&mut self) {
        // filter frequency
        if let Some(freq) = &self.filter_freq_receiver {
            if let Ok(msg) = freq.try_recv() {
                // self.filter_freq.set(msg, self.glide_time);
            }
        }

        // waveshaper drive
        if let Some(drive) = &self.drive_amount_receiver {
            if let Ok(msg) = drive.try_recv() {
                for ws in &mut self.waveshaper {
                    ws.set_curve(msg);
                }
            }
        }

        // self.filter_freq.next();
    }

    /// Returns a thread-safe reference to the `AudioModel`'s callback timer.
    pub fn callback_timer_ref(&self) -> Arc<Mutex<std::time::Instant>> {
        Arc::clone(&self.callback_time_elapsed)
    }

    pub fn latency(&mut self) -> u32 {
        unimplemented!("latency calculation not fully implemented");
        self.latency_samples = 0;

        if let Some(oversampler) = self.oversamplers.first() {
            self.latency_samples += oversampler.latency(
                self.oversampling_factor
                    .load(std::sync::atomic::Ordering::Relaxed),
            );
        }

        self.latency_samples
    }

    pub fn set_idle_timer(&mut self, is_processing: bool) {
        self.idle_timer_samples = if is_processing {
            (unsafe { SAMPLE_RATE } * IDLE_TIME_SECS) as usize
        }
        else if self.idle_timer_samples > 0 {
            self.idle_timer_samples - 1
        }
        else {
            0
        }
    }

    pub fn is_idle(&self) -> bool {
        !self.is_processing && self.idle_timer_samples == 0
    }

    /// Returns the (approximate) sample index for the current moment in time.
    ///
    /// This is **not** a particularly precise method of tracking time events,
    /// but should be more than adequate for things like note events.
    pub fn current_sample_idx(&self) -> u32 {
        let guard = self.callback_time_elapsed.lock().unwrap();
        let samples_exact =
            guard.elapsed().as_secs_f64() * unsafe { SAMPLE_RATE };

        samples_exact.round() as u32 % BUFFER_SIZE as u32
    }
}
