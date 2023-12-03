use super::{stft::stft_trait::StftInputMut, *};
use crate::util::window::*;
use nannou_audio::Buffer;
use realfft::{
    num_complex::Complex, ComplexToReal, RealFftPlanner, RealToComplex,
};
use std::sync::Arc;

pub mod mask;
use mask::*;

/// A spectral filtering processor, which accepts a `SpectralMask` as a frequency
/// mask and applies it to an audio signal in the frequency domain.
pub struct SpectralFilter {
    /// stft processor
    stft: StftHelper,

    /// a window function with gain compensation
    compensated_window_function: Vec<f64>,

    window_function: Vec<f64>,

    /// frequency domain buffers
    complex_buffers: Vec<Vec<Complex<f64>>>,

    /// forward fft plan
    fft: Arc<dyn RealToComplex<f64>>,

    /// inverse fft plan
    ifft: Arc<dyn ComplexToReal<f64>>,

    /// filter mask
    mask: SpectralMask,
}

impl SpectralFilter {
    const OVERLAP_FACTOR: usize = 4;

    /// # Panics
    ///
    /// Panics if `num_channels` or `max_block_size` is `0`.
    pub fn new(num_channels: usize, max_block_size: usize) -> Self {
        Self {
            stft: StftHelper::new(num_channels, max_block_size, 0),

            compensated_window_function: sine(max_block_size)
                .into_iter()
                .map(|x| {
                    x * ((max_block_size * Self::OVERLAP_FACTOR) as f64).recip()
                })
                .collect(),

            window_function: sine(max_block_size),

            complex_buffers: vec![
                vec![
                    Complex::default();
                    max_block_size / 2 + 1
                ];
                num_channels
            ],

            fft: RealFftPlanner::new().plan_fft_forward(max_block_size),
            ifft: RealFftPlanner::new().plan_fft_inverse(max_block_size),

            mask: SpectralMask::new(max_block_size)
                .with_size(max_block_size / 2),
        }
    }

    /// # Panics
    ///
    /// Panics if `block_size` is greater than the max block size of the processor.
    pub fn set_block_size(&mut self, block_size: usize) {
        assert!(block_size <= self.stft.max_block_size());

        let compensation_factor = self.compensation_factor(block_size);

        // window function
        self.compensated_window_function = sine(block_size)
            .into_iter()
            .map(|x| x * compensation_factor)
            .collect();

        self.window_function = sine(block_size);

        // stft
        self.stft.set_block_size(block_size);

        // complex buffer
        self.complex_buffers
            .iter_mut()
            .for_each(|buf| buf.resize(block_size / 2 + 1, Complex::default()));

        self.fft = RealFftPlanner::new().plan_fft_forward(block_size);
        self.ifft = RealFftPlanner::new().plan_fft_inverse(block_size);

        // mask
        self.mask.resize(block_size / 2, 1.0);
    }

    /// Clones `mask` into the filter.
    ///
    /// # Panics
    ///
    /// Panics if `mask.len()` is not equal to the block size of the processor.
    /// (See [`set_block_size()`](Self::set_block_size))
    pub fn set_mask(&mut self, mask: &SpectralMask) {
        self.mask.clone_from_slice(mask);
    }

    /// Processes a block of audio. This does not necessarily call the FFT algorithms.
    #[allow(clippy::missing_panics_doc)] // this function will not panic.
    pub fn process_block<B>(&mut self, buffer: &mut B)
    where
        B: StftInputMut,
    {
        self.stft.process_overlap_add(
            buffer,
            Self::OVERLAP_FACTOR,
            |ch_idx, audio_block| {
                // window the input
                multiply_buffers(audio_block, &self.window_function);

                // to freq domain
                self.fft
                    .process(audio_block, &mut self.complex_buffers[ch_idx])
                    .unwrap();

                // process magnitudes
                self.complex_buffers[ch_idx]
                    .iter_mut()
                    .zip(self.mask.iter())
                    .for_each(|(bin, &mask)| {
                        *bin *= mask;
                    });

                self.complex_buffers[ch_idx][0] *= 0.0;

                // back to time domain
                self.ifft
                    .process(&mut self.complex_buffers[ch_idx], audio_block)
                    .unwrap();

                // window the output
                multiply_buffers(
                    audio_block, &self.compensated_window_function,
                );
            },
        );
    }

    pub fn max_block_size(&self) -> usize {
        self.stft.max_block_size()
    }

    pub fn compensation_factor(&self, block_size: usize) -> f64 {
        ((block_size / 2) as f64 / (Self::OVERLAP_FACTOR as f64 * 4.0)).recip()
    }
}

impl Default for SpectralFilter {
    fn default() -> Self {
        const DEFAULT_BLOCK_SIZE: usize = 1 << 14;

        Self {
            stft: StftHelper::new(NUM_CHANNELS, DEFAULT_BLOCK_SIZE, 0),
            fft: RealFftPlanner::new().plan_fft_forward(DEFAULT_BLOCK_SIZE),
            ifft: RealFftPlanner::new().plan_fft_inverse(DEFAULT_BLOCK_SIZE),

            mask: SpectralMask::new(DEFAULT_BLOCK_SIZE)
                .with_size(DEFAULT_BLOCK_SIZE / 2),

            compensated_window_function: Vec::default(),
            window_function: Vec::default(),

            complex_buffers: Vec::default(),
        }
    }
}
