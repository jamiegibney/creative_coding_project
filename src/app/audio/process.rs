use super::*;

/// The main audio processing callback.
#[allow(clippy::missing_panics_doc)]
pub fn process(audio: &mut AudioModel, buffer: &mut Buffer) {
    // This works by breaking down the buffer into smaller discrete blocks.
    // For each block, it first processes incoming note events, which are
    // obtained from the `NoteHandler`. The block size is set to min({samples
    // remaining in buffer}, `MAX_BLOCK_SIZE`, {next event index - block start
    // index}).

    let AudioModel { context, voice_handler, .. } = audio;

    let buffer_len = buffer.len_frames();
    let mut note_handler_guard = context.note_handler.lock().unwrap();

    let mut next_event = note_handler_guard.next_event();
    let mut block_start: usize = 0;
    let mut block_end = MAX_BLOCK_SIZE.min(buffer_len);

    while block_start < buffer_len {
        // first, handle incoming events.
        'events: loop {
            match next_event {
                Some(event) if (event.timing() as usize) <= block_start => {
                    match event {
                        NoteEvent::NoteOn { note, .. } => {
                            voice_handler
                                .start_voice(note, todo!("add new envelope!"));
                        }
                        NoteEvent::NoteOff { note, .. } => {
                            voice_handler.start_release_for_voice(None, note);
                        }
                    }

                    next_event = note_handler_guard.next_event();
                }
                Some(event) if (event.timing() as usize) < block_end => {
                    block_end = event.timing() as usize;
                    break 'events;
                }
                _ => break 'events,
            }
        }

        // * 2 used because the channels are interleaved
        buffer[block_start..block_end * 2].fill(0.0);

        let block_len = block_end - block_start;
        let mut gain = [0.0; MAX_BLOCK_SIZE];
        // TODO: have "next_block" method for Ramp
        for x in gain.iter_mut().take(block_len) {
            *x = audio.gain.next();
        }

        voice_handler.process_block(buffer, block_start, block_end, gain);

        voice_handler.terminate_finished_voices();

        block_start = block_end;
        block_end = (block_start + MAX_BLOCK_SIZE).min(buffer_len);
    }

    drop(note_handler_guard);

    // effects go here...

    for output in buffer.frames_mut() {
        // let (l, r) = audio.process_effects(ouput[0], output[1]);
        //
        // output[0] = l;
        // output[1] = r;
    }
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
