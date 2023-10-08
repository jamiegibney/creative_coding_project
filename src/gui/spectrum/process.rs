use crate::dsp::fft::window;
use nih_plug::prelude::*;
use nih_plug::util::window::{blackman, hann};
use realfft::{
    num_complex::Complex32, ComplexToReal, RealFftPlanner, RealToComplex,
};
use std::sync::Arc;
use triple_buffer::TripleBuffer;

// These settings closely match FabFilter Pro-Q 3's spectrum analyser
// on its "Maximum" / "Very Fast" resolution / time settings.
pub const SPECTRUM_WINDOW_SIZE: usize = 1 << 13; // 8192
pub const SPECTRUM_OVERLAP_FACTOR: usize = 1 << 4; // 16
pub const SPECTRUM_ATTACK_MS: f32 = 120.0;
pub const SPECTRUM_RELEASE_MS: f32 = 110.0;

pub type Spectrum = [f32; SPECTRUM_WINDOW_SIZE / 2 + 1];

pub type SpectrumOutput = triple_buffer::Output<Spectrum>;

pub struct SpectrumInput {
    /// An adapter to process most of the overlap-add operation.
    stft: util::StftHelper,

    /// Number of channels currently being processed.
    pub num_channels: usize,

    /// The attack time for all bin envelopes, which smooths the
    /// transition from old bins to new, higher bins.
    attack_weight: f32,

    /// The release time for all bin envelopes, which smooths the
    /// transition from old bins to new, lower bins.
    release_weight: f32,

    /// This is a way to send information to a corresponding
    /// `SpectrumOutput`. The struct's `spectrum_result_buffer` is
    /// copied into this buffer each time a new spectrum is available.
    triple_buffer_input: triple_buffer::Input<Spectrum>,

    /// A scratch buffer used to compute the power amplitude spectrum.
    spectrum_result_buffer: Spectrum,

    /// The forward FFT algorithm used to produce the spectral data.
    plan: Arc<dyn RealToComplex<f32>>,
    plan_i: Arc<dyn ComplexToReal<f32>>,

    /// This is a fixed Hann window with gain compensation applied ahead
    /// of time.
    compensated_window_function: Vec<f32>,
    window_function: Vec<f32>,

    /// The frequency output of the FFT.
    complex_buffer: Vec<Complex32>,
    // TODO: this needs an input buffer for latency, right?
    //  unless the latency is applied separately elsewhere
}

impl SpectrumInput {
    /// Returns a new spectrum input/output pair. The output should be moved
    /// to the editor.
    pub fn new(num_channels: usize) -> (Self, SpectrumOutput) {
        let (triple_buffer_input, output) =
            TripleBuffer::new(&[0.0; SPECTRUM_WINDOW_SIZE / 2 + 1]).split();

        let input = Self {
            stft: util::StftHelper::new(num_channels, SPECTRUM_WINDOW_SIZE, 0),
            num_channels,
            attack_weight: 0.0,
            release_weight: 0.0,
            triple_buffer_input,
            spectrum_result_buffer: [0.0; SPECTRUM_WINDOW_SIZE / 2 + 1],
            plan: RealFftPlanner::new().plan_fft_forward(SPECTRUM_WINDOW_SIZE),
            plan_i: RealFftPlanner::new()
                .plan_fft_inverse(SPECTRUM_WINDOW_SIZE),

            compensated_window_function: blackman(SPECTRUM_WINDOW_SIZE)
                .into_iter()
                .map(|x| {
                    x / (SPECTRUM_WINDOW_SIZE * SPECTRUM_OVERLAP_FACTOR) as f32
                })
                .collect(),

            window_function: hann(SPECTRUM_WINDOW_SIZE),

            complex_buffer: vec![
                Complex32::default();
                SPECTRUM_WINDOW_SIZE / 2 + 1
            ],
        };

        (input, output)
    }

    pub fn update_num_channels(&mut self, num_channels: usize) {
        self.num_channels = num_channels;
        self.stft =
            util::StftHelper::new(num_channels, SPECTRUM_WINDOW_SIZE, 0);
    }

    /// Updates the attack/release smoothing based on the given sample rate.
    ///
    /// For use in `initialize()`
    pub fn update_sample_rate(&mut self, sample_rate: f32) {
        let effective_sample_rate = sample_rate / SPECTRUM_WINDOW_SIZE as f32
            * SPECTRUM_OVERLAP_FACTOR as f32
            * self.num_channels as f32;

        // 0.25 is used to represent -12dB change in amplitude.
        let minus_12_db = 0.25f64;
        let attack_samples =
            (SPECTRUM_ATTACK_MS / 1000.0 * effective_sample_rate) as f64;
        self.attack_weight = minus_12_db.powf(attack_samples.recip()) as f32;

        let release_samples =
            (SPECTRUM_RELEASE_MS / 1000.0 * effective_sample_rate) as f64;
        self.release_weight = minus_12_db.powf(release_samples.recip()) as f32;
    }

    /// Computes the spectral information for an audio buffer and sends
    /// it to the output spectrum pair.
    pub fn compute(&mut self, buffer: &mut Buffer) {
        // LATENCY GOES HERE IF INCLUDED IN THE PROCESSOR
        self.stft.process_analyze_only(
            buffer,
            SPECTRUM_OVERLAP_FACTOR,
            |_channel_idx, real_buffer| {
                // apply the window function
                window::multiply_buffers(real_buffer, &self.window_function);

                // process the forward FFT
                self.plan
                    .process(real_buffer, &mut self.complex_buffer)
                    .unwrap();

                // apply the enveloping
                for (bin, spectrum) in self
                    .complex_buffer
                    .iter_mut()
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
                self.triple_buffer_input.write(self.spectrum_result_buffer);
            },
        );
    }
}
