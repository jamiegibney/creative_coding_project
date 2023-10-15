// NOTE: there are some mutable static variables in this file, which are generally
// dangerous and not recommended for safety, particularly in a multi-threaded
// applications where data races are possible. The rationale for using these is that
// they are read far, far more often than they are overwritten (if they are overwritten 
// at all), and it is very convenient to have global access to this data, particularly
// for playing around and  experimentation. It also allows me to play around with race 
// conditions if I want, which seems like good practice...

use nannou::prelude::Vec2;

/// The global sample rate, set to 44.1 kHz as default.
///
/// # Safety
///
/// You must be very careful about changing this; ensure that it is mutated
/// in a way that is thread-safe and somewhat predictable; small adjustments
/// are recommended, if you need to change this at all.
pub static mut SAMPLE_RATE: f64 = 44100.0;

/// The global tuning frequency, set to 440 Hz as default.
///
/// # Safety
///
/// You must be very careful about changing this; ensure that it is mutated
/// in a way that is thread-safe and somewhat predictable (i.e., don't change
/// this to be, for example, `-423947.4623`). Small adjustments are recommended,
/// if you need to change this at all.
pub static mut TUNING_FREQ_HZ: f64 = 440.0;

/// The maximum number of simultaneous polyphonic voices.
pub const NUM_VOICES: u32 = 16;

/// The maximum size of an audio block. When processing audio, the buffer is
/// broken down into blocks which are this big, unless the buffer size is 
/// smaller.
pub const MAX_BLOCK_SIZE: usize = 1 << 6; // 64

pub struct V2 {
    pub x: f64,
    pub y: f64,
}

pub const WINDOW_SIZE: V2 = V2 {
    x: 1400.0,
    y: 800.0,
};

// TODO this is constant for now, but should be variable later.
pub const BUFFER_SIZE: usize = 256;

pub const NUM_CHANNELS: usize = 2;

pub const PRINT_DSP_LOAD: bool = true;
