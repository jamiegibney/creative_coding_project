const LANCZOS3_UPSAMPLING_KERNEL: [f64; 11] = [
    0.024_317_08, -0.0, -0.135_094_91, 0.0, 0.607_927_1, 1.0, 0.607_927_1, 0.0,
    -0.135_094_91, -0.0, 0.024_317_08,
];

const LANCZOS3_DOWNSAMPLING_KERNEL: [f64; 11] = [
    0.012_158_54, -0.0, -0.067_547_46, 0.0, 0.303_963_55, 0.5, 0.303_963_55,
    0.0, -0.067_547_46, -0.0, 0.012_158_54,
];

const LANZCOS3_KERNEL_LATENCY: usize = LANCZOS3_UPSAMPLING_KERNEL.len() / 2;

/// A multi-channel oversampling stage using Lanczos resampling (a = 3).
#[derive(Clone)]
pub(super) struct Lanczos3Stage {
    oversampling_amount: usize,

    upsampling_buffer: Vec<Vec<f64>>,
    upsampling_write_pos: usize,

    additional_upsampling_latency: usize,

    downsampling_buffer: Vec<Vec<f64>>,
    downsampling_write_pos: usize,

    pub(super) scratch_buffer: Vec<Vec<f64>>,

    num_channels: usize,
}

impl Lanczos3Stage {
    /// # Panics
    ///
    /// Panics if `num_channels == 0`.
    pub fn new(
        num_channels: usize,
        max_block_size: usize,
        stage_number: u32,
    ) -> Self {
        assert_ne!(num_channels, 0);
        let oversampling_amount = 2usize.pow(stage_number + 1);

        let uncompensated_stage_latency = LANZCOS3_KERNEL_LATENCY * 2;

        let additional_upsampling_latency =
            (-(uncompensated_stage_latency as isize))
                .rem_euclid(oversampling_amount as isize) as usize;

        Self {
            oversampling_amount,

            upsampling_buffer: vec![
                vec![
                    0.0;
                    LANCZOS3_UPSAMPLING_KERNEL.len()
                        + additional_upsampling_latency
                ];
                num_channels
            ],
            upsampling_write_pos: 0,

            additional_upsampling_latency,

            downsampling_buffer: vec![
                vec![
                    0.0;
                    LANCZOS3_DOWNSAMPLING_KERNEL.len()
                ];
                num_channels
            ],
            downsampling_write_pos: 0,

            scratch_buffer: vec![
                vec![0.0; max_block_size * oversampling_amount];
                num_channels
            ],

            num_channels,
        }
    }

    /// Updates the number of channels used by the oversampling stage.
    ///
    /// # Panics
    ///
    /// Panics if `num_channels == 0`.
    pub fn update_num_channels(&mut self, num_channels: usize) {
        assert!(num_channels != 0);

        let upsampling_buf_size = LANCZOS3_UPSAMPLING_KERNEL.len()
            + self.additional_upsampling_latency;
        self.upsampling_buffer
            .resize(num_channels, vec![0.0; upsampling_buf_size]);

        self.downsampling_buffer.resize(
            num_channels,
            vec![0.0; LANCZOS3_DOWNSAMPLING_KERNEL.len()],
        );

        if let Some(buf) = self.scratch_buffer.first() {
            self.scratch_buffer
                .resize(num_channels, vec![0.0; buf.len()]);
        }
        else {
            panic!("no buffers were found in the oversampling stage's scratch buffer, which should be considered a bug!");
        }

        self.num_channels = num_channels;
    }

    pub fn num_channels(&self) -> usize {
        self.num_channels
    }

    pub fn reset(&mut self) {
        for buf in &mut self.upsampling_buffer {
            buf.fill(0.0);
        }
        for buf in &mut self.downsampling_buffer {
            buf.fill(0.0);
        }

        self.upsampling_write_pos = 0;
        self.downsampling_write_pos = 0;
    }

    pub fn effective_latency(&self) -> u32 {
        let uncompensated_stage_latency = LANZCOS3_KERNEL_LATENCY * 2;
        let total_stage_latency =
            uncompensated_stage_latency + self.additional_upsampling_latency;

        (total_stage_latency as f64 / self.oversampling_amount as f64) as u32
    }

    pub fn upsample_from(&mut self, block: &[f64]) {
        let num_samples = block.len() / self.num_channels;
        let output_length = num_samples * 2;
        // may as well check the scratch_buffer is not empty...
        assert_ne!(self.scratch_buffer.len(), 0);
        assert!(output_length <= self.scratch_buffer[0].len());

        // copy the block's data into every other element in the scratch buffer,
        // interleaving zeroes in the gaps.
        for ch in 0..self.num_channels {
            for i in 0..num_samples {
                let sample = block[i * self.num_channels + ch];
                let output_idx = i * 2;

                self.scratch_buffer[ch][output_idx] = sample;
                self.scratch_buffer[ch][output_idx + 1] = 0.0;
            }
        }

        // cache the write pos so that it is the same for each channel
        let start_pos = self.upsampling_write_pos;
        let up_buffer_len = self.upsampling_buffer[0].len();

        for ch in 0..self.num_channels {
            // reset the write pos for each channel
            self.upsampling_write_pos = start_pos;

            let mut direct_read_pos = (self.upsampling_write_pos
                + LANZCOS3_KERNEL_LATENCY)
                % up_buffer_len;

            for out_idx in 0..output_length {
                // copy from scratch buffer
                self.upsampling_buffer[ch][self.upsampling_write_pos] =
                    self.scratch_buffer[ch][out_idx];

                // increment write positions
                self.increment_up_write_positions(&mut direct_read_pos);

                self.scratch_buffer[ch][out_idx] = if out_idx % 2
                    == (LANZCOS3_KERNEL_LATENCY % 2)
                {
                    debug_assert!(
                        self.upsampling_buffer[ch][(direct_read_pos
                            + up_buffer_len
                            - 1)
                            % up_buffer_len]
                            <= f64::EPSILON
                    );
                    debug_assert!(
                        self.upsampling_buffer[ch]
                            [(direct_read_pos + 1) % up_buffer_len]
                            <= f64::EPSILON
                    );

                    self.upsampling_buffer[ch][direct_read_pos]
                }
                else {
                    convolve_ring_buffer(
                        &self.upsampling_buffer[ch],
                        &LANCZOS3_UPSAMPLING_KERNEL, self.upsampling_write_pos,
                    )
                }
            }
        }
    }

    pub fn downsample_to(&mut self, block: &mut [f64]) {
        // block is interleaved multi-channel data, so div by the number of channels
        // to get just one channel's worth. * 2 is for one oversampling stage.
        let input_length = block.len() / self.num_channels * 2;
        // may as well check the scratch_buffer is not empty...
        assert_ne!(self.scratch_buffer.len(), 0);
        assert!(input_length <= self.scratch_buffer[0].len());

        // cache the write pos so that it is the same for each channel
        let start_pos = self.downsampling_write_pos;

        for ch in 0..self.num_channels {
            // reset the write pos for each channel
            self.downsampling_write_pos = start_pos;

            for input_idx in 0..input_length {
                // copy from scratch buffer
                self.downsampling_buffer[ch][self.downsampling_write_pos] =
                    self.scratch_buffer[ch][input_idx];

                // increment write pos
                self.increment_down_write_pos();

                if input_idx % 2 == 0 {
                    let output_idx = input_idx / 2;

                    block[output_idx * self.num_channels + ch] =
                        convolve_ring_buffer(
                            &self.downsampling_buffer[ch],
                            &LANCZOS3_DOWNSAMPLING_KERNEL,
                            self.downsampling_write_pos,
                        );
                }
            }
        }
    }

    fn increment_up_write_positions(&mut self, direct_pos: &mut usize) {
        self.upsampling_write_pos += 1;
        if self.upsampling_write_pos == self.upsampling_buffer.len() {
            self.upsampling_write_pos = 0;
        }

        *direct_pos += 1;
        if *direct_pos == self.upsampling_buffer.len() {
            *direct_pos = 0;
        }
    }

    fn increment_down_write_pos(&mut self) {
        self.downsampling_write_pos += 1;
        if self.downsampling_write_pos == self.downsampling_buffer.len() {
            self.downsampling_write_pos = 0;
        }
    }
}

/// Convolves a **single** channel's ring buffer with the Lanczos kernel.
fn convolve_ring_buffer(
    input_buffer: &[f64],
    kernel: &[f64],
    ring_buffer_pos: usize,
) -> f64 {
    debug_assert!(input_buffer.len() >= kernel.len());

    let mut total = 0.0;

    let num_samples_until_wraparound =
        (input_buffer.len() - ring_buffer_pos).min(kernel.len());

    for (read_pos_offset, &kernel_sample) in kernel
        .iter()
        .rev()
        .take(num_samples_until_wraparound)
        .enumerate()
    {
        total +=
            kernel_sample * input_buffer[ring_buffer_pos + read_pos_offset];
    }

    for (read_pos, kernel_sample) in kernel
        .iter()
        .rev()
        .skip(num_samples_until_wraparound)
        .enumerate()
    {
        total += kernel_sample * input_buffer[read_pos];
    }

    total
}
