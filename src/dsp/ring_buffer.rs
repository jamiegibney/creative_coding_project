use super::Ramp;
use crate::prelude::*;
use crate::util::interp;
use crate::util::interp::InterpolationType as InterpType;

/// A ring buffer (AKA circular buffer) module, particularly useful for delays.
///
/// Implements delay time smoothing and sample interpolation.
#[derive(Debug, Clone)]
pub struct RingBuffer {
    size_samples: usize,
    write_pos: usize,
    buffer: Vec<f64>,
    interpolation_type: InterpType,
    delay_time_secs: Ramp,
    smoothing_type: SmoothingType,
    smoothing_time_secs: f64,
}

impl RingBuffer {
    /// Returns a new, initialised `RingBuffer` instance.
    #[must_use]
    pub fn new(size_samples: usize) -> Self {
        Self {
            size_samples,
            write_pos: 0,
            buffer: vec![0.0; size_samples],
            interpolation_type: InterpType::default(),
            delay_time_secs: Ramp::new(0.0, 0.0),
            smoothing_type: SmoothingType::Cosine,
            smoothing_time_secs: 0.03,
        }
    }

    pub fn dbg_print(&self) {
        let Self {
            size_samples,
            write_pos,
            interpolation_type,
            delay_time_secs,
            smoothing_type,
            smoothing_time_secs,
            ..
        } = self;

        dbg!(size_samples);
        dbg!(write_pos);
        dbg!(interpolation_type);
        dbg!(delay_time_secs);
        dbg!(smoothing_type);
        dbg!(smoothing_time_secs);
    }

    /// Prepares the buffer with `size_samples` samples. Also prepares the delay smoothing.
    ///
    /// # Panics
    pub fn prepare_with_samples(
        &mut self,
        size_samples: usize,
        smoothing_time_secs: f64,
    ) {
        self.buffer.clear();

        if size_samples != self.size_samples {
            self.size_samples = size_samples;
            self.buffer.resize(size_samples, 0.0);
            self.write_pos = 0;
        }

        self.delay_time_secs.reset(0.0, smoothing_time_secs);
        self.set_smoothing_type(self.smoothing_type);
    }

    /// Prepares the buffer with enough samples to hold `size_seconds` amount of time at
    /// the given sample rate. Also prepares the delay smoothing.
    ///
    /// # Panics
    ///
    /// Panics if `size_seconds` is negative.
    pub fn prepare_with_time(
        &mut self,
        size_seconds: f64,
        smoothing_time: f64,
    ) {
        assert!(size_seconds >= 0.0, "passed a negative size value");

        let num_samples =
            (unsafe { SAMPLE_RATE } * size_seconds).ceil() as usize;

        self.prepare_with_samples(num_samples, smoothing_time);
    }

    /// Sets the buffer's delay time in seconds.
    pub fn set_delay_time(&mut self, delay_seconds: f64) {
        self.delay_time_secs
            .reset(delay_seconds, self.smoothing_time_secs);
    }

    /// Sets the buffer's delay time in samples.
    pub fn set_delay_samples(&mut self, delay_samples: usize) {
        self.delay_time_secs.reset(
            delay_samples as f64 / unsafe { SAMPLE_RATE },
            self.smoothing_time_secs,
        );
    }

    /// Pushes a sample to the buffer, incrementing the write position.
    pub fn push(&mut self, sample: f64) {
        self.buffer[self.write_pos] = sample;
        self.increment_write_pos();
    }

    /// Pushes a vector of samples to the buffer.
    ///
    /// # Panics
    ///
    /// `panic!`s if the vector passed is too large to store in the buffer.
    pub fn push_vec(&mut self, samples: Vec<f64>) {
        assert!(self.size_samples >= samples.len(),
                "{}", format!("passed a vector with too many samples: capacity: {}, given: {}",
                              self.size_samples,
                              samples.len()));

        for sample in samples {
            self.buffer[self.write_pos] = sample;
            self.increment_write_pos();
        }
    }

    /// Returns the (interpolated) delayed value relative to the write position.
    pub fn read(&mut self) -> f64 {
        let (read_pos, interp) = self.read_pos_and_interp();

        let samples: [f64; 4] = [
            self.buffer[read_pos[0]], self.buffer[read_pos[1]],
            self.buffer[read_pos[2]], self.buffer[read_pos[3]],
        ];

        match self.interpolation_type {
            InterpType::NoInterp => samples[1],
            InterpType::Linear => interp::lerp(samples[1], samples[2], interp),
            InterpType::Cosine => {
                interp::cosine(samples[1], samples[2], interp)
            }
            InterpType::DefaultCubic => interp::cubic(
                samples[0], samples[1], samples[2], samples[3], interp,
            ),
            InterpType::CatmullCubic => interp::cubic_catmull(
                samples[0], samples[1], samples[2], samples[3], interp,
            ),
            InterpType::HermiteCubic => interp::cubic_hermite(
                samples[0], samples[1], samples[2], samples[3], interp, -0.5,
                0.2,
            ),
        }
    }

    /// Sets the interpolation method to use for delay values that lie between samples.
    pub fn set_interpolation_type(&mut self, interpolation_type: InterpType) {
        self.interpolation_type = interpolation_type;
    }

    /// Sets the smoothing function to use for delay time changes.
    pub fn set_smoothing_type(&mut self, smoothing_type: SmoothingType) {
        self.smoothing_type = smoothing_type;
        self.delay_time_secs.set_smoothing_type(self.smoothing_type);
    }

    /// Returns the size of the buffer in seconds.
    pub fn size_as_seconds(&self) -> f64 {
        self.size_samples as f64 / unsafe { SAMPLE_RATE }
    }

    /// # Internal method
    ///
    /// Increments the write pointer position, wrapping it around the buffer.
    fn increment_write_pos(&mut self) {
        self.write_pos = (self.write_pos + 1) % self.size_samples;
    }

    /// # Internal method
    ///
    /// Returns four read positions as an array, and the inter-sample remainder, as a tuple.
    fn read_pos_and_interp(&mut self) -> ([usize; 4], f64) {
        let delay_samples =
            self.delay_time_secs.next() * unsafe { SAMPLE_RATE };
        let floor_samples = delay_samples.floor();
        let remainder_time = delay_samples - floor_samples;

        (self.get_read_positions(floor_samples as usize), remainder_time)
    }

    /// # Internal method
    ///
    /// Returns four read positions: one before the read position, the read position itself,
    /// and two after it. Accounts for buffer wrapping.
    const fn get_read_positions(&self, delay_samples: usize) -> [usize; 4] {
        let r1 = if delay_samples >= self.write_pos {
            (delay_samples + self.write_pos) % self.size_samples
        }
        else {
            self.write_pos - delay_samples
        };

        let r0 = if r1 == 0 { self.size_samples - 1 } else { r1 - 1 };
        let r2 = (r1 + 1) % self.size_samples;
        let r3 = (r2 + 1) % self.size_samples;

        [r0, r1, r2, r3]
    }
}
