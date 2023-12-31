//! Audio processing callback.

use crate::{
    dsp::*,
    prelude::xfer::{s_curve_linear_centre, s_curve_round},
};

use super::*;

const SIGNAL_EPSILON: f64 = MINUS_INFINITY_GAIN / 5.0;

/// The main audio processing callback.
pub fn process(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    let dsp_start = std::time::Instant::now();

    // This works by breaking down the buffer into smaller discrete blocks.
    // For each block, it first processes incoming note events, which are
    // obtained from the `VoiceHandler`. The block size is set to min({samples
    // remaining in buffer}, `MAX_BLOCK_SIZE`, {next event index - block start
    // index}).

    // has to be extracted here because it is borrowed in the line below
    let audio_is_idle = audio.is_idle();
    let buffer_len = buffer.len_frames();

    // best not to block at all here - if the VoiceHandler lock can't be
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
    if next_event.is_none() && !voice_handler.is_voice_active() && audio_is_idle
    {
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
                            voice_handler.start_voice(
                                note,
                                audio.data.sample_rate.lr(),
                                Some(audio.generation.amp_envelope.clone()),
                            );
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

    // audio effects/processors
    process_fx(audio, buffer);
    callback_timer(audio);
}

/// Sets the audio callback timer.
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
#[allow(clippy::needless_range_loop)]
fn process_fx(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    // update spectral mask
    if let Some(mask) = &mut audio.buffers.spectral_mask {
        if mask.update() {
            audio.processors.spectral_filter.set_mask(mask.read());
        }
    }

    // set spectral filter
    audio.update_spectral_filter();

    let should_process_reso_bank =
        audio.params.reso_bank_mix.current_value() > f64::EPSILON;

    // process the resonator bank
    for (i, fr) in buffer.frames_mut().enumerate() {
        audio.update_reso_bank();

        for ch in 0..NUM_CHANNELS {
            fr[ch] =
                audio.processors.pre_fx_dc_filter[ch].process_mono(fr[ch], ch);
            if should_process_reso_bank {
                fr[ch] =
                    audio.processors.resonator_bank.process_mono(fr[ch], ch);
            }
            fr[ch] *= 64.0;
        }
    }

    // process the pre-fx spectrum analyser
    audio.compute_pre_spectrum(buffer);

    for frame in buffer.frames_mut() {
        let pre_gain = audio.params.pre_fx_gain.next();

        for ch in 0..NUM_CHANNELS {
            frame[ch] *= pre_gain;
        }
    }

    for (i, fr) in buffer.frames_mut().enumerate() {
        audio.update_post_processors();

        // because ping-pong delay requires cross-feeding channels, it has to
        // be out of the other two loops in the middle here.
        (fr[0], fr[1]) =
            audio.processors.stereo_delay.process_stereo(fr[0], fr[1]);

        // process filters
        for ch in 0..NUM_CHANNELS {
            let mut sample = fr[ch];

            sample = audio.process_filters(sample, ch);

            sample = audio.processors.waveshaper[ch].process(sample);

            sample =
                audio.processors.post_fx_dc_filter[ch].process_mono(sample, ch);

            fr[ch] = sample;
        }

        // process compressor
        (fr[0], fr[1]) =
            audio.processors.compressor.process_stereo(fr[0], fr[1]);
    }

    // process the spectral filter
    audio
        .processors
        .spectral_filter
        .set_mix(audio.params.mask_mix.lr());
    audio.processors.spectral_filter.process_block(buffer);

    // process the post-fx spectrum analyser
    audio.compute_post_spectrum(buffer);

    // UNUSED OVERSAMPLING LOOP 

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
        // let gain = audio.buffers.master_gain_buffer[i];
        let gain = audio.params.master_gain.next();

        output[0] *= gain;
        output[1] *= gain;

        // used to decide whether to skip DSP processing in the next block or not
        if (output[0].abs() > SIGNAL_EPSILON
            || output[1].abs() > SIGNAL_EPSILON)
            && !is_processing
        {
            is_processing = true;
        }

        audio.set_idle_timer(is_processing);
    }

    audio.data.is_processing = is_processing;
}
