use super::*;

#[derive(Clone, Debug, Default)]
pub struct PingPongDelay {
    delay_l: Delay,
    delay_r: Delay,
    delay_offset: Delay,
}

impl PingPongDelay {
    pub fn new(max_delay_time_secs: f64, sample_rate: f64) -> Self {
        let delay = Delay::new(max_delay_time_secs, sample_rate);
        Self {
            delay_l: delay.clone(),
            delay_r: delay,
            delay_offset: Delay::new(max_delay_time_secs / 2.0, sample_rate),
        }
    }

    pub fn with_delay_time(mut self, delay_secs: f64) -> Self {
        self.delay_l.set_delay_time(delay_secs);
        self.delay_r.set_delay_time(delay_secs);
        self.delay_offset.set_delay_time(delay_secs / 2.0);
        self
    }

    pub fn with_delay_time_samples(mut self, delay_samples: f64) -> Self {
        self.delay_l.set_delay_time_samples(delay_samples);
        self.delay_r.set_delay_time_samples(delay_samples);
        self.delay_offset
            .set_delay_time_samples(delay_samples / 2.0);
        self
    }

    pub fn set_delay_time(&mut self, delay_secs: f64) {
        self.delay_l.set_delay_time(delay_secs);
        self.delay_r.set_delay_time(delay_secs);
        self.delay_offset.set_delay_time(delay_secs / 2.0);
    }

    pub fn set_delay_time_samples(&mut self, delay_samples: f64) {
        self.delay_l.set_delay_time_samples(delay_samples);
        self.delay_r.set_delay_time_samples(delay_samples);
        self.delay_offset
            .set_delay_time_samples(delay_samples / 2.0);
    }

    pub fn set_feedback_amount(&mut self, feedback: f64) {
        self.delay_l.set_feedback_amount(feedback);
        self.delay_r.set_feedback_amount(feedback);
    }

    pub fn max_delay_time_secs(&self) -> f64 {
        self.delay_l.max_delay_time_secs()
    }

    pub fn set_smoothing_time(&mut self, smoothing_time_secs: f64) {
        self.delay_l.set_smoothing_time(smoothing_time_secs);
        self.delay_r.set_smoothing_time(smoothing_time_secs);
        self.delay_offset.set_smoothing_time(smoothing_time_secs);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f64) {
        self.delay_l.set_sample_rate(sample_rate);
        self.delay_r.set_sample_rate(sample_rate);
        self.delay_offset.set_sample_rate(sample_rate);
    }
}

impl Effect for PingPongDelay {
    fn process_stereo(&mut self, in_l: f64, in_r: f64) -> (f64, f64) {
        let mut out_l = self.delay_offset.process_mono(in_l);
        out_l = self.delay_l.process_mono(out_l);
        let out_r = self.delay_r.process_mono(in_r);

        (out_l, out_r)
    }

    fn get_sample_rate(&self) -> f64 {
        self.delay_l.get_sample_rate()
    }
}
