//! The spectral processing and enveloping for a spectrogram.

use crate::dsp::StftHelper;
use crate::prelude::*;
use crate::util::window::{multiply_buffers, sine};
use realfft::{num_complex::Complex64, RealFftPlanner, RealToComplex};
use std::sync::Arc;
use triple_buffer::TripleBuffer;

pub const SPECTRUM_WINDOW_SIZE: usize = 1 << 11; // 2048
pub const SPECTRUM_OVERLAP_FACTOR: usize = 1 << 3; // 8
pub const DEFAULT_SPECTRUM_ATTACK_MS: f64 = 150.0;
pub const DEFAULT_SPECTRUM_RELEASE_MS: f64 = 180.0;

pub const RESULT_BUFFER_SIZE: usize = SPECTRUM_WINDOW_SIZE / 2 + 1;

/// The output of a spectrum analyzer.
pub type SpectrumOutput = triple_buffer::Output<Vec<f64>>;

/// The input to a spectrum analyzer.
pub struct SpectrumInput {
    /// An adapter to process most of the overlap-add operation.
    stft: StftHelper,

    /// Number of channels currently being processed.
    pub num_channels: usize,

    /// The attack time for all bin envelopes, which smooths the
    /// transition from old bins to new, higher bins.
    attack_weight: f64,
    /// The time taken to increase each bin by 12 dB in the attack phase.
    attack_time_ms: f64,

    /// The release time for all bin envelopes, which smooths the
    /// transition from old bins to new, lower bins.
    release_weight: f64,
    /// The time taken to decrease each bin by 12 dB in the release phase.
    release_time_ms: f64,

    /// This is a way to send information to a corresponding
    /// `SpectrumOutput`. The struct's `spectrum_result_buffer` is
    /// copied into this buffer each time a new spectrum is available.
    triple_buffer_input: triple_buffer::Input<Vec<f64>>,

    /// A scratch buffer used to compute the power amplitude spectrum.
    spectrum_result_buffer: Vec<f64>,

    /// The forward FFT algorithm used to produce the spectral data.
    plan: Arc<dyn RealToComplex<f64>>,

    /// This is a fixed Hann window with gain compensation applied ahead of time.
    window_function: Vec<f64>,

    /// The frequency output of the FFT.
    complex_buffer: Vec<Complex64>,
}

impl SpectrumInput {
    /// Returns a new spectrum input/output pair. The output should be moved
    /// to the editor.
    pub fn new(num_channels: usize) -> (Self, SpectrumOutput) {
        let (triple_buffer_input, output) =
            // TripleBuffer::new(&[0.0; SPECTRUM_WINDOW_SIZE / 2 + 1]).split();
            TripleBuffer::new(&vec![0.0; RESULT_BUFFER_SIZE]).split();

        let mut input = Self {
            stft: StftHelper::new(num_channels, SPECTRUM_WINDOW_SIZE, 0),
            num_channels,
            attack_weight: 0.0,
            attack_time_ms: DEFAULT_SPECTRUM_ATTACK_MS,
            release_weight: 0.0,
            release_time_ms: DEFAULT_SPECTRUM_RELEASE_MS,
            triple_buffer_input,
            spectrum_result_buffer: vec![0.0; RESULT_BUFFER_SIZE],
            plan: RealFftPlanner::new().plan_fft_forward(SPECTRUM_WINDOW_SIZE),

            // TODO does this need to be compensated?
            window_function: sine(SPECTRUM_WINDOW_SIZE),

            complex_buffer: vec![Complex64::default(); RESULT_BUFFER_SIZE],
        };

        input.update_timing();

        (input, output)
    }

    /// Updates the internal number of channels.
    ///
    /// This method may allocate.
    pub fn update_num_channels(&mut self, num_channels: usize) {
        self.num_channels = num_channels;
        self.stft = StftHelper::new(num_channels, SPECTRUM_WINDOW_SIZE, 0);
    }

    /// Updates the attack/release smoothing based on the given sample rate.
    ///
    /// Should be called if the sample rate changes.
    pub fn update_timing(&mut self) {
        let effective_sample_rate = unsafe { SAMPLE_RATE }
            / SPECTRUM_WINDOW_SIZE as f64
            * SPECTRUM_OVERLAP_FACTOR as f64
            * self.num_channels as f64;

        // 0.25 is used to represent a -12dB change in amplitude.
        let minus_12_db = 0.25f64;
        let attack_samples =
            self.attack_time_ms / 1000.0 * effective_sample_rate;
        self.attack_weight = minus_12_db.powf(attack_samples.recip());

        let release_samples =
            self.release_time_ms / 1000.0 * effective_sample_rate;
        self.release_weight = minus_12_db.powf(release_samples.recip());
    }

    /// Relatively scales the timing of the spectrogram based on
    /// [`DEFAULT_SPECTRUM_ATTACK_MS`] and [`DEFAULT_SPECTRUM_RELEASE_MS`].
    ///
    /// In other words, the above values are used if `factor == 1.0`.
    pub fn set_relative_timing(&mut self, factor: f64) {
        self.attack_time_ms = DEFAULT_SPECTRUM_ATTACK_MS * factor;
        self.release_time_ms = DEFAULT_SPECTRUM_RELEASE_MS * factor;
    }

    /// Computes the spectral information for an audio buffer and sends
    /// it to the output spectrum pair.
    #[allow(clippy::missing_panics_doc)] // this function should not panic
    pub fn compute(&mut self, buffer: &[f64]) {
        self.stft.process_forward_only(
            &buffer,
            SPECTRUM_OVERLAP_FACTOR,
            |_, real_buffer| {
                // apply the window function
                multiply_buffers(real_buffer, &self.window_function);

                // process the forward FFT
                self.plan
                    .process(real_buffer, &mut self.complex_buffer)
                    .unwrap();

                // apply the enveloping
                for (bin, spectrum) in self
                    .complex_buffer
                    .iter()
                    .skip(1)
                    .zip(&mut self.spectrum_result_buffer)
                {
                    let mag = bin.norm();

                    if mag > *spectrum {
                        *spectrum = (*spectrum).mul_add(
                            self.attack_weight,
                            mag * (1.0 - self.attack_weight),
                        );
                    }
                    else {
                        *spectrum = (*spectrum).mul_add(
                            self.release_weight,
                            mag * (1.0 - self.release_weight),
                        );
                    }
                }

                // send to the triple buffer output
                self.triple_buffer_input
                    .write(self.spectrum_result_buffer.clone());
            },
        );
    }
}
