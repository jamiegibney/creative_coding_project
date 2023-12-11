use crate::{
    dsp::filtering::comb::distortion::waveshaper::smooth_soft_clip,
    prelude::xfer::{s_curve_linear_centre, s_curve_round},
};

use super::*;

const SIGNAL_EPSILON: f64 = MINUS_INFINITY_GAIN / 5.0;

/// The main audio processing callback.
pub fn process(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    let dsp_start = std::time::Instant::now();

    // This works by breaking down the buffer into smaller discrete blocks.
    // For each block, it first processes incoming note events, which are
    // obtained from the `NoteHandler`. The block size is set to min({samples
    // remaining in buffer}, `MAX_BLOCK_SIZE`, {next event index - block start
    // index}).

    // has to be extracted here because it is borrowed in the line below
    let audio_is_idle = audio.is_idle();
    let buffer_len = buffer.len_frames();

    // best not to block at all here - if the NoteHandler lock can't be
    // obtained, then the note events won't be processed for this buffer.
    // let mut note_handler_guard = context.note_handler.try_lock().ok();
    // let mut next_event =
    //     note_handler_guard.as_mut().and_then(|g| g.next_event());
    let mut next_event = audio
        .message_channels
        .borrow()
        .note_event
        .as_ref()
        .and_then(|ch| ch.try_recv().ok());

    let voice_handler = &mut audio.voice_handler;

    // if there is no note event, no active voice, and there was no audio
    // processed in the last frame, most of the signal processing can be skipped.
    if next_event.is_none() && !voice_handler.is_voice_active() && audio_is_idle {
        callback_timer(audio);
        return;
    }

    let mut block_start: usize = 0;
    let mut block_end = MAX_BLOCK_SIZE.min(buffer_len);

    // audio generators
    while block_start < buffer_len {
        // first, handle incoming events.
        'events: loop {
            match next_event {
                // if the event is now (or before the block), match
                // the event and handle its voice accordingly.
                Some(event) if (event.timing() as usize) <= block_start => {
                    match event {
                        NoteEvent::NoteOn { note, .. } => {
                            voice_handler
                                .start_voice(note, Some(audio.generation.amp_envelope.clone()));
                        }
                        NoteEvent::NoteOff { note, .. } => {
                            voice_handler.start_release_for_voice(None, note);
                        }
                    }

                    // then obtain the next event and loop again
                    next_event = audio
                        .message_channels
                        .borrow()
                        .note_event
                        .as_ref()
                        .and_then(|ch| ch.try_recv().ok());
                }
                // if the event exists within this block, set the next block
                // to start at the event and continue processing the block
                Some(event) if (event.timing() as usize) < block_end => {
                    block_end = event.timing() as usize;
                    break 'events;
                }
                _ => break 'events,
            }
        }

        let block_len = block_end - block_start;

        let mut gain = [0.0; MAX_BLOCK_SIZE];
        audio.data.voice_gain.next_block(&mut gain, block_len);

        voice_handler.process_block(buffer, block_start, block_end, gain);

        voice_handler.terminate_finished_voices();

        block_start = block_end;
        block_end = (block_end + MAX_BLOCK_SIZE).min(buffer_len);
    }

    // drop(note_handler_guard);

    audio.compute_pre_spectrum(buffer);

    // audio effects/processors
    process_fx(audio, buffer);

    audio.compute_post_spectrum(buffer);

    // print_dsp_load(audio, dsp_start);
    callback_timer(audio);
}

// /// Captures the audio buffer pre-FX, then sends it to a separate thread to
// /// compute a spectrum.
// ///
// /// Paired with the below `compute_post_spectrum()` function, processing the
// /// audio spectra on separate threads **enormously** reduces the DSP load.
// /// Some quick benchmarking showed a load reduction of around *one order of
// /// magnitude* on the audio thread. Vroom.
// fn compute_pre_fx_spectrum(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
//     // copy the buffer pre-fx to the audio model
//     audio.pre_buffer_cache.try_lock().map_or((), |mut guard| {
//         for i in 0..buffer.len() {
//             guard[i] = buffer[i];
//         }
//     });
//
//     // then compute the pre-fx spectrum on a separate thread
//     audio.compute_pre_spectrum();
// }
//
// /// Captures the audio buffer post-FX, then sends it to a separate thread to
// /// compute a spectrum.
// fn compute_post_fx_spectrum(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
//     // copy the buffer post-fx to the audio model
//     audio.post_buffer_cache.try_lock().map_or((), |mut guard| {
//         for i in 0..buffer.len() {
//             guard[i] = buffer[i];
//         }
//     });
//
//     // then compute the post-fx spectrum on a separate thread
//     audio.compute_post_spectrum();
// }

fn callback_timer(audio: &AudioModel) {
    // the chance of not being able to acquire the lock is very small here,
    // but because this is the audio thread, it's preferable to not block at
    // all. so if the lock can't be obtained, then the callback_time_elapsed
    // will temporarily not be reset. this won't cause issues in the context
    // of this program.
    if let Ok(mut guard) = audio.data.callback_time_elapsed.try_lock() {
        if guard.elapsed().as_secs_f64() >= 0.0001 {
            *guard = std::time::Instant::now();
        }
    }
}

/// Processes all audio FX.
fn process_fx(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    // try to receive any messages from other threads
    audio.try_receive();
    // audio.increment_sample_count(buffer.len() as u32);

    // update spectral mask
    // if let Ok(guard) = audio.spectral_mask.try_lock() {
    //     audio.spectral_filter.set_mask(&guard);
    // }
    if let Some(mask) = &mut audio.buffers.spectral_mask {
        if mask.update() {
            audio.processors.spectral_filter.set_mask(mask.read());
        }
    }
    // if audio.buffers.spectral_mask.update() {
    //     audio.processors.spectral_filter.set_mask(audio.spectral_mask.read());
    // }

    // process the spectral masking
    if !audio.data.spectral_mask_post_fx {
        audio.processors.spectral_filter.process_block(buffer);
    }

    // TODO: how does this account for different buffer sizes?
    audio
        .data
        .master_gain
        // is this not better?
        .next_block(&mut audio.buffers.master_gain_buffer, buffer.len());
    // .next_block_exact(&mut audio.buffers.master_gain_buffer);

    #[allow(clippy::needless_range_loop)]
    for (i, fr) in buffer.frames_mut().enumerate() {
        for ch in 0..NUM_CHANNELS {
            fr[ch] = audio.processors.pre_fx_dc_filter[ch].process_mono(fr[ch], ch);
        }

        let (mut l, mut r) = audio.processors.resonator_bank.process_stereo(fr[0], fr[1]);

        (l, r) = audio.processors.ping_pong_delay.process_stereo(l, r);

        fr[0] = l;
        fr[1] = r;

        for ch in 0..NUM_CHANNELS {
            let mut sample = fr[ch];
            sample = audio.processors.filter_hs[ch].process(sample);
            sample = audio.processors.filter_peak[ch].process(sample);
            // sample = audio.processors.filter_lp[ch].process(sample);
            sample = audio.processors.filter_hp[ch].process(sample);
            sample = smooth_soft_clip(sample, 1.0);
            sample = audio.processors.post_fx_dc_filter[ch].process_mono(sample, ch);

            fr[ch] = sample;
        }
    }

    // process the spectral masking
    if audio.data.spectral_mask_post_fx {
        audio.processors.spectral_filter.process_block(buffer);
    }

    // // copy the audio buffer into the oversampling buffer so the channel layout
    // // is compatible with the oversamplers.
    // audio.oversampling_buffer.copy_from_buffer(buffer);
    //
    // // compute gain information
    // audio.master_gain.next_block_exact(&mut audio.gain_data);
    //
    // // process the oversampling
    // for ((ch, block), oversampler) in audio
    //     .oversampling_buffer
    //     .iter_mut()
    //     .enumerate()
    //     .zip(audio.oversamplers.iter_mut())
    // {
    //     oversampler.process(
    //         &mut block[..BUFFER_SIZE],
    //         audio
    //             .oversampling_factor
    //             .load(std::sync::atomic::Ordering::Relaxed),
    //         |upsampled| {
    //             // a single channel iterating through each upsampled sample
    //             for (smp_idx, sample) in upsampled.iter_mut().enumerate() {
    //                 *sample = audio.pre_dc_filter[ch].process_mono(*sample);
    //                 *sample = audio.waveshaper[ch].process(*sample);
    //                 *sample = audio.post_dc_filter[ch].process_mono(*sample);
    //                 // *sample = audio.filter_lp[ch].process(*sample);
    //
    //                 *sample *= audio.gain_data[smp_idx];
    //             }
    //         },
    //     );
    // }
    //
    // // copy the oversampling buffer content back to the main audio buffer with the
    // // correct channel layout.
    // audio.oversampling_buffer.copy_to_buffer(buffer);

    // final loop
    let mut is_processing = false;
    for (i, output) in buffer.frames_mut().enumerate() {
        let gain = audio.buffers.master_gain_buffer[i];
        let ceiling = db_to_level(-6.0);
        // hard-clip output;
        output[0] = output[0].clamp(-ceiling, ceiling) * gain;
        output[1] = output[1].clamp(-ceiling, ceiling) * gain;

        // used to decide whether to skip DSP processing in the next block or not
        if (output[0].abs() > SIGNAL_EPSILON || output[1].abs() > SIGNAL_EPSILON) && !is_processing
        {
            is_processing = true;
        }

        audio.set_idle_timer(is_processing);
    }

    audio.data.is_processing = is_processing;
}

/* fn process_old(audio: &mut AudioModel, output: &mut Buffer) {
    for f in output.frames_mut() {
        let env_level = audio.envelope.next(audio.envelope_trigger);
        let volume = audio.volume * (env_level);
        let noise = || nannou::rand::random_f64().mul_add(2.0, -1.0) * volume;

        audio.try_receive();
        let freq = audio.filter_freq.current_value();

        if audio.filter_freq.is_active() {
            audio.set_filter_freq(freq);
        }

        let output = (noise(), noise());
        let output = audio.process_filters(output); // peak filtering
        let output = audio.process_distortion(output); // waveshaping
        let output = audio.process_comb_filters(output); // main comb filters, which contain a
                                                         // peak, highpass, and comb filter

        let output = audio.process_post_peak_filters(output); // wide peak filtering
        f[0] = output.0 as f32;
        f[1] = output.1 as f32;
    }
} */
