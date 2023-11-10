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

/// The global oversampling rate, set to `SAMPLE_RATE` by default.
///
/// # Safety
///
/// Please only use the [`update_oversampling_rate()`](update_oversampling_rate)
/// function to update this value, as it lowers the chance of setting an unsafe value.
pub static mut OVERSAMPLED_SAMPLE_RATE: f64 = unsafe { SAMPLE_RATE };

/// A function to update the global oversampled sample rate (`OVERSAMPLED_SAMPLE_RATE`).
///
/// **IMPORTANT**: `oversampling_factor` is the *factor*, **not** the *exponent*. In other
/// words, a value of `4` corresponds to 4x oversampling, not 2⁴ (16)x oversampling.
///
/// # Panics
///
/// Panics if `oversampling_factor` is greater than `2 ^ MAX_OVERSAMPLING_FACTOR`.
///
/// # Safety
///
/// This function is marked unsafe, not because it itself is unsafe to use, but because
/// it concerns updating the global oversampled sample rate used by many signal processors
/// (potentially across different threads), and requiring an unsafe block makes it clearer
/// that it must be implemented correctly. It is intended as a constrained shorthand for
/// updating `OVERSAMPLED_SAMPLE_RATE` directly.
pub unsafe fn update_oversampled_sample_rate(oversampling_factor: usize) {
    assert!(oversampling_factor <= 2usize.pow(MAX_OVERSAMPLING_FACTOR as u32));
    unsafe {
        OVERSAMPLED_SAMPLE_RATE =
            SAMPLE_RATE * 2.0f64.powi(oversampling_factor as i32);
    }
}

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

/// A convenience struct to allow `WINDOW_SIZE` to have `x` and `y` fields.
pub struct V2 {
    pub x: f64,
    pub y: f64,
}

/// The size of the application's window in display units.
pub const WINDOW_SIZE: V2 = V2 { x: 1400.0, y: 800.0 };

// TODO this is constant for now, but should be variable later.
/// The default DSP buffer size.
pub const BUFFER_SIZE: usize = 256;

/// The maximum available DSP buffer size.
pub const MAX_BUFFER_SIZE: usize = 2048;

/// The number of audio channels for the application.
pub const NUM_CHANNELS: usize = 2;

/// An option to allow the DSP load to be printed to the standard output. Incurs a
/// slight run-time cost if `true`.
pub const PRINT_DSP_LOAD: bool = true;

/// The maximum available oversampling factor (i.e. this is `2⁴ == 16x` oversampling).
pub const MAX_OVERSAMPLING_FACTOR: usize = 4; // 16x oversampling
/// The default oversampling factor (i.e. this is `2² == 4x` oversampling).
pub const DEFAULT_OVERSAMPLING_FACTOR: usize = 3; // 4x oversampling
