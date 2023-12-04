use super::*;
use std::sync::atomic::Ordering::Relaxed;

pub mod builder;
pub mod components;
pub use builder::*;
pub use components::*;

/// When the DSP stops , it will continue to process for this length of time to
/// allow the audio spectrums to fully relax. After this time has passed, the DSP
/// is skipped to reduce total load when idle.
const DSP_IDLE_HOLD_TIME_SECS: f64 = 0.8;

/// The program's audio state.
pub struct AudioModel2 {
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

    /// The audio thread pool, intended for processing the spectrograms
    /// asynchronously.
    thread_pool: ThreadPool,
}

impl AudioModel2 {
    pub fn compute_pre_spectrum(&mut self) {
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

    pub fn compute_post_spectrum(&mut self) {
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
            (self.data.sample_rate.load(Relaxed) * DSP_IDLE_HOLD_TIME_SECS)
                as u64
        }
        else if self.data.idle_timer_samples > 0 {
            self.data.idle_timer_samples - 1
        }
        else {
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

        let samples_exact =
            guard.elapsed().as_secs_f64() * self.data.sample_rate.load(Relaxed);

        drop(guard);

        samples_exact.round() as u32 % BUFFER_SIZE as u32
    }
}
