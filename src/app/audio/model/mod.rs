use super::*;
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;

pub mod audio_constructor;
pub mod builder;
pub mod components;
pub use builder::*;
pub use components::*;

/// When the DSP stops , it will continue to process for this length of time to
/// allow the audio spectrums to fully relax. After this time has passed, the DSP
/// is skipped to reduce total load when idle.
const DSP_IDLE_HOLD_TIME_SECS: f64 = 0.8;

/// The program's audio state.
pub struct AudioModel {
    /// Fields related to audio generation (envelopes, oscillators, ...).
    pub generation: AudioGeneration,
    /// Signal processors — both musical FX and DSP-related.
    pub processors: AudioProcessors,

    /// Audio-related data (gain, oversampling state, ...).
    pub data: AudioData,
    pub buffers: AudioBuffers,

    /// The pre- and post-FX spectrograms on the audio thread.
    pub spectrograms: AudioSpectrograms,

    /// The audio thread's voice handler.
    pub voice_handler: VoiceHandler,
    /// Audio-related contextual data.
    pub context: AudioContext,

    /// Message receiving channels.
    pub message_channels: RefCell<AudioMessageReceivers>,

    /// The audio thread pool, intended for processing the spectrograms
    /// asynchronously.
    thread_pool: ThreadPool,
}

impl AudioModel {
    pub fn compute_pre_spectrum(&mut self, buffer: &Buffer<f64>) {
        self.spectrograms
            .pre_fx_spectrogram_buffer
            .try_lock()
            .map_or((), |mut guard| {
                for i in 0..buffer.len() {
                    guard[i] = buffer[i];
                }
            });

        let spectrum = Arc::clone(&self.spectrograms.pre_fx_spectrogram);
        let buffer = Arc::clone(&self.spectrograms.pre_fx_spectrogram_buffer);

        // noone:
        // rust: if let if let if let if let if let
        self.thread_pool.execute(move || {
            if let Ok(mut spectrum) = spectrum.try_lock() {
                if let Some(spectrum) = spectrum.as_mut() {
                    if let Ok(buf) = buffer.try_lock() {
                        spectrum.compute(&buf);
                    }
                }
            }
        });
    }

    pub fn compute_post_spectrum(&mut self, buffer: &Buffer<f64>) {
        self.spectrograms
            .post_fx_spectrogram_buffer
            .try_lock()
            .map_or((), |mut guard| {
                for i in 0..buffer.len() {
                    guard[i] = buffer[i];
                }
            });

        let spectrum = Arc::clone(&self.spectrograms.post_fx_spectrogram);
        let buffer = Arc::clone(&self.spectrograms.post_fx_spectrogram_buffer);

        self.thread_pool.execute(move || {
            if let Ok(mut spectrum) = spectrum.try_lock() {
                if let Some(spectrum) = spectrum.as_mut() {
                    if let Ok(buf) = buffer.try_lock() {
                        spectrum.compute(&buf);
                    }
                }
            }
        });
    }

    pub fn set_idle_timer(&mut self, is_processing: bool) {
        self.data.idle_timer_samples = if is_processing {
            (self.data.sample_rate.load(Relaxed) * DSP_IDLE_HOLD_TIME_SECS) as u64
        } else if self.data.idle_timer_samples > 0 {
            self.data.idle_timer_samples - 1
        } else {
            0
        };
    }

    pub fn is_idle(&self) -> bool {
        !self.data.is_processing && self.data.idle_timer_samples == 0
    }

    /// # Panics
    ///
    /// Panics if the callback timer cannot be locked.
    pub fn current_sample_idx(&self) -> u32 {
        let guard = self.data.callback_time_elapsed.lock().unwrap();

        let samples_exact = guard.elapsed().as_secs_f64() * self.data.sample_rate.load(Relaxed);

        drop(guard);

        samples_exact.round() as u32 % BUFFER_SIZE as u32
    }

    /// Returns the internal sample rate of the audio model.
    pub fn get_sample_rate(&self) -> f64 {
        self.data.sample_rate.load(Relaxed)
    }

    /// Returns the internal upsampled rate of the audio model.
    pub fn get_upsampled_rate(&self) -> f64 {
        self.data.upsampled_rate.load(Relaxed)
    }

    /// Returns the next available note event, if it exists.
    pub fn next_note_event(&self) -> Option<NoteEvent> {
        self.message_channels
            .borrow()
            .note_event
            .as_ref()
            .and_then(|ch| ch.try_recv().ok())
    }

    pub fn try_receive(&mut self) {
        let receivers = self.message_channels.borrow_mut();

        if let Some(pitch_trigger) = &receivers.resonator_bank_reset_pitch {
            if pitch_trigger.try_recv().is_ok() {
                self.processors.resonator_bank.randomise_resonator_pitches();
            }
        }

        if let Some(pan_trigger) = &receivers.resonator_bank_reset_pan {
            if pan_trigger.try_recv().is_ok() {
                // TODO: make this message hold the max pan amount instead of unit
                self.processors.resonator_bank.randomise_pan(0.3);
            }
        }

        if let Some(bank_params) = &receivers.resonator_bank_params {
            if let Ok(params) = bank_params.try_recv() {
                self.processors.resonator_bank.set_params(params);
            }
        }

        if let Some(mask_order) = &receivers.spectral_mask_post_fx {
            if mask_order.try_recv().is_ok() {
                self.processors.spectral_filter.clear();
                self.data.spectral_mask_post_fx = !self.data.spectral_mask_post_fx;
            }
        }
    }

    pub fn increment_sample_count(&mut self, buffer_size: u32) {
        let time = 6.0;
        let tmr = (time * self.data.sample_rate.lr()) as u32;

        self.data.sample_timer += buffer_size;
        if self.data.sample_timer > tmr {
            self.processors.resonator_bank.randomise_resonator_pitches();
            self.data.sample_timer -= tmr;
        }
    }

    pub fn reset_resonantor_bank(&mut self) {
        self.processors.resonator_bank.randomise_resonator_pitches();
    }
}
