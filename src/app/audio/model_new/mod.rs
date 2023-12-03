use super::*;

pub mod components;
pub mod builder;
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
    // pub fn 
}
