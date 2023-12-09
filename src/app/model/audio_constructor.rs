use super::*;
use crate::dsp::{filters::comb::delay::Delay, *};
use atomic_float::AtomicF64;
use std::sync::atomic::AtomicUsize;
use triple_buffer::Output;

pub const DEFAULT_SPECTRAL_SIZE: usize = 1 << 10;
pub const DEFAULT_GAIN: f64 = 0.8;

pub fn build_audio_model(mut context: AudioContext) -> AudioPackage {
    let spectral_mask = context.spectral_mask_output.take();
    let sample_rate = context.sample_rate;
    let upsampled_rate = DEFAULT_OVERSAMPLING_FACTOR as f64 * sample_rate;

    AudioModelBuilder::new(context)
        .processors(audio_processors(sample_rate, sample_rate))
        .generation(audio_generation(sample_rate))
        .data(audio_data(sample_rate, sample_rate))
        .buffers(audio_buffers(spectral_mask))
        .build()
}

#[allow(clippy::too_many_lines)]
fn audio_processors(sample_rate: f64, upsampled_rate: f64) -> AudioProcessors {
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
        ws.set_curve(0.0);
        ws.set_asymmetric(true);
        ws.set_drive(0.3);
    }

    let mut spectral_filter = SpectralFilter::new(NUM_CHANNELS, 1 << 14);
    spectral_filter.set_block_size(DEFAULT_SPECTRAL_SIZE);

    let mut filter_lp = st_bq();
    for i in 0..2 {
        filter_lp[i].set_type(FilterType::Lowpass);
        filter_lp[i].set_freq(4000.0);
        filter_lp[i].set_q(BUTTERWORTH_Q);
    }

    let mut filter_hp = st_bq();
    for i in 0..2 {
        filter_hp[i].set_type(FilterType::Highpass);
        filter_hp[i].set_freq(500.0);
        filter_hp[i].set_q(BUTTERWORTH_Q);
    }

    let mut resonator_bank = ResonatorBank::new(upsampled_rate, 256);
    resonator_bank.set_num_resonators(64);
    resonator_bank.set_scale(Scale::MajPentatonic);
    resonator_bank.set_root_note(39.0);
    resonator_bank.set_inharm(0.3);
    resonator_bank.set_freq_spread(0.6);
    resonator_bank.set_freq_shift(-8.0);
    resonator_bank.quantise_to_scale(true);
    resonator_bank.randomise_pan(1.0);
    resonator_bank.randomise_resonator_pitches();
    let mut resonator_bank = DryWet::new(resonator_bank);
    resonator_bank.set_mix_equal_power(0.99);

    let mut resonator = TwoPoleResonator::new(upsampled_rate);
    resonator.set_resonance(0.99);
    resonator.set_cutoff(440.0);

    let mut delay = Delay::new(2.0, upsampled_rate).with_delay_time(0.332);
    delay.set_feedback_amount(0.7);

    let mut delay = DryWet::new(delay);
    delay.set_mix_equal_power(0.1);

    let mut pp_delay =
        PingPongDelay::new(2.0, upsampled_rate).with_delay_time(0.45);
    pp_delay.set_feedback_amount(0.65);

    let mut ping_pong_delay = DryWet::new(pp_delay);
    ping_pong_delay.set_mix_equal_power(0.5);

    let mut high_shelf = BiquadFilter::new(upsampled_rate);
    high_shelf.set_q(BUTTERWORTH_Q);
    high_shelf.set_params(&BiquadParams {
        freq: 800.0,
        gain: 6.0,
        q: BUTTERWORTH_Q,
        filter_type: FilterType::Highshelf,
    });

    let mut filter_hs = st_bq();
    let mut filter_peak = st_bq();
    for i in 0..2 {
        filter_hs[i].set_params(&BiquadParams {
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

    AudioProcessors {
        filter_lp,
        filter_hp,
        filter_hs,
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
        ping_pong_delay: Box::new(ping_pong_delay),

        resonator_bank,
        resonator: Box::new([resonator.clone(), resonator]),

        spectral_filter,
        waveshaper: Box::new(waveshaper),
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
    amp_envelope.set_parameters(300.1, 300.0, 0.0, 20.0);

    AudioGeneration { amp_envelope, generator: Generator::Noise }
}

fn audio_data(sample_rate: f64, upsampled_rate: f64) -> AudioData {
    AudioData {
        voice_gain: Smoother::new(1.0, 0.01, sample_rate),
        master_gain: Smoother::new(1.0, DEFAULT_GAIN, upsampled_rate),
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
        sample_timer: 0,
        callback_time_elapsed: Arc::new(Mutex::new(Instant::now())),
        spectral_mask_post_fx: false,
    }
}

fn audio_buffers(spectral_mask: Option<Output<SpectralMask>>) -> AudioBuffers {
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
    }
}
