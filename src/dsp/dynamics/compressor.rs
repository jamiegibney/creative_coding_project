use super::*;

const DEFAULT_ATTACK_TIME_MS: f64 = 100.0;
const DEFAULT_RELEASE_TIME_MS: f64 = 100.0;

#[derive(Clone, Debug)]
pub struct Compressor {
    attack_time_ms: f64,
    release_time_ms: f64,

    // hold time & lookahead?
    attack_samples: f64,
    release_samples: f64,

    sample_rate: f64,

    threshold_db: f64,

    knee_width: f64,
    ratio: f64,

    filter: OnePoleLowpass,

    rms_window_size_ms: f64,

    lookahead_ms: f64,

    envelope: f64,
    gain: f64,
}

impl Compressor {
    pub fn new(sample_rate: f64) -> Self {
        Compressor {
            attack_time_ms: DEFAULT_ATTACK_TIME_MS,
            attack_samples: 0.0,
            release_time_ms: DEFAULT_RELEASE_TIME_MS,
            release_samples: 0.0,

            sample_rate,

            threshold_db: 0.0,
            knee_width: 0.0,
            ratio: 1.0,

            rms_window_size_ms: 1.0,
            lookahead_ms: 0.0,

            filter: OnePoleLowpass::new(sample_rate),

            envelope: 0.0,
            gain: 0.0,
        }
    }

    /// Sets the compressor's threshold in decibels.
    ///
    /// # Panics
    ///
    /// Panics if `level_db` is greater than `0.0`.
    pub fn set_threshold_level(&mut self, level_db: f64) {
        assert!(level_db <= 0.0);

        self.threshold_db = level_db;
    }

    /// Sets the ratio of the compressor. Any values over `100.0` are clamped to `100.0`.
    ///
    /// # Panics
    ///
    /// Panics if `ratio` is less than `1.0`.
    pub fn set_ratio(&mut self, mut ratio: f64) {
        assert!(ratio >= 1.0);
        ratio = ratio.clamp(1.0, 100.0);

        self.ratio = ratio;
    }

    /// Sets the compressor's knee width.
    ///
    /// # Panics
    ///
    /// Panics if `width` is negative.
    pub fn set_knee_width(&mut self, width: f64) {
        assert!(width.is_sign_positive());

        self.knee_width = width;
    }

    /// Sets the compressor's attack time in milliseconds.
    ///
    /// [Conversion source](https://www.musicdsp.org/en/latest/Effects/169-compressor.html)
    pub fn set_attack_time_ms(&mut self, time_ms: f64) {
        // self.attack_time_ms = time_ms;
        // self.attack_samples =
        //     -(unsafe { OVERSAMPLED_SAMPLE_RATE } * (time_ms * 0.001)).recip().exp();

        self.attack_samples = unsafe { OVERSAMPLED_SAMPLE_RATE } * (time_ms * 0.001);
    }

    /// Sets the compressor's release time in milliseconds.
    ///
    /// [Conversion source](https://www.musicdsp.org/en/latest/Effects/169-compressor.html)
    pub fn set_release_time_ms(&mut self, time_ms: f64) {
        // self.release_time_ms = time_ms;
        // self.release_samples =
        //     -(unsafe { OVERSAMPLED_SAMPLE_RATE } * (time_ms * 0.001)).recip().exp();

        self.release_samples = unsafe { OVERSAMPLED_SAMPLE_RATE } * (time_ms * 0.001);
    }

    /// Sets the compressor's lookahead time in milliseconds.
    pub fn set_lookahead_time_ms(&mut self, time_ms: f64) {
        self.lookahead_ms = time_ms;
    }

    /// Sets the compressor's RMS averaging window size in milliseconds.
    pub fn set_rms_window_size_ms(&mut self, _time_ms: f64) {
        unimplemented!()
    }

    /// Standard compression gain function with a rounded knee and, otherwise,
    /// a linear profile. This represents the *amount of gain to apply* for a
    /// given envelope level, not a scale.
    ///
    /// This function may be used to find the compressor's transfer function,
    /// which may be useful if you wish to draw it, for example.
    ///
    /// From *Audio Processes by David Creasey*.
    pub fn gain_function(&self, input: f64) -> f64 {
        let Self {
            threshold_db: thresh,
            knee_width: width,
            ratio,
            ..
        } = self;
        let half_width = width / 2.0;

        // below the knee
        if input <= (thresh - half_width) {
            0.0
        }
        // within the knee
        else if (thresh - half_width) < input && input <= (thresh + half_width) {
            (2.0 * width).recip() * (ratio.recip() - 1.0) * (input - thresh + half_width).powi(2)
        }
        // above the knee
        else {
            (ratio.recip() - 1.0) * (input - thresh)
        }
    }
}

impl Effect for Compressor {
    fn process_stereo(&mut self, _in_l: f64, _in_r: f64) -> (f64, f64) {
        todo!()
    }

    fn get_sample_rate(&self) -> f64 {
        self.sample_rate
    }
}
