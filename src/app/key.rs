use super::{audio::AudioModel, *};
use crate::musical::*;
// use crate::prelude::*;

/// Function for handling keypresses.
pub fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        // stop audio playback
        Key::Space => {
            if model.audio_stream.is_playing() {
                model.audio_stream.pause().unwrap();
            }
            else {
                model.audio_stream.play().unwrap();
            }
        }
        Key::Z => {
            model.octave.decrease();
        }
        Key::X => {
            model.octave.increase();
        }
        _ => (),
    };

    if let Some(v) = model.pressed_keys.get_mut(&key) {
        if *v {
            return;
        }
        *v = true;
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
        let mut note_handler = model.note_handler.lock().unwrap();
        note_handler
            .push_event(NoteEvent::NoteOn { note, timing: samples_elapsed });
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
        let mut note_handler = model.note_handler.lock().unwrap();
        note_handler
            .push_event(NoteEvent::NoteOff { note, timing: samples_elapsed });
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
