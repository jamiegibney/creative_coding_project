use crate::prelude::xfer::smooth_soft_clip;

use super::*;
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;

pub mod audio_constructor;
pub mod builder;
pub mod components;
pub mod params;
pub use builder::*;
pub use components::*;
pub use params::AudioParams;

/// When the DSP stops , it will continue to process for this length of time to
/// allow the audio spectrums to fully relax. After this time has passed, the DSP
/// is skipped to reduce total load when idle.
const DSP_IDLE_HOLD_TIME_SECS: f64 = 0.8;

/// The program's audio state.
pub struct AudioModel {
    /// Fields related to audio generation (envelopes, oscillators, ...).
    pub generation: AudioGeneration,
    /// Signal processors — both musical FX and DSP-related.
    pub processors: AudioProcessors,

    /// Audio-related data (gain, oversampling state, ...).
    pub data: AudioData,
    pub buffers: AudioBuffers,

    /// The pre- and post-FX spectrograms on the audio thread.
    pub spectrograms: AudioSpectrograms,

    /// The audio thread's voice handler.
    pub voice_handler: VoiceHandler,
    /// Audio-related contextual data.
    pub context: AudioContext,

    /// Message receiving channels.
    pub message_channels: RefCell<AudioMessageReceivers>,

    /// All audio-related parameters linked to the UI.
    pub params: AudioParams,

    /// The audio thread pool, intended for processing the spectrograms
    /// asynchronously.
    thread_pool: ThreadPool,
}

impl AudioModel {
    pub fn compute_pre_spectrum(&mut self, buffer: &Buffer<f64>) {
        self.spectrograms
            .pre_fx_spectrogram_buffer
            .try_lock()
            .map_or((), |mut guard| {
                for i in 0..buffer.len() {
                    guard[i] = buffer[i];
                }
            });

        let spectrum = Arc::clone(&self.spectrograms.pre_fx_spectrogram);
        let buffer = Arc::clone(&self.spectrograms.pre_fx_spectrogram_buffer);

        // noone:
        // rust: if let if let if let if let if let
        self.thread_pool.execute(move || {
            if let Ok(mut spectrum) = spectrum.try_lock() {
                if let Some(spectrum) = spectrum.as_mut() {
                    if let Ok(buf) = buffer.try_lock() {
                        spectrum.compute(&buf);
                    }
                }
            }
        });
    }

    pub fn compute_post_spectrum(&mut self, buffer: &Buffer<f64>) {
        self.spectrograms
            .post_fx_spectrogram_buffer
            .try_lock()
            .map_or((), |mut guard| {
                for i in 0..buffer.len() {
                    guard[i] = buffer[i];
                }
            });

        let spectrum = Arc::clone(&self.spectrograms.post_fx_spectrogram);
        let buffer = Arc::clone(&self.spectrograms.post_fx_spectrogram_buffer);

        self.thread_pool.execute(move || {
            if let Ok(mut spectrum) = spectrum.try_lock() {
                if let Some(spectrum) = spectrum.as_mut() {
                    if let Ok(buf) = buffer.try_lock() {
                        spectrum.compute(&buf);
                    }
                }
            }
        });
    }

    pub fn set_idle_timer(&mut self, is_processing: bool) {
        self.data.idle_timer_samples = if is_processing {
            (self.data.sample_rate.load(Relaxed) * DSP_IDLE_HOLD_TIME_SECS)
                as u64
        }
        else if self.data.idle_timer_samples > 0 {
            self.data.idle_timer_samples - 1
        }
        else {
            0
        };
    }

    pub fn is_idle(&self) -> bool {
        !self.data.is_processing && self.data.idle_timer_samples == 0
    }

    /// # Panics
    ///
    /// Panics if the callback timer cannot be locked.
    pub fn current_sample_idx(&self) -> u32 {
        let guard = self.data.callback_time_elapsed.lock().unwrap();

        let samples_exact =
            guard.elapsed().as_secs_f64() * self.data.sample_rate.load(Relaxed);

        drop(guard);

        samples_exact.round() as u32 % BUFFER_SIZE as u32
    }

    /// Returns the internal sample rate of the audio model.
    pub fn get_sample_rate(&self) -> f64 {
        self.data.sample_rate.load(Relaxed)
    }

    /// Returns the internal upsampled rate of the audio model.
    pub fn get_upsampled_rate(&self) -> f64 {
        self.data.upsampled_rate.load(Relaxed)
    }

    /// Returns the next available note event, if it exists.
    pub fn next_note_event(&self) -> Option<NoteEvent> {
        self.message_channels
            .borrow()
            .note_event
            .as_ref()
            .and_then(|ch| ch.try_recv().ok())
    }

    pub fn increment_sample_count(&mut self, buffer_size: u32) {
        let time = 6.0;
        let tmr = (time * self.data.sample_rate.lr()) as u32;

        self.data.sample_timer += buffer_size;
        if self.data.sample_timer > tmr {
            self.processors.resonator_bank.randomize();
            self.data.sample_timer -= tmr;
        }
    }

    pub fn update_spectral_filter_size(&mut self) {
        let param = self.params.mask_resolution.lr().value();

        if param != self.data.spectral_filter_size {
            self.data.spectral_filter_size = param;
            self.processors
                .spectral_filter
                .set_block_size(self.params.mask_resolution.lr().value());
        }
    }

    pub fn update_spectral_filter_order(&mut self) -> bool {
        let param = self.params.mask_is_post_fx.lr();

        if param != self.data.spectral_mask_post_fx {
            self.processors.spectral_filter.clear();
            self.data.spectral_mask_post_fx = param;

            return true;
        }

        false
    }

    pub fn update_reso_bank(&mut self) {
        if let Some(bank_data) = &mut self.buffers.reso_bank_data {
            if bank_data.update() {
                // println!("trying to update reso bank data");
                self.processors
                    .resonator_bank
                    .set_state_from_data(bank_data.read());
            }
        }

        let scale_param = self.params.reso_bank_scale.lr();
        let curr_scale = self.data.reso_bank_scale;

        if scale_param != curr_scale {
            self.data.reso_bank_scale = scale_param;
            self.processors.resonator_bank.set_scale(scale_param);
        }

        let root_note_param = self.params.reso_bank_root_note.lr() as f64;
        let curr_root_note = self.data.reso_bank_root_note;

        if !epsilon_eq(root_note_param, curr_root_note) {
            self.data.reso_bank_root_note = root_note_param;
            self.processors
                .resonator_bank
                .set_root_note(root_note_param);
        }

        if self.params.reso_bank_spread.is_active() {
            self.processors
                .resonator_bank
                .set_freq_spread(self.params.reso_bank_spread.next());
        }
        if self.params.reso_bank_shift.is_active() {
            self.processors
                .resonator_bank
                .set_freq_shift(self.params.reso_bank_shift.next());
        }
        if self.params.reso_bank_inharm.is_active() {
            self.processors
                .resonator_bank
                .set_inharm(self.params.reso_bank_inharm.next());
        }
        if self.params.reso_bank_pan.is_active() {
            self.processors
                .resonator_bank
                .set_panning_scale(self.params.reso_bank_pan.next());
        }
        if self.params.reso_bank_mix.is_active() {
            self.processors
                .resonator_bank
                .set_mix_equal_power(self.params.reso_bank_mix.next());
        }

        self.processors
            .resonator_bank
            .quantize_to_scale(self.params.reso_bank_quantise.lr());

        self.processors.resonator_bank.set_num_resonators(
            self.params.reso_bank_resonator_count.lr() as usize,
        );
    }

    #[allow(clippy::too_many_lines)]
    pub fn update_post_fx_processors(&mut self) {
        let AudioProcessors {
            filter_low,  // arr
            filter_high, // arr

            stereo_delay,

            spectral_filter,

            waveshaper, // arr

            compressor,
            ..
        } = &mut self.processors;

        // delay
        if self.params.delay_mix.is_active() {
            stereo_delay.set_mix_equal_power(self.params.delay_mix.next());
        }
        if self.params.delay_feedback.is_active() {
            stereo_delay.set_feedback_amount(self.params.delay_feedback.next());
        }

        let delay_time = self.params.delay_time_ms.lr();

        stereo_delay.ping_pong(self.params.use_ping_pong.lr());
        stereo_delay.set_delay_time(delay_time * 0.001);

        // compressor
        if self.params.comp_ratio.is_active() {
            compressor.set_ratio(self.params.comp_ratio.next());
        }
        if self.params.comp_thresh.is_active() {
            compressor.set_threshold_level_db(self.params.comp_thresh.next());
        }
        if self.params.comp_attack_ms.is_active() {
            compressor.set_attack_time_ms(self.params.comp_attack_ms.next());
        }
        if self.params.comp_release_ms.is_active() {
            compressor.set_release_time_ms(self.params.comp_release_ms.next());
        }

        // waveshaper
        let param_dist_algo = self.params.dist_type.lr();
        let curr_dist_algo = self.data.distortion_algorithm;

        let update_ws_algo = if param_dist_algo == curr_dist_algo {
            false
        }
        else {
            self.data.distortion_algorithm = param_dist_algo;
            true
        };

        if self.params.dist_amount.is_active() {
            waveshaper[0].set_curve(self.params.dist_amount.next());
            waveshaper[1].set_curve(self.params.dist_amount.current_value());
        }

        for ch in 0..2 {
            if update_ws_algo {
                match self.data.distortion_algorithm {
                    DistortionType::None => {
                        waveshaper[ch].set_asymmetric(false);
                        waveshaper[ch].set_xfer_function(|input, _| input);
                    }
                    DistortionType::Soft => {
                        waveshaper[ch].set_asymmetric(false);
                        waveshaper[ch].set_curve_range(0.0..=1.0);
                        waveshaper[ch].set_xfer_function(
                            |mut input, mut cv| {
                                smooth_soft_clip(input * 5.0, cv) * 0.2
                            },
                        );
                    }
                    DistortionType::Hard => {
                        waveshaper[ch].set_asymmetric(false);
                        waveshaper[ch].set_curve_range(0.01..=1.0);
                        waveshaper[ch].set_xfer_function(
                            |mut input, mut cv| {
                                let cv = (cv * 15.0).max(0.01);

                                ((cv * input).tanh() / cv.tanh())
                                    * map(cv, 0.15, 15.0, 1.0, 0.15)
                            },
                        );
                    }
                    DistortionType::Wrap => {
                        waveshaper[ch].set_curve_range(0.0..=1.0);
                        waveshaper[ch].set_xfer_function(
                            |mut input, mut cv| {
                                cv = 1.0 - cv.clamp(0.0, 1.0);
                                input = input.clamp(-1.0, 1.0);

                                if -1.0 <= input && input <= -cv {
                                    (-2.0f64).mul_add(cv, -input)
                                }
                                else {
                                    input
                                }
                            },
                        );
                    }
                    DistortionType::Crush => {
                        waveshaper[ch].set_curve_range(0.0..=1.0);
                        waveshaper[ch].set_xfer_function(|input, mut cv| {
                            let round = |val: f64| val.floor();

                            cv = scale(cv, 80.0, 2.0);
                            round(cv * input) / cv
                        });
                    }
                }
            }
        }

        let low_fil_shelf = self.params.low_filter_is_shelf.lr();

        if self.data.low_filter_is_shelf != low_fil_shelf {
            self.data.low_filter_is_shelf = low_fil_shelf;

            filter_low[0].set_type(if low_fil_shelf {
                FilterType::Lowshelf
            }
            else {
                FilterType::Highpass
            });

            filter_low[1].set_type(if low_fil_shelf {
                FilterType::Lowshelf
            }
            else {
                FilterType::Highpass
            });
        }

        let high_fil_shelf = self.params.high_filter_is_shelf.lr();

        if self.data.high_filter_is_shelf != high_fil_shelf {
            self.data.high_filter_is_shelf = high_fil_shelf;

            filter_high[0].set_type(if high_fil_shelf {
                FilterType::Highshelf
            }
            else {
                FilterType::Lowpass
            });

            filter_high[1].set_type(if high_fil_shelf {
                FilterType::Highshelf
            }
            else {
                FilterType::Lowpass
            });
        }

        // low filter params
        if self.params.low_filter_cutoff.is_active() {
            filter_low[0].set_freq(self.params.low_filter_cutoff.next());
            filter_low[1]
                .set_freq(self.params.low_filter_cutoff.current_value());
        }
        if self.params.low_filter_gain_db.is_active() {
            filter_low[0].set_gain(self.params.low_filter_gain_db.next());
            filter_low[1]
                .set_gain(self.params.low_filter_gain_db.current_value());
        }
        if self.params.low_filter_q.is_active() {
            let q = if low_fil_shelf {
                BUTTERWORTH_Q
            }
            else {
                self.params.low_filter_q.next()
            };
            filter_low[0].set_q(q);
            filter_low[1].set_q(q);
        }

        if self.params.high_filter_cutoff.is_active() {
            filter_high[0].set_freq(self.params.high_filter_cutoff.next());
            filter_high[1]
                .set_freq(self.params.high_filter_cutoff.current_value());
        }
        if self.params.high_filter_gain_db.is_active() {
            filter_high[0].set_gain(self.params.high_filter_gain_db.next());
            filter_high[1]
                .set_gain(self.params.high_filter_gain_db.current_value());
        }
        if self.params.high_filter_q.is_active() {
            let q = if high_fil_shelf {
                BUTTERWORTH_Q
            }
            else {
                self.params.high_filter_q.next()
            };

            filter_high[0].set_q(q);
            filter_high[1].set_q(q);
        }
    }

    pub fn process_filters(&mut self, mut sample: f64, ch_idx: usize) -> f64 {
        sample = self.processors.filter_low[ch_idx].process(sample);
        sample = self.processors.filter_high[ch_idx].process(sample);

        // tone shaping filters
        sample = self.processors.filter_hs_2[ch_idx].process(sample);
        sample = self.processors.filter_peak[ch_idx].process(sample);

        sample
    }
}
