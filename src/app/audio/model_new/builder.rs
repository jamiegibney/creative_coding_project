use super::*;

// TODO add the audio message channels (need to figure out what is needed)
pub struct AudioModelBuilder {
    /// The audio model.
    model: AudioModel2,
    /// A byte which tracks which fields of the `AudioModel` have been set.
    prepared_state: u8,
}

pub struct AudioPackage {
    pub model: AudioModel2,
    pub spectrum_outputs: (SpectrumOutput, SpectrumOutput),
    pub callback_timer_ref: Arc<Mutex<std::time::Instant>>,
    pub sample_rate_ref: Arc<AtomicF64>,
    // pub message_channels: AudioSenders,
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
    /// - [`data()`](Self::data)
    /// - [`spectrograms()`](Self::spectrograms)
    ///
    /// # Panics
    ///
    /// Panics if the `voice_event_receiver` field of `context` is `None`,
    /// or if the internal thread pool fails to spawn threads.
    pub fn new(mut context: AudioContext) -> Self {
        Self {
            model: AudioModel2 {
                generation: AudioGeneration::default(),
                processors: AudioProcessors::default(),
                data: AudioData::default(),
                buffers: AudioBuffers::default(),
                spectrograms: AudioSpectrograms::default(),
                voice_handler: VoiceHandler::build(
                    context.note_handler_ref(),
                    context.voice_event_receiver.take().unwrap(),
                ),
                context,
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

    /// Moves `spectrograms` into the `AudioModel`.
    pub fn spectrograms(mut self, spectrograms: AudioSpectrograms) -> Self {
        self.model.spectrograms = spectrograms;
        self.prepared_state |= 0b0001_0000;
        self
    }

    /// Builds the audio model.
    ///
    /// # Panics
    ///
    /// Panics if you haven't called all of the following methods:
    ///
    /// - [`processors()`](Self::processors)
    /// - [`generation()`](Self::generation)
    /// - [`data()`](Self::data)
    /// - [`data()`](Self::data)
    /// - [`spectrograms()`](Self::spectrograms)
    pub fn build(mut self) -> AudioPackage {
        assert!(self.prepared_state == Self::PREPARED_CHECKSUM);

        AudioPackage {
            spectrum_outputs: self.spectrum_outputs(),
            callback_timer_ref: Arc::clone(
                &self.model.data.callback_time_elapsed,
            ),
            sample_rate_ref: Arc::clone(&self.model.data.sample_rate),
            // message_channels: todo!(),
            model: self.model,
        }
    }

    fn spectrum_outputs(&mut self) -> (SpectrumOutput, SpectrumOutput) {
        let (mut pre_in, pre_out) = SpectrumInput::new(2);
        let (mut post_in, post_out) = SpectrumInput::new(2);

        let empty = vec![0.0; BUFFER_SIZE * NUM_CHANNELS];
        pre_in.compute(&empty);
        post_in.compute(&empty);

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

    // fn message_channels(&mut self) -> AudioSenders {
    //     let (envelope_trigger_sender, _receiver) = channel();
    //
    //     let (filter_freq_sender, receiver) = channel();
    //     self.model.filter_freq_receiver = Some(receiver);
    //
    //     let (drive_amount_sender, receiver) = channel();
    //     self.drive_amount_receiver = Some(receiver);
    //
    //     AudioSenders {
    //         envelope_trigger: envelope_trigger_sender,
    //         filter_freq: filter_freq_sender,
    //         drive_amount: drive_amount_sender,
    //     }
    // }
}
