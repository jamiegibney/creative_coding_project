use super::*;

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
    let AudioModel { context, voice_handler, .. } = audio;
    let buffer_len = buffer.len_frames();

    // best not to block at all here - if the NoteHandler lock can't be
    // obtained, then the note events won't be processed for this buffer.
    let mut note_handler_guard = context.note_handler.try_lock().ok();
    let mut next_event =
        note_handler_guard.as_mut().and_then(|g| g.next_event());

    // if there is no note event, no active voice, and there was no audio
    // processed in the last frame, most of the signal processing can be skipped.
    if next_event.is_none() && !voice_handler.is_voice_active() && audio_is_idle
    {
        drop(note_handler_guard);
        print_dsp_load(audio, dsp_start);
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
                                Some(audio.amp_envelope.clone()),
                            );
                        }
                        NoteEvent::NoteOff { note, .. } => {
                            voice_handler.start_release_for_voice(None, note);
                        }
                    }

                    // then obtain the next event and loop again
                    // SAFETY: this is ok, because this pattern would not
                    // match if the note_handler_guard was not obtained.
                    next_event = unsafe {
                        note_handler_guard
                            .as_mut()
                            .unwrap_unchecked()
                            .next_event()
                    };
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
        audio.gain.next_block(&mut gain, block_len);

        voice_handler.process_block(buffer, block_start, block_end, gain);

        voice_handler.terminate_finished_voices();

        block_start = block_end;
        block_end = (block_end + MAX_BLOCK_SIZE).min(buffer_len);
    }

    drop(note_handler_guard);

    compute_pre_fx_spectrum(audio, buffer);

    // audio effects/processors
    process_fx(audio, buffer);

    compute_post_fx_spectrum(audio, buffer);

    // print_dsp_load(audio, dsp_start);
    callback_timer(audio);
}

/// Captures the audio buffer pre-FX, then sends it to a separate thread to
/// compute a spectrum.
///
/// Paired with the below `compute_post_spectrum()` function, processing the
/// audio spectra on separate threads **enormously** reduces the DSP load.
/// Some quick benchmarking showed a load reduction of around *one order of
/// magnitude* on the audio thread. Vroom.
fn compute_pre_fx_spectrum(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    // copy the buffer pre-fx to the audio model
    audio.pre_buffer_cache.try_lock().map_or((), |mut guard| {
        for i in 0..buffer.len() {
            guard[i] = buffer[i];
        }
    });

    // then compute the pre-fx spectrum on a separate thread
    audio.compute_pre_spectrum();
}

/// Captures the audio buffer post-FX, then sends it to a separate thread to
/// compute a spectrum.
fn compute_post_fx_spectrum(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    // copy the buffer post-fx to the audio model
    audio.post_buffer_cache.try_lock().map_or((), |mut guard| {
        for i in 0..buffer.len() {
            guard[i] = buffer[i];
        }
    });

    // then compute the post-fx spectrum on a separate thread
    audio.compute_post_spectrum();
}

fn print_dsp_load(audio: &mut AudioModel, start_time: std::time::Instant) {
    if PRINT_DSP_LOAD {
        let total_buf_time = sample_length() * BUFFER_SIZE as f64;
        audio.average_load[audio.avr_pos] = start_time.elapsed().as_secs_f64();

        let avr = audio.average_load.iter().sum::<f64>()
            / DSP_LOAD_AVERAGING_SAMPLES as f64;
        println!("DSP load: {:.2}%", avr / total_buf_time * 100.0);

        audio.avr_pos = (audio.avr_pos + 1) % DSP_LOAD_AVERAGING_SAMPLES;
    }
}

fn callback_timer(audio: &mut AudioModel) {
    // the chance of not being able to acquire the lock is very small here,
    // but because this is the audio thread, it's preferable to not block at
    // all. so if the lock can't be obtained, then the callback_time_elapsed
    // will temporarily not be reset. this won't cause issues in the context
    // of this program.
    audio
        .callback_time_elapsed
        .try_lock()
        .map_or((), |mut guard| {
            if guard.elapsed().as_secs_f64() >= 0.0001 {
                *guard = std::time::Instant::now();
            }
        });
}

/// Processes all audio FX.
fn process_fx(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    // audio effects/processors

    // copy the audio buffer into the oversampling buffer so the channel layout
    // is compatible with the oversamplers.
    audio.oversampling_buffer.copy_from_buffer(buffer);

    // process the oversampling
    for ((ch, block), oversampler) in audio
        .oversampling_buffer
        .iter_mut()
        .enumerate()
        .zip(audio.oversamplers.iter_mut())
    {
        oversampler.process(
            &mut block[..BUFFER_SIZE],
            audio
                .oversampling_factor
                .load(std::sync::atomic::Ordering::Relaxed),
            |upsampled| {
                // a single channel iterating through each upsampled sample
                for (i, sample) in upsampled.iter_mut().enumerate() {
                    // *sample = audio.filter_lp[ch].process(*sample);
                    // *sample = audio.filter_comb[ch].process(*sample);
                    *sample = audio.waveshaper[ch].process(*sample);
                    *sample *= 2.5;
                }
            },
        );
    }

    // copy the oversampling buffer content back to the main audio buffer with the
    // correct channel layout.
    audio.oversampling_buffer.copy_to_buffer(buffer);

    let mut is_processing = false;

    for output in buffer.frames_mut() {
        output[0] = output[0].clamp(-1.0, 1.0);
        output[1] = output[1].clamp(-1.0, 1.0);

        // this seems to be a better level to compare against than f64::EPSILON
        let signal_epsilon = MINUS_INFINITY_GAIN / 5.0;

        // used to decide whether to skip DSP processing in the next block or not.
        if output[0].abs() > signal_epsilon || output[1].abs() > signal_epsilon
        {
            is_processing = true;
        }

        audio.set_idle_timer(is_processing);
    }

    audio.is_processing = is_processing;
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
