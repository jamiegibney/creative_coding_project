use super::*;
use crate::dsp::*;
use crate::dsp::{adsr::AdsrEnvelope, ramp::Ramp};
use std::sync::mpsc::{channel, Receiver, Sender};

/// A struct containing the channel senders returned by
/// `AudioModel::initialize()`.
///
/// The fields of this struct are used to communicate directly
/// with the audio thread.
pub struct AudioSenders {
    pub envelope_trigger: Sender<bool>,
    pub filter_freq: Sender<f64>,
    pub drive_amount: Sender<f64>,
}

/// The audio state for the whole program.
pub struct AudioModel {
    pub filter_lp: [BiquadFilter; 2],
    pub filter_hp: [BiquadFilter; 2],
    pub filter_peak: [BiquadFilter; 2],
    pub filter_peak_post: [BiquadFilter; 2],
    pub filter_comb: [IirCombFilter; 2],

    pub filter_freq: Ramp,
    filter_freq_receiver: Option<Receiver<f64>>,

    pub waveshaper: [Waveshaper; 2],
    drive_amount_receiver: Option<Receiver<f64>>,

    pub envelope: AdsrEnvelope,
    envelope_trigger: bool,
    envelope_trigger_receiver: Option<Receiver<bool>>,

    pub glide_time: f64,
    pub volume: f64,
}

impl AudioModel {
    pub fn new() -> Self {
        let sample_rate = unsafe { SAMPLE_RATE };
        let biquad = BiquadFilter::new(sample_rate);

        let mut comb = IirCombFilter::with_interpolation(true);

        let mut comb_peak = BiquadFilter::new(sample_rate);
        comb_peak.set_params(&BiquadParams {
            freq: 326.0,
            gain: 4.0,
            q: 0.3,
            filter_type: FilterType::Peak,
        });

        // let mut comb_lp = BiquadFilter::new(sample_rate);
        // comb_lp.set_params(&BiquadParams {
        //     freq: 2652.0,
        //     gain: 0.0,
        //     q: 2.0,
        //     filter_type: FilterType::Lowpass,
        // });

        let mut comb_lp = FirstOrderFilter::new(sample_rate);
        comb_lp.set_type(FilterType::Lowpass);
        comb_lp.set_freq(1000.0);

        let mut comb_comb = IirCombFilter::with_interpolation(true);
        comb_comb.set_freq(6324.0);
        comb_comb.set_gain_db(-6.0);

        comb.set_internal_filters(vec![
            Box::new(comb_peak),
            Box::new(comb_comb),
            Box::new(comb_lp),
        ]);

        let glide_time = 0.001;

        let mut waveshaper = [Waveshaper::new(), Waveshaper::new()];

        for ws in &mut waveshaper {
            ws.set_curve(1.0);
            ws.set_asymmetric(true);
            ws.set_drive(1.0);
            ws.set_xfer_function(xfer::s_curve_round);
        }

        Self {
            filter_lp: [biquad.clone(), biquad.clone()],
            filter_hp: [biquad.clone(), biquad.clone()],
            filter_peak: [biquad.clone(), biquad.clone()],
            filter_comb: [comb.clone(), comb],
            filter_peak_post: [biquad.clone(), biquad.clone()],

            waveshaper,
            drive_amount_receiver: None,

            filter_freq: Ramp::new(440.0, glide_time),
            filter_freq_receiver: None,

            volume: db_to_level(-24.0),
            // volume: db_to_level(-42.0),
            envelope: AdsrEnvelope::new(),
            envelope_trigger: false,
            envelope_trigger_receiver: None,

            glide_time,
        }
    }

    pub fn initialize(&mut self) -> AudioSenders {
        self.set_filters();

        // ENVELOPE PARAMETERS
        // self.envelope.set_parameters(500.0, 2000.0, 0.6, 80.0);
        self.envelope.set_parameters(1.0, 60.0, 0.0, 60.0);
        self.envelope.set_decay_curve(0.9);
        // self.envelope.set_attack_curve(-1.0);

        self.filter_freq.set_smoothing_type(SmoothingType::Linear);

        let (envelope_trigger_sender, receiver) = channel();
        self.envelope_trigger_receiver = Some(receiver);

        let (filter_freq_sender, receiver) = channel();
        self.filter_freq_receiver = Some(receiver);

        let (drive_amount_sender, receiver) = channel();
        self.drive_amount_receiver = Some(receiver);

        AudioSenders {
            envelope_trigger: envelope_trigger_sender,
            filter_freq: filter_freq_sender,
            drive_amount: drive_amount_sender,
        }
    }

    pub fn set_filters(&mut self) {
        let params = BiquadParams {
            freq: 440.0,
            gain: 0.0,
            q: 12.0,
            filter_type: FilterType::Lowpass,
        };

        for lp in &mut self.filter_lp {
            lp.set_params(&params);
        }

        for hp in &mut self.filter_hp {
            hp.set_params(&params);
            hp.set_type(FilterType::Highpass);
        }

        for peak in &mut self.filter_peak {
            peak.set_params(&params);
            peak.set_type(FilterType::Peak);
            peak.set_q(20.0);
            peak.set_gain(30.0);
        }

        for comb in &mut self.filter_comb {
            comb.set_positive_polarity(false);
            comb.set_interpolation(InterpType::Linear);
            comb.set_gain_db(-10.0);
        }

        for peak in &mut self.filter_peak_post {
            peak.set_params(&params);
            peak.set_type(FilterType::Peak);
            peak.set_q(0.3);
            peak.set_gain(18.0);
        }
    }

    pub fn set_filter_freq(&mut self, mut freq: f64) {
        freq = freq.clamp(10.0, unsafe { SAMPLE_RATE } / 2.0);
        for ch in 0..2 {
            self.filter_lp[ch].set_freq(freq);
            self.filter_hp[ch].set_freq(freq);
            self.filter_peak[ch].set_freq(freq);
            self.filter_comb[ch].set_freq(freq);
            self.filter_peak_post[ch].set_freq(freq);
        }
    }

    pub fn process_filters(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        // let l =
        //     self.filter_peak[0].process(self.filter_hp[0].process(sample_l));
        // let r =
        //     self.filter_peak[1].process(self.filter_hp[1].process(sample_r));
        let l = self.filter_peak[0].process(sample_l);
        let r = self.filter_peak[1].process(sample_r);

        (l, r)
    }

    pub fn process_post_peak_filters(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        let l = self.filter_peak_post[0].process(sample_l);
        let r = self.filter_peak_post[1].process(sample_r);

        (l, r)
    }

    pub fn process_comb_filters(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        let l = self.filter_comb[0].process(sample_l);
        let r = self.filter_comb[1].process(sample_r);

        (l, r)
    }

    pub fn process_distortion(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        let l = self.waveshaper[0].process(sample_l);
        let r = self.waveshaper[1].process(sample_r);

        (l, r)
    }

    pub fn try_receive(&mut self) {
        // envelope trigger
        if let Some(trigger) = &self.envelope_trigger_receiver {
            if let Ok(msg) = trigger.try_recv() {
                self.envelope_trigger = msg;
            }
        }

        // filter frequency
        if let Some(freq) = &self.filter_freq_receiver {
            if let Ok(msg) = freq.try_recv() {
                self.filter_freq.set(msg, self.glide_time);
            }
        }

        // waveshaper drive
        if let Some(drive) = &self.drive_amount_receiver {
            if let Ok(msg) = drive.try_recv() {
                for ws in &mut self.waveshaper {
                    ws.set_curve(msg);
                }
            }
        }

        self.filter_freq.next();
    }
}

impl Default for AudioModel {
    fn default() -> Self {
        Self::new()
    }
}

/// The main audio processing callback.
pub fn audio(audio: &mut AudioModel, output: &mut Buffer) {
    for f in output.frames_mut() {
        let noise = || nannou::rand::random_f64().mul_add(2.0, -1.0);

        let env_level = audio.envelope.next(audio.envelope_trigger);
        let volume = audio.volume * (env_level);

        audio.try_receive();
        let freq = audio.filter_freq.current_value();

        if audio.filter_freq.is_active() {
            audio.set_filter_freq(freq);
        }

        let output = (noise() * volume, noise() * volume);
        let output = audio.process_filters(output); // peak filtering
        let output = audio.process_distortion(output); // waveshaping
        let output = audio.process_comb_filters(output); // main comb filters, which contain a
                                                         // peak, highpass, and comb filter
        let output = audio.process_post_peak_filters(output); // wide peak filtering

        f[0] = output.0 as f32;
        f[1] = output.1 as f32;
    }
}
