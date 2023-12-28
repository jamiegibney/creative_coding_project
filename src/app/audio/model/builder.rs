use super::*;
use crossbeam_channel::{bounded, unbounded};
use std::cell::RefCell;

pub struct AudioModelBuilder {
    /// The audio model.
    model: AudioModel,
    /// A byte which tracks which fields of the `AudioModel` have been set.
    prepared_state: u8,
}

pub struct AudioPackage {
    pub model: AudioModel,
    pub spectrum_outputs: (SpectrumOutput, SpectrumOutput),
    pub callback_timer_ref: Arc<Mutex<std::time::Instant>>,
    pub sample_rate_ref: Arc<AtomicF64>,
    pub message_channels: AudioMessageSenders,
}

impl AudioModelBuilder {
    /// The bits required for the `AudioModel` to be "prepared".
    const PREPARED_CHECKSUM: u8 = 0b0001_1111;

    /// Initialises a new, default audio model.
    ///
    /// You must call the following methods before using the model:
    ///
    /// - [`processors()`](Self::processors)
    /// - [`generation()`](Self::generation)
    /// - [`data()`](Self::data)
    /// - [`buffers()`](Self::buffers)
    /// - [`params()`](Self::params)
    ///
    /// # Panics
    ///
    /// Panics if the `voice_event_receiver` field of `context` is `None`,
    /// or if the internal thread pool fails to spawn threads.
    pub fn new(mut context: AudioContext) -> Self {
        Self {
            model: AudioModel {
                generation: AudioGeneration::default(),
                processors: AudioProcessors::default(),
                data: AudioData::default(),
                buffers: AudioBuffers::default(),
                spectrograms: AudioSpectrograms::default(),
                voice_handler: VoiceHandler::build(
                    // context.note_handler_ref(),
                    context.voice_event_receiver.take().unwrap(),
                ),
                context,
                message_channels: RefCell::new(AudioMessageReceivers::default()),
                params: AudioParams::default(),
                thread_pool: ThreadPool::build(4).unwrap(),
            },
            prepared_state: 0b0000_0000,
        }
    }

    /// Moves `processors` into the `AudioModel`.
    pub fn processors(mut self, processors: AudioProcessors) -> Self {
        self.model.processors = processors;
        self.prepared_state |= 0b0000_0001;
        self
    }

    /// Moves `generation` into the `AudioModel`.
    pub fn generation(mut self, generation: AudioGeneration) -> Self {
        self.model.generation = generation;
        self.prepared_state |= 0b0000_0010;
        self
    }

    /// Moves `data` into the `AudioModel`.
    pub fn data(mut self, data: AudioData) -> Self {
        self.model.data = data;
        self.prepared_state |= 0b0000_0100;
        self
    }

    /// Moves `buffers` into the `AudioModel`.
    pub fn buffers(mut self, buffers: AudioBuffers) -> Self {
        self.model.buffers = buffers;
        self.prepared_state |= 0b0000_1000;
        self
    }

    /// Connects the appropriate values with the UI.
    pub fn params(mut self, ui_params: &UIParams) -> Self {
        self.prepared_state |= 0b0001_0000;
        self.attach_ui_params(ui_params);
        self
    }

    /// Builds the audio model.
    ///
    /// # Panics
    ///
    /// Panics if you haven't called **all** of the following methods:
    ///
    /// - [`processors()`](Self::processors)
    /// - [`generation()`](Self::generation)
    /// - [`data()`](Self::data)
    /// - [`buffers()`](Self::buffers)
    /// - [`params()`](Self::params)
    pub fn build(mut self) -> AudioPackage {
        assert!(
            self.prepared_state == Self::PREPARED_CHECKSUM,
            "AudioModelBuilder::build(): failed to verify preparation checksum, please call all the required methods"
        );

        AudioPackage {
            spectrum_outputs: self.spectrum_outputs(),
            callback_timer_ref: Arc::clone(
                &self.model.data.callback_time_elapsed,
            ),
            sample_rate_ref: Arc::clone(&self.model.data.sample_rate),
            message_channels: self.message_channels(),
            model: self.model,
        }
    }

    fn spectrum_outputs(&mut self) -> (SpectrumOutput, SpectrumOutput) {
        let (mut pre_in, pre_out) = SpectrumInput::new(2);
        let (mut post_in, post_out) = SpectrumInput::new(2);

        let empty = vec![0.0; BUFFER_SIZE * NUM_CHANNELS];
        pre_in.compute(&empty);
        post_in.compute(&empty);

        let buffer = vec![0.0; MAX_BUFFER_SIZE * NUM_CHANNELS];
        self.model.spectrograms.pre_fx_spectrogram_buffer =
            Arc::new(Mutex::new(buffer.clone()));

        self.model.spectrograms.post_fx_spectrogram_buffer =
            Arc::new(Mutex::new(buffer));

        let mut guard = loop {
            let res = self.model.spectrograms.pre_fx_spectrogram.try_lock();
            if let Ok(x) = res {
                break x;
            }
        };
        *guard = Some(pre_in);
        drop(guard);

        let mut guard = loop {
            let res = self.model.spectrograms.post_fx_spectrogram.try_lock();
            if let Ok(x) = res {
                break x;
            }
        };
        *guard = Some(post_in);
        drop(guard);

        (pre_out, post_out)
    }

    fn message_channels(&mut self) -> AudioMessageSenders {
        let mut msg_ch = self.model.message_channels.borrow_mut();
        let (drive_amount, receiver) = unbounded();
        msg_ch.drive_amount = Some(receiver);

        let (filter_freq, receiver) = unbounded();
        msg_ch.filter_freq = Some(receiver);

        let (note_event, receiver) = bounded(MAX_NOTE_EVENTS_PER_BUFFER);
        msg_ch.note_event = Some(receiver);

        let (resonator_bank_params, receiver) = unbounded();
        msg_ch.resonator_bank_params = Some(receiver);

        let (resonator_bank_reset_pitch, receiver) = unbounded();
        msg_ch.resonator_bank_reset_pitch = Some(receiver);

        let (resonator_bank_reset_pan, receiver) = unbounded();
        msg_ch.resonator_bank_reset_pan = Some(receiver);

        let (spectral_mask_post_fx, receiver) = unbounded();
        msg_ch.spectral_mask_post_fx = Some(receiver);

        AudioMessageSenders {
            note_event,
            filter_freq,
            drive_amount,
            resonator_bank_params,
            resonator_bank_reset_pitch,
            resonator_bank_reset_pan,
            spectral_mask_post_fx,
        }
    }

    pub fn attach_ui_params(&mut self, ui_params: &UIParams) {
        let UIParams {
            mask_resolution,
            mask_is_post_fx,
            // reso_bank_scale,
            // reso_bank_root_note,
            // reso_bank_spread,
            // reso_bank_shift,
            // reso_bank_inharm,
            // reso_bank_pan,
            // reso_bank_quantise,
            // reso_bank_resonator_count,
            // reso_bank_cell_count,
            // reso_bank_cell_jitter,
            // reso_bank_cell_scatter,
            low_filter_cutoff,
            low_filter_q,
            low_filter_gain_db,
            low_filter_is_shelf,

            high_filter_cutoff,
            high_filter_q,
            high_filter_gain_db,
            high_filter_is_shelf,

            delay_time_ms,
            delay_feedback,
            delay_mix,
            use_ping_pong,

            dist_amount,
            dist_type,

            comp_thresh,
            comp_ratio,
            comp_attack_ms,
            comp_release_ms,

            master_gain,
            ..
        } = ui_params;

        self.model.params.mask_resolution = Arc::clone(mask_resolution);
        self.model.params.mask_is_post_fx = Arc::clone(mask_is_post_fx);

        self.model.params.low_filter_cutoff = Arc::clone(low_filter_cutoff);
        self.model.params.low_filter_q = Arc::clone(low_filter_q);
        self.model.params.low_filter_gain_db = Arc::clone(low_filter_gain_db);
        self.model.params.low_filter_is_shelf = Arc::clone(low_filter_is_shelf);

        self.model.params.high_filter_cutoff = Arc::clone(high_filter_cutoff);
        self.model.params.high_filter_q = Arc::clone(high_filter_q);
        self.model.params.high_filter_gain_db = Arc::clone(high_filter_gain_db);
        self.model.params.high_filter_is_shelf =
            Arc::clone(high_filter_is_shelf);

        self.model.params.delay_time_ms = Arc::clone(delay_time_ms);
        self.model.params.delay_feedback = Arc::clone(delay_feedback);
        self.model.params.delay_mix = Arc::clone(delay_mix);
        self.model.params.use_ping_pong = Arc::clone(use_ping_pong);

        self.model.params.dist_amount = Arc::clone(dist_amount);
        self.model.params.dist_type = Arc::clone(dist_type);

        self.model.params.comp_thresh = Arc::clone(comp_thresh);
        self.model.params.comp_ratio = Arc::clone(comp_ratio);
        self.model.params.comp_attack_ms = Arc::clone(comp_attack_ms);
        self.model.params.comp_release_ms = Arc::clone(comp_release_ms);

        self.model.params.master_gain = Arc::clone(master_gain);
    }
}
