use super::*;

/// The main audio processing callback.
pub fn process(audio: &mut AudioModel, buffer: &mut Buffer<f64>) {
    // This works by breaking down the buffer into smaller discrete blocks.
    // For each block, it first processes incoming note events, which are
    // obtained from the `NoteHandler`. The block size is set to min({samples
    // remaining in buffer}, `MAX_BLOCK_SIZE`, {next event index - block start
    // index}).

    buffer.fill(0.0);

    let AudioModel { context, voice_handler, .. } = audio;
    let buffer_len = buffer.len_frames();

    // best not to block at all here - if the NoteHandler lock can't be
    // obtained, then the note events won't be processed for this buffer.
    let mut note_handler_guard = context.note_handler.try_lock().ok();
    let mut next_event =
        note_handler_guard.as_mut().and_then(|g| g.next_event());

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

    if let Some(pre_spectrum) = audio.pre_spectrum.as_mut() {
        pre_spectrum.compute(buffer);
    }

    drop(note_handler_guard);

    // audio effects/processors
    // for output in buffer.frames_mut() {
    //     let (l, r) = (output[0], output[1]);
    //     // let (l, r) = audio.process_comb_filters((l, r));
    //     // let (l, r) = audio.process_distortion((l, r));
    //     // let (l, r) = audio.process_filters((l, r));
    //
    //     output[0] = l;
    //     output[1] = r;
    // }

    for output in buffer.frames_mut() {
        output[0] = output[0].clamp(-1.0, 1.0);
        output[1] = output[1].clamp(-1.0, 1.0);
    }

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
