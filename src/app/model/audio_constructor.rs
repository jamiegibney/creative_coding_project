use super::*;
use crate::dsp::*;
use atomic_float::AtomicF64;
use std::sync::atomic::AtomicUsize;
use triple_buffer::Output;

pub const DEFAULT_SPECTRAL_SIZE: usize = 1 << 10;
pub const DEFAULT_GAIN: f64 = 0.008;

const DSP_IDLE_HOLD_TIME_SECS: f64 = 0.8;

pub fn build_audio_model(mut context: AudioContext) -> AudioPackage {
    let spectral_mask = context.spectral_mask_output.take();
    let sample_rate = context.sample_rate;
    let upsampled_rate = DEFAULT_OVERSAMPLING_FACTOR as f64 * sample_rate;

    AudioModelBuilder::new(context)
        .processors(audio_processors(sample_rate, upsampled_rate))
        .generation(audio_generation(sample_rate))
        .data(audio_data(sample_rate, upsampled_rate))
        .buffers(audio_buffers(spectral_mask))
        .build()
}

fn audio_processors(sample_rate: f64, upsampled_rate: f64) -> AudioProcessors {
    let st_bq = || {
        Box::new([
            BiquadFilter::new(upsampled_rate),
            BiquadFilter::new(upsampled_rate),
        ])
    };

    let mut comb = IirCombFilter::with_interpolation(true, upsampled_rate);
    comb.set_freq(10.0);
    comb.set_gain_db(-0.001);

    let mut comb_peak = BiquadFilter::new(upsampled_rate);
    comb_peak.set_params(&BiquadParams {
        freq: 726.0,
        gain: 4.0,
        q: 0.4,
        filter_type: FilterType::Peak,
    });

    let mut comb_lp = FirstOrderFilter::new(upsampled_rate);
    comb_lp.set_type(FilterType::Lowpass);
    comb_lp.set_freq(3000.0);

    comb.set_internal_filters(vec![Box::new(comb_peak), Box::new(comb_lp)]);

    let mut waveshaper = [Waveshaper::new(), Waveshaper::new()];

    for ws in &mut waveshaper {
        ws.set_curve(0.1);
        ws.set_asymmetric(false);
        ws.set_drive(0.1);
        ws.set_xfer_function(xfer::s_curve);
    }

    let mut spectral_filter = SpectralFilter::new(NUM_CHANNELS, 1 << 14);
    spectral_filter.set_block_size(DEFAULT_SPECTRAL_SIZE);

    AudioProcessors {
        filter_lp: st_bq(),
        filter_hp: st_bq(),
        filter_peak: st_bq(),
        filter_peak_post: st_bq(),
        filter_comb: Box::new([comb.clone(), comb]),

        pre_fx_dc_filter: Box::new(std::array::from_fn(|_| {
            DCFilter::new(upsampled_rate, 2)
        })),
        post_fx_dc_filter: Box::new(std::array::from_fn(|_| {
            DCFilter::new(upsampled_rate, 2)
        })),

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
    amp_envelope.set_parameters(0.0, 300.0, 1.0, 10.0);

    AudioGeneration { amp_envelope, generator: Generator::Noise }
}

fn audio_data(sample_rate: f64, upsampled_rate: f64) -> AudioData {
    AudioData {
        voice_gain: Smoother::new(1.0, 0.5, sample_rate),
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
        callback_time_elapsed: Arc::new(Mutex::new(Instant::now())),
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
