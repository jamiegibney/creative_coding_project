use super::*;
use crate::dsp::{
    filtering::{comb::delay::Delay, resonator::resonator_bank::ResoBankData},
    *,
};
use atomic_float::AtomicF64;
use std::sync::atomic::AtomicUsize;
use triple_buffer::Output;

pub const DEFAULT_SPECTRAL_BLOCK_SIZE: usize = 1 << 10; // 1024
pub const DEFAULT_GAIN: f64 = 1.5;
pub const MAX_NUM_RESONATORS: usize = 32;

pub fn build_audio_model(
    mut context: AudioContext,
    ui_params: &UIParams,
) -> AudioPackage {
    let spectral_mask = context.spectral_mask_output.take();
    let reso_bank_data = context.reso_bank_data_output.take();
    let sample_rate = context.sample_rate;
    let upsampled_rate = DEFAULT_OVERSAMPLING_FACTOR as f64 * sample_rate;

    AudioModelBuilder::new(context)
        .processors(audio_processors(sample_rate, sample_rate, ui_params))
        .generation(audio_generation(sample_rate))
        .data(audio_data(sample_rate, sample_rate, ui_params))
        .buffers(audio_buffers(spectral_mask, reso_bank_data))
        .params(ui_params)
        .build()
}

#[allow(clippy::too_many_lines)]
fn audio_processors(
    sample_rate: f64,
    upsampled_rate: f64,
    ui_params: &UIParams,
) -> AudioProcessors {
    let st_bq = || {
        Box::new([
            BiquadFilter::new(upsampled_rate),
            BiquadFilter::new(upsampled_rate),
        ])
    };

    let mut comb = IirCombFilter::with_interpolation(true, upsampled_rate);
    // comb.
    comb.set_freq(3.0);
    comb.set_gain_db(-3.0);

    // let mut comb_peak = BiquadFilter::new(upsampled_rate);
    // comb_peak.set_params(&BiquadParams {
    //     freq: 726.0,
    //     gain: 4.0,
    //     q: 0.4,
    //     filter_type: FilterType::Peak,
    // });
    //
    // let mut comb_lp = FirstOrderFilter::new(upsampled_rate);
    // comb_lp.set_type(FilterType::Lowpass);
    // comb_lp.set_freq(3000.0);

    // comb.set_internal_filters(vec![Box::new(comb_peak), Box::new(comb_lp)]);

    let mut waveshaper = [Waveshaper::new(), Waveshaper::new()];

    for ws in &mut waveshaper {
        ws.set_curve(ui_params.dist_amount.current_value());
        ws.set_asymmetric(false);
        ws.set_drive(1.0);
        ws.set_xfer_function(|input, _| input);
    }

    let mut spectral_filter =
        SpectralFilter::new(NUM_CHANNELS, MAX_SPECTRAL_BLOCK_SIZE);
    spectral_filter.set_block_size(ui_params.mask_resolution.lr().value());

    let mut filter_low = st_bq();
    let mut filter_high = st_bq();

    for ch in 0..2 {
        filter_low[ch].set_type(FilterType::Highpass);
        filter_low[ch].set_freq(ui_params.low_filter_cutoff.current_value());
        filter_low[ch].set_gain(ui_params.low_filter_gain_db.current_value());

        filter_high[ch].set_type(FilterType::Highshelf);
        filter_high[ch].set_freq(ui_params.high_filter_cutoff.current_value());
        filter_high[ch].set_gain(ui_params.high_filter_gain_db.current_value());
    }

    let mut resonator_bank =
        ResonatorBank::new(upsampled_rate, MAX_NUM_RESONATORS);
    resonator_bank
        .set_num_resonators(ui_params.reso_bank_resonator_count.lr() as usize);
    resonator_bank.set_scale(ui_params.reso_bank_scale.lr());
    resonator_bank.set_root_note(ui_params.reso_bank_root_note.lr() as f64);
    resonator_bank.set_inharm(ui_params.reso_bank_inharm.current_value());
    resonator_bank.set_freq_spread(0.5);
    resonator_bank.set_freq_shift(ui_params.reso_bank_shift.current_value());
    resonator_bank.quantize_to_scale(ui_params.reso_bank_quantize.lr());
    resonator_bank.set_panning_scale(ui_params.reso_bank_pan.current_value());
    resonator_bank.randomize();

    let mut resonator_bank = DryWet::new(resonator_bank);
    resonator_bank.set_mix_equal_power(ui_params.reso_bank_mix.current_value());

    let mut resonator = TwoPoleResonator::new(upsampled_rate);
    resonator.set_resonance(0.99);
    resonator.set_cutoff(440.0);

    let mut delay = Delay::new(2.0, upsampled_rate).with_delay_time(0.332);
    delay.set_feedback_amount(0.7);

    let mut delay = DryWet::new(delay);
    delay.set_mix_equal_power(0.1);

    let mut st_delay = StereoDelay::new(1.0, upsampled_rate)
        .with_delay_time(ui_params.delay_time_ms.lr() * 0.001)
        .with_ping_pong(ui_params.use_ping_pong.lr());
    st_delay.set_feedback_amount(ui_params.delay_feedback.current_value());

    let mut stereo_delay = DryWet::new(st_delay);
    stereo_delay.set_mix_equal_power(ui_params.delay_mix.current_value());

    // tone-shaping filters
    let mut filter_hs_2 = st_bq();
    let mut filter_peak = st_bq();
    for i in 0..2 {
        filter_hs_2[i].set_params(&BiquadParams {
            freq: 3200.0,
            gain: 2.0,
            q: BUTTERWORTH_Q,
            filter_type: FilterType::Highshelf,
        });
        filter_peak[i].set_params(&BiquadParams {
            freq: 300.0,
            gain: -1.5,
            q: BUTTERWORTH_Q,
            filter_type: FilterType::Peak,
        });
    }

    let mut compressor = Compressor::new(upsampled_rate);
    compressor.set_threshold_level_db(ui_params.comp_thresh.current_value());
    compressor.set_ratio(ui_params.comp_ratio.current_value());
    compressor.set_attack_time_ms(ui_params.comp_attack_ms.current_value());
    compressor.set_release_time_ms(ui_params.comp_release_ms.current_value());
    compressor.set_knee_width(5.0);
    compressor.use_rms(false);

    AudioProcessors {
        filter_low,
        filter_high,

        filter_hs_2,
        filter_peak: st_bq(),
        filter_peak_post: st_bq(),
        filter_comb: Box::new([comb.clone(), comb]),

        pre_fx_dc_filter: Box::new(std::array::from_fn(|_| {
            DCFilter::new(upsampled_rate, 2)
        })),
        post_fx_dc_filter: Box::new(std::array::from_fn(|_| {
            DCFilter::new(upsampled_rate, 2)
        })),

        delay: Box::new([delay.clone(), delay]),
        stereo_delay: Box::new(stereo_delay),

        resonator_bank,
        resonator: Box::new([resonator.clone(), resonator]),

        spectral_filter,
        waveshaper: Box::new(waveshaper),

        compressor: Box::new(compressor),

        oversamplers: vec![
            Oversampler::new(
                MAX_BUFFER_SIZE, MAX_OVERSAMPLING_FACTOR, 3
            );
            NUM_CHANNELS
        ],
    }
}

fn audio_generation(sample_rate: f64) -> AudioGeneration {
    let mut amp_envelope = AdsrEnvelope::new(sample_rate);
    amp_envelope.set_parameters(15.0, 300.0, 1.0, 20.0);

    AudioGeneration { amp_envelope, generator: Generator::Noise }
}

fn audio_data(
    sample_rate: f64,
    upsampled_rate: f64,
    ui_params: &UIParams,
) -> AudioData {
    AudioData {
        voice_gain: Smoother::new(1.0, 0.01, sample_rate),
        // master_gain: Arc::new(SmootherAtomic::new(
        //     1.0, DEFAULT_GAIN, upsampled_rate,
        // )),
        sample_rate: Arc::new(AtomicF64::new(sample_rate)),
        upsampled_rate: Arc::new(AtomicF64::new(upsampled_rate)),
        latency_samples: 0,
        oversampling_factor: Arc::new(AtomicUsize::new(
            DEFAULT_OVERSAMPLING_FACTOR,
        )),
        is_processing: false,
        idle_timer_samples: 0,
        average_load: vec![0.0; DSP_LOAD_AVERAGING_SAMPLES],
        average_pos: 0,
        distortion_algorithm: ui_params.dist_type.lr(),
        sample_timer: 0,
        callback_time_elapsed: Arc::new(Mutex::new(std::time::Instant::now())),

        spectral_mask_post_fx: ui_params.mask_is_post_fx.lr(),
        spectral_filter_size: ui_params.mask_resolution.lr().value(),

        reso_bank_scale: ui_params.reso_bank_scale.lr(),
        reso_bank_root_note: ui_params.reso_bank_root_note.lr() as f64,

        delay_time_ms: 250.0,

        high_filter_is_shelf: ui_params.high_filter_is_shelf.lr(),
        low_filter_is_shelf: ui_params.low_filter_is_shelf.lr(),
    }
}

fn audio_buffers(
    spectral_mask: Option<Output<SpectralMask>>,
    reso_bank_data: Option<Output<ResoBankData>>,
) -> AudioBuffers {
    AudioBuffers {
        master_gain_buffer: vec![
            DEFAULT_GAIN;
            BUFFER_SIZE
                * (1 << DEFAULT_OVERSAMPLING_FACTOR)
        ],
        oversampling_buffer: OversamplingBuffer::new(
            NUM_CHANNELS, MAX_BUFFER_SIZE,
        ),
        spectral_mask,
        reso_bank_data,
    }
}
