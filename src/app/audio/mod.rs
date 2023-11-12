use super::*;
use crate::dsp::*;
use crate::gui::spectrum::*;
use nannou_audio::Buffer;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use thread_pool::ThreadPool;

// pub mod buffer;
pub mod context;
pub mod model;
pub mod process;
pub mod voice;

pub use context::AudioContext;
pub use model::AudioModel;
pub use process::process;
pub use voice::*;

pub const DSP_LOAD_AVERAGING_SAMPLES: usize = 32;

// const MAX_OVERSAMPLING_FACTOR: usize = 4; // 16x oversampling
// const DEFAULT_OVERSAMPLING_FACTOR: usize = 2; // 4x oversampling

/// A struct containing the channel senders returned by
/// `AudioModel::initialize()`.
///
/// The fields of this struct are used to communicate directly
/// with the audio thread.
pub struct AudioSenders {
    pub envelope_trigger: Sender<bool>,
    pub filter_freq: Sender<f64>,
    pub drive_amount: Sender<f64>,
}
