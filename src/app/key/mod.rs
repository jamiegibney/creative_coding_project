use super::*;
use crate::app::audio::VoiceEvent;

/// Function for handling keypresses.
pub fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::P => model
            .voice_event_sender
            .send(VoiceEvent::ReleaseAll)
            .unwrap(),
        Key::Z => model.octave.decrease(),
        Key::X => model.octave.increase(),
        Key::R => match model.ui_params.mask_algorithm.lr() {
            GenerativeAlgo::Contours => {
                // let mut ctr = model.contours.as_mut().unwrap().write().unwrap();
                let mut ctr = model.contours.write().unwrap();

                ctr.randomize();
                drop(ctr);
            }
            GenerativeAlgo::SmoothLife => {
                // let mut sml = model.smooth_life.as_mut().unwrap().write().unwrap();
                let mut sml = model.smooth_life.write().unwrap();

                sml.randomize();
                drop(sml);
            }
            GenerativeAlgo::Voronoi => {
                // let mut
                todo!();
            }
        },
        Key::Tab => {
            // reso bank push
        }
        Key::Space => {
            // reso bank reset
        }
        _ => (),
    };

    if let Some(v) = model.pressed_keys.get_mut(&key) {
        if *v {
            return;
        }
        *v = true;
    }
    else {
        return;
    }

    // get midi note value from keyboard input
    let key_note_value = Note::key_value(&key);

    if let Some(note) = key_note_value {
        // transpose octave if higher keys are pressed
        let octave = octave_from_key(model.octave, key);

        // get the midi note value from octave and key note
        let note = octave.starting_midi_note() + note;

        // get the approximate number of samples which have elapsed in
        // this buffer
        let samples_elapsed = model.current_sample_idx();

        // push note event to the note handler
        model
            .audio_senders
            .note_event
            .send(NoteEvent::NoteOn { note, timing: samples_elapsed })
            .unwrap();
    }
}

/// Function for handling key releases.
pub fn key_released(_app: &App, model: &mut Model, key: Key) {
    // get midi note value from keyboard input
    let key_note_value = Note::key_value(&key);

    if let Some(v) = model.pressed_keys.get_mut(&key) {
        *v = false;
    }

    if let Some(note) = key_note_value {
        // transpose octave if higher keys are pressed
        let octave = octave_from_key(model.octave, key);

        // get the midi note value from octave and key note
        let note = octave.starting_midi_note() + note;

        // get the approximate number of samples which have elapsed in
        // this buffer
        let samples_elapsed = model.current_sample_idx();

        // push note event to the note handler
        model
            .audio_senders
            .note_event
            .send(NoteEvent::NoteOff { note, timing: samples_elapsed })
            .unwrap();
    }
}

/// Returns the correctly transposed octave from the computer keyboard input.
fn octave_from_key(octave: Octave, key: Key) -> Octave {
    if matches!(key, Key::K | Key::O | Key::L | Key::P) {
        octave.transpose(1)
    }
    else {
        octave
    }
}
