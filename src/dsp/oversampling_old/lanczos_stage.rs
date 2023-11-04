const LANCZOS3_UPSAMPLING_KERNEL: [f64; 11] = [
    0.024_317_08, -0.0, -0.135_094_91, 0.0, 0.607_927_1, 1.0, 0.607_927_1, 0.0,
    -0.135_094_91, -0.0, 0.024_317_08,
];

const LANCZOS3_DOWNSAMPLING_KERNEL: [f64; 11] = [
    0.012_158_54, -0.0, -0.067_547_46, 0.0, 0.303_963_55, 0.5, 0.303_963_55,
    0.0, -0.067_547_46, -0.0, 0.012_158_54,
];

const LANZCOS3_KERNEL_LATENCY: usize = LANCZOS3_UPSAMPLING_KERNEL.len() / 2;

#[derive(Clone)]
pub(super) struct Lanczos3Stage {
    oversampling_amount: usize,

    upsampling_buffer: Vec<f64>,
    upsampling_write_pos: usize,

    additional_upsampling_latency: usize,

    downsampling_buffer: Vec<f64>,
    downsampling_write_pos: usize,

    pub(super) scratch_buffer: Vec<f64>,
}

impl Lanczos3Stage {
    /// Creates a new oversampling stage.
    pub fn new(max_block_size: usize, stage_number: u32) -> Self {
        let oversampling_amount = 2usize.pow(stage_number + 1);

        let uncompensated_stage_latency = LANZCOS3_KERNEL_LATENCY * 2;

        let additional_upsampling_latency =
            (-(uncompensated_stage_latency as isize))
                .rem_euclid(oversampling_amount as isize) as usize;

        Self {
            oversampling_amount,

            upsampling_buffer: vec![
                0.0;
                LANCZOS3_UPSAMPLING_KERNEL.len()
                    + additional_upsampling_latency
            ],
            upsampling_write_pos: 0,

            additional_upsampling_latency,

            downsampling_buffer: vec![0.0; LANCZOS3_DOWNSAMPLING_KERNEL.len()],
            downsampling_write_pos: 0,

            scratch_buffer: vec![0.0; max_block_size * oversampling_amount],
        }
    }

    pub fn reset(&mut self) {
        self.upsampling_buffer.fill(0.0);
        self.upsampling_write_pos = 0;

        self.downsampling_buffer.fill(0.0);
        self.downsampling_write_pos = 0;
    }

    pub fn effective_latency(&self) -> u32 {
        let uncompensated_stage_latency = LANZCOS3_KERNEL_LATENCY * 2;
        let total_stage_latency =
            uncompensated_stage_latency + self.additional_upsampling_latency;

        (total_stage_latency as f64 / self.oversampling_amount as f64) as u32
    }

    /// Upsamples a single block of audio. That is, **one** channel.
    pub fn upsample_from(&mut self, block: &[f64]) {
        let output_length = block.len() * 2;
        assert!(output_length <= self.scratch_buffer.len());

        for (i, &smp) in block.iter().enumerate() {
            let output_smp_idx = i * 2;
            self.scratch_buffer[output_smp_idx] = smp;
            self.scratch_buffer[output_smp_idx + 1] = 0.0;
        }

        let mut direct_read_pos = (self.upsampling_write_pos
            + LANZCOS3_KERNEL_LATENCY)
            % self.upsampling_buffer.len();

        for out_idx in 0..output_length {
            self.upsampling_buffer[self.upsampling_write_pos] =
                self.scratch_buffer[out_idx];

            self.increment_up_write_positions(&mut direct_read_pos);

            self.scratch_buffer[out_idx] =
                if out_idx % 2 == (LANZCOS3_KERNEL_LATENCY % 2) {
                    debug_assert!(
                        self.upsampling_buffer[(direct_read_pos
                            + self.upsampling_buffer.len()
                            - 1)
                            % self.upsampling_buffer.len()]
                            <= f64::EPSILON
                    );
                    debug_assert!(
                        self.upsampling_buffer[(direct_read_pos + 1)
                            % self.upsampling_buffer.len()]
                            <= f64::EPSILON
                    );

                    self.upsampling_buffer[direct_read_pos]
                }
                else {
                    convolve_ring_buffer(
                        &self.upsampling_buffer, &LANCZOS3_UPSAMPLING_KERNEL,
                        self.upsampling_write_pos,
                    )
                }
        }
    }

    pub fn downsample_to(&mut self, block: &mut [f64]) {
        let input_length = block.len() * 2;
        assert!(input_length <= self.scratch_buffer.len());

        for input_idx in 0..input_length {
            self.downsampling_buffer[self.downsampling_write_pos] =
                self.scratch_buffer[input_idx];

            self.increment_down_write_pos();

            if input_idx % 2 == 0 {
                let output_idx = input_idx / 2;

                block[output_idx] = convolve_ring_buffer(
                    &self.downsampling_buffer, &LANCZOS3_DOWNSAMPLING_KERNEL,
                    self.downsampling_write_pos,
                );
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
