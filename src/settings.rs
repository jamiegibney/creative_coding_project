/// The global sample rate, set to 44.1 kHz as default.
///
/// # Safety
///
/// You must be very careful about changing this; ensure that it is mutated
/// in a way that is thread-safe and somewhat predictable (i.e., don't change
/// this to be, for example, `-423947.4623`). Small adjustments are recommended,
/// if you need to change this at all.
pub static mut SAMPLE_RATE: f64 = 44100.0;

pub const BUFFER_SIZE: usize = 4096;

/// The global tuning frequency, set to 440 Hz as default.
///
/// # Safety
///
/// You must be very careful about changing this; ensure that it is mutated
/// in a way that is thread-safe and somewhat predictable (i.e., don't change
/// this to be, for example, `-423947.4623`). Small adjustments are recommended,
/// if you need to change this at all.
pub static mut TUNING_FREQ_HZ: f64 = 440.0;

